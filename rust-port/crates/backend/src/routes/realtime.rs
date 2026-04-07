use axum::{
    Router,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use db::marketplace::{
    ConversationRecord, delete_conversation_presence, find_conversation_by_id,
    upsert_conversation_presence,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::time::{self, Duration};
use tracing::warn;

use crate::{
    auth_session, auth_session::ResolvedSession, realtime_bus::RoutedRealtimeEvent, state::AppState,
};
use shared::{RealtimeEvent, RealtimeEventKind, RealtimeTopic};

const PRESENCE_HEARTBEAT_SECONDS: u64 = 20;

#[derive(Debug, Deserialize)]
struct RealtimeQuery {
    token: String,
    conversation_id: Option<i64>,
    topics: Option<String>,
}

#[derive(Debug, Clone)]
struct ConversationPresenceScope {
    conversation: ConversationRecord,
    participant_user_ids: Vec<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(websocket))
}

async fn websocket(
    websocket: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<RealtimeQuery>,
) -> Response {
    let requested_topics = parse_topics(query.topics.as_deref());

    match auth_session::resolve_session_from_token(&state, &query.token).await {
        Ok(Some(session)) => {
            match resolve_presence_scope(&state, &session, query.conversation_id).await {
                Ok(presence_scope) => websocket
                    .on_upgrade(move |socket| {
                        handle_socket(
                            socket,
                            state,
                            session,
                            query.token,
                            presence_scope,
                            requested_topics,
                        )
                    })
                    .into_response(),
                Err(status) => status.into_response(),
            }
        }
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(error) => {
            warn!(error = %error, "failed to resolve websocket session");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn resolve_presence_scope(
    state: &AppState,
    session: &ResolvedSession,
    conversation_id: Option<i64>,
) -> Result<Option<ConversationPresenceScope>, StatusCode> {
    let Some(conversation_id) = conversation_id else {
        return Ok(None);
    };

    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some(conversation) = find_conversation_by_id(pool, conversation_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    if session.user.id != conversation.shipper_id && session.user.id != conversation.carrier_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Some(ConversationPresenceScope {
        participant_user_ids: conversation_participant_user_ids(&conversation),
        conversation,
    }))
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    session: ResolvedSession,
    token: String,
    presence_scope: Option<ConversationPresenceScope>,
    requested_topics: Vec<String>,
) {
    let mut receiver = state.realtime_tx.subscribe();
    let user_id = session.user.id.max(0) as u64;
    let role_key = auth_session::role_key(session.user.primary_role());
    let session_permissions = session.session.permissions.clone();
    let (mut sender, mut incoming) = socket.split();
    let mut heartbeat = time::interval(Duration::from_secs(PRESENCE_HEARTBEAT_SECONDS));

    if let Some(scope) = presence_scope.as_ref() {
        sync_presence(&state, scope, session.user.id, "online").await;
        publish_presence_event(&state, scope, &session, "online", "joined the conversation");
    }

    loop {
        tokio::select! {
            recv = receiver.recv() => {
                match recv {
                    Ok(event) => {
                        if !event.should_deliver_to(
                            user_id,
                            &role_key,
                            &session_permissions,
                            &requested_topics,
                        ) {
                            continue;
                        }

                        if send_event(&mut sender, &event.payload).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            _ = heartbeat.tick() => {
                let session_still_valid = auth_session::resolve_session_from_token(&state, &token)
                    .await
                    .ok()
                    .flatten()
                    .is_some();

                if !session_still_valid {
                    let invalidated = RealtimeEvent {
                        kind: RealtimeEventKind::SessionInvalidated,
                        leg_id: None,
                        conversation_id: presence_scope.as_ref().map(|scope| scope.conversation.id.max(0) as u64),
                        offer_id: None,
                        message_id: None,
                        actor_user_id: Some(user_id),
                        subject_user_id: Some(user_id),
                        presence_state: None,
                        last_read_message_id: None,
                        summary: "The Rust websocket session expired and must reconnect.".into(),
                    };
                    let _ = send_event(&mut sender, &invalidated).await;
                    break;
                }

                if let Some(scope) = presence_scope.as_ref() {
                    sync_presence(&state, scope, session.user.id, "online").await;
                }
            }
            incoming_message = incoming.next() => {
                match incoming_message {
                    Some(Ok(Message::Ping(payload))) => {
                        if sender.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        warn!(error = %error, user_id, "websocket receive failed");
                        break;
                    }
                }
            }
        }
    }

    if let Some(scope) = presence_scope.as_ref() {
        clear_presence(&state, scope, session.user.id).await;
        publish_presence_event(&state, scope, &session, "offline", "left the conversation");
    }

    let _ = sender.close().await;
}

async fn send_event(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: &RealtimeEvent,
) -> Result<(), ()> {
    let Ok(payload) = serde_json::to_string(event) else {
        return Ok(());
    };

    sender
        .send(Message::Text(payload.into()))
        .await
        .map_err(|_| ())
}

async fn sync_presence(
    state: &AppState,
    scope: &ConversationPresenceScope,
    user_id: i64,
    presence_state: &str,
) {
    let Some(pool) = state.pool.as_ref() else {
        return;
    };

    if let Err(error) =
        upsert_conversation_presence(pool, scope.conversation.id, user_id, presence_state).await
    {
        warn!(error = %error, conversation_id = scope.conversation.id, user_id, "failed to upsert conversation presence");
    }
}

async fn clear_presence(state: &AppState, scope: &ConversationPresenceScope, user_id: i64) {
    let Some(pool) = state.pool.as_ref() else {
        return;
    };

    if let Err(error) = delete_conversation_presence(pool, scope.conversation.id, user_id).await {
        warn!(error = %error, conversation_id = scope.conversation.id, user_id, "failed to delete conversation presence");
    }
}

fn publish_presence_event(
    state: &AppState,
    scope: &ConversationPresenceScope,
    session: &ResolvedSession,
    presence_state: &str,
    action_label: &str,
) {
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::ConversationPresenceChanged,
            leg_id: Some(scope.conversation.load_leg_id.max(0) as u64),
            conversation_id: Some(scope.conversation.id.max(0) as u64),
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: Some(session.user.id.max(0) as u64),
            presence_state: Some(presence_state.to_string()),
            last_read_message_id: None,
            summary: format!("{} {}.", session.user.name, action_label),
        })
        .for_user_ids(scope.participant_user_ids.iter().copied())
        .with_topics([RealtimeTopic::Conversation.as_key()]),
    );
}

fn conversation_participant_user_ids(conversation: &ConversationRecord) -> Vec<u64> {
    let mut user_ids = Vec::new();

    if conversation.shipper_id > 0 {
        user_ids.push(conversation.shipper_id as u64);
    }

    if conversation.carrier_id > 0 {
        user_ids.push(conversation.carrier_id as u64);
    }

    user_ids
}

fn parse_topics(raw_topics: Option<&str>) -> Vec<String> {
    raw_topics
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|topic| !topic.is_empty())
        .map(|topic| topic.to_string())
        .collect()
}
