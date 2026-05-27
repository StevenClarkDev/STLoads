use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use chrono::NaiveDateTime;
use db::{
    dispatch::{LoadLegScopeRecord, find_load_leg_scope},
    marketplace::{
        counter_offer, create_message, ensure_rate_confirmation_ref, find_conversation_by_id,
        find_offer_by_id, mark_conversation_read, review_offer,
    },
};
use domain::{
    auth::UserRole,
    marketplace::{
        MarketplaceModuleContract, OfferStatus, OfferStatusDescriptor, marketplace_module_contract,
        offer_status_descriptors,
    },
};
use serde::{Deserialize, Serialize};
use shared::{
    ApiResponse, ChatSendMessageRequest, ChatSendMessageResponse, ChatWorkspaceScreen,
    ConversationReadResponse, OfferCounterRequest, OfferCounterResponse, OfferReviewDecision,
    OfferReviewRequest, OfferReviewResponse, RateConfirmationResponse, RealtimeEvent,
    RealtimeEventKind, RealtimeTopic,
};

use crate::{auth_session, realtime_bus::RoutedRealtimeEvent, screen_data, state::AppState};

#[derive(Debug, Serialize)]
struct MarketplaceOverview {
    contract: MarketplaceModuleContract,
    offer_statuses: usize,
    realtime_channels: usize,
    screen_routes: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct ChatWorkspaceQuery {
    conversation_id: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/offer-statuses", get(offer_statuses))
        .route("/chat-workspace", get(chat_workspace))
        .route("/offers/{offer_id}/review", post(review_offer_handler))
        .route("/offers/{offer_id}/counter", post(counter_offer_handler))
        .route(
            "/offers/{offer_id}/rate-confirmation",
            post(rate_confirmation_handler),
        )
        .route("/tenders/{offer_id}/decision", post(review_offer_handler))
        .route(
            "/conversations/{conversation_id}/messages",
            post(send_message_handler),
        )
        .route(
            "/conversations/{conversation_id}/read",
            post(mark_conversation_read_handler),
        )
}

async fn index() -> Json<ApiResponse<MarketplaceOverview>> {
    let contract = marketplace_module_contract();
    Json(ApiResponse::ok(MarketplaceOverview {
        offer_statuses: offer_status_descriptors().len(),
        realtime_channels: contract.realtime_channels.len(),
        screen_routes: vec!["/marketplace/chat-workspace"],
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("marketplace route group ready"))
}

async fn contract() -> Json<ApiResponse<MarketplaceModuleContract>> {
    Json(ApiResponse::ok(marketplace_module_contract()))
}

async fn offer_statuses() -> Json<ApiResponse<Vec<OfferStatusDescriptor>>> {
    Json(ApiResponse::ok(offer_status_descriptors().to_vec()))
}

async fn chat_workspace(
    State(state): State<AppState>,
    Query(query): Query<ChatWorkspaceQuery>,
    headers: HeaderMap,
) -> Json<ApiResponse<ChatWorkspaceScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        screen_data::chat_workspace_screen(&state, viewer.as_ref(), query.conversation_id).await,
    ))
}

async fn review_offer_handler(
    State(state): State<AppState>,
    Path(offer_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<OfferReviewRequest>,
) -> Json<ApiResponse<OfferReviewResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Unavailable".into(),
            message: format!(
                "Offer review is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Unauthorized".into(),
            message: "Sign in before reviewing marketplace offers.".into(),
        }));
    };

    if session.user.primary_role() == Some(UserRole::Carrier) {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Forbidden".into(),
            message: "Carrier accounts cannot accept or decline offers from the Rust review route."
                .into(),
        }));
    }

    let Ok(Some(existing_offer)) = find_offer_by_id(pool, offer_id).await else {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Missing".into(),
            message: "The requested offer was not found.".into(),
        }));
    };

    let is_admin = session.user.primary_role() == Some(UserRole::Admin);
    let load_scope = find_load_leg_scope(pool, existing_offer.load_leg_id)
        .await
        .ok()
        .flatten();
    let same_organization = load_scope
        .as_ref()
        .map(|scope| {
            crate::auth_session::session_matches_organization(&session, scope.organization_id)
        })
        .unwrap_or(false);
    let is_load_owner = load_scope
        .as_ref()
        .and_then(|scope| scope.load_owner_user_id)
        == Some(session.user.id);

    if !same_organization || (!is_admin && !is_load_owner) {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: existing_offer.load_leg_id,
            status_label: "Forbidden".into(),
            message: "Only the authenticated load owner or an admin can review this offer.".into(),
        }));
    }

    if !existing_offer
        .status()
        .map(OfferStatus::is_reviewable)
        .unwrap_or(false)
    {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: existing_offer.load_leg_id,
            status_label: "Locked".into(),
            message:
                "Only pending or countered offers can be accepted or declined from this Rust marketplace route."
                    .into(),
        }));
    }

    match review_offer(
        pool,
        offer_id,
        matches!(payload.decision, OfferReviewDecision::Accept),
        Some(session.user.id),
    )
    .await
    {
        Ok(Some(offer)) => {
            let (status_label, action_label) = match payload.decision {
                OfferReviewDecision::Accept => ("Accepted", "accepted"),
                OfferReviewDecision::Decline => ("Declined", "declined"),
            };

            let mut target_user_ids = collect_scope_user_ids(load_scope.as_ref());
            if offer.carrier_id > 0 {
                target_user_ids.push(offer.carrier_id as u64);
            }
            if let Some(conversation_id) = offer.conversation_id
                && let Ok(Some(conversation)) = find_conversation_by_id(pool, conversation_id).await
            {
                target_user_ids.extend(conversation_participant_user_ids(&conversation));
            }
            target_user_ids.push(session.user.id.max(0) as u64);
            target_user_ids.sort_unstable();
            target_user_ids.dedup();

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::OfferReviewed,
                    leg_id: Some(offer.load_leg_id.max(0) as u64),
                    conversation_id: offer.conversation_id.map(|value| value.max(0) as u64),
                    offer_id: Some(offer_id.max(0) as u64),
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: None,
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!(
                        "{} {} offer #{}.",
                        session.user.name, action_label, offer_id
                    ),
                })
                .for_user_ids(target_user_ids)
                .with_topics([
                    RealtimeTopic::Conversation.as_key(),
                    RealtimeTopic::LoadBoard.as_key(),
                ]),
            );

            Json(ApiResponse::ok(OfferReviewResponse {
                success: true,
                offer_id,
                leg_id: offer.load_leg_id,
                status_label: status_label.into(),
                message: format!(
                    "Offer {} from the authenticated Rust marketplace route; scoped participants will refresh through realtime events.",
                    action_label
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Missing".into(),
            message: "The requested offer was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Error".into(),
            message: format!("Offer review failed: {}", error),
        })),
    }
}

async fn counter_offer_handler(
    State(state): State<AppState>,
    Path(offer_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<OfferCounterRequest>,
) -> Json<ApiResponse<OfferCounterResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(OfferCounterResponse {
            success: false,
            offer_id: 0,
            source_offer_id: offer_id,
            leg_id: 0,
            status_label: "Unavailable".into(),
            message: format!(
                "Counteroffers are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(OfferCounterResponse {
            success: false,
            offer_id: 0,
            source_offer_id: offer_id,
            leg_id: 0,
            status_label: "Unauthorized".into(),
            message: "Sign in before creating marketplace counteroffers.".into(),
        }));
    };
    if payload.amount <= 0.0 {
        return Json(ApiResponse::ok(OfferCounterResponse {
            success: false,
            offer_id: 0,
            source_offer_id: offer_id,
            leg_id: 0,
            status_label: "Invalid".into(),
            message: "Counteroffer amount must be greater than zero.".into(),
        }));
    }

    let existing_offer = match find_offer_by_id(pool, offer_id).await {
        Ok(Some(offer)) => offer,
        Ok(None) => {
            return Json(ApiResponse::ok(OfferCounterResponse {
                success: false,
                offer_id: 0,
                source_offer_id: offer_id,
                leg_id: 0,
                status_label: "Missing".into(),
                message: "The requested offer was not found.".into(),
            }));
        }
        Err(error) => {
            return Json(ApiResponse::ok(OfferCounterResponse {
                success: false,
                offer_id: 0,
                source_offer_id: offer_id,
                leg_id: 0,
                status_label: "Error".into(),
                message: format!("Counteroffer lookup failed: {}", error),
            }));
        }
    };

    let load_scope = find_load_leg_scope(pool, existing_offer.load_leg_id)
        .await
        .ok()
        .flatten();
    let is_admin = session.user.primary_role() == Some(UserRole::Admin);
    let is_load_owner = load_scope
        .as_ref()
        .and_then(|scope| scope.load_owner_user_id)
        == Some(session.user.id);
    if !is_admin && !is_load_owner {
        return Json(ApiResponse::ok(OfferCounterResponse {
            success: false,
            offer_id: 0,
            source_offer_id: offer_id,
            leg_id: existing_offer.load_leg_id,
            status_label: "Forbidden".into(),
            message: "Only the load owner or an admin can create this counteroffer.".into(),
        }));
    }

    let expires_at = payload.expires_at.as_deref().and_then(parse_offer_datetime);
    let note = payload
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match counter_offer(
        pool,
        offer_id,
        payload.amount,
        expires_at,
        note,
        Some(session.user.id),
    )
    .await
    {
        Ok(Some(offer)) => Json(ApiResponse::ok(OfferCounterResponse {
            success: true,
            offer_id: offer.id,
            source_offer_id: offer_id,
            leg_id: offer.load_leg_id,
            status_label: offer
                .status()
                .map(|status| status.label().to_string())
                .unwrap_or_else(|| "Countered".into()),
            message: "Counteroffer created and the prior offer was superseded transactionally."
                .into(),
        })),
        Ok(None) => Json(ApiResponse::ok(OfferCounterResponse {
            success: false,
            offer_id: 0,
            source_offer_id: offer_id,
            leg_id: 0,
            status_label: "Missing".into(),
            message: "The requested offer was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(OfferCounterResponse {
            success: false,
            offer_id: 0,
            source_offer_id: offer_id,
            leg_id: existing_offer.load_leg_id,
            status_label: "Error".into(),
            message: format!("Counteroffer failed: {}", error),
        })),
    }
}

async fn rate_confirmation_handler(
    State(state): State<AppState>,
    Path(offer_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<RateConfirmationResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(RateConfirmationResponse {
            success: false,
            offer_id,
            leg_id: 0,
            confirmation_ref: None,
            message: format!(
                "Rate confirmation is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(RateConfirmationResponse {
            success: false,
            offer_id,
            leg_id: 0,
            confirmation_ref: None,
            message: "Sign in before generating rate confirmations.".into(),
        }));
    };
    let existing_offer = match find_offer_by_id(pool, offer_id).await {
        Ok(Some(offer)) => offer,
        _ => {
            return Json(ApiResponse::ok(RateConfirmationResponse {
                success: false,
                offer_id,
                leg_id: 0,
                confirmation_ref: None,
                message: "The requested offer was not found.".into(),
            }));
        }
    };
    let load_scope = find_load_leg_scope(pool, existing_offer.load_leg_id)
        .await
        .ok()
        .flatten();
    let is_admin = session.user.primary_role() == Some(UserRole::Admin);
    let is_load_owner = load_scope
        .as_ref()
        .and_then(|scope| scope.load_owner_user_id)
        == Some(session.user.id);
    if !is_admin && !is_load_owner {
        return Json(ApiResponse::ok(RateConfirmationResponse {
            success: false,
            offer_id,
            leg_id: existing_offer.load_leg_id,
            confirmation_ref: None,
            message: "Only the load owner or an admin can generate this rate confirmation.".into(),
        }));
    }

    match ensure_rate_confirmation_ref(pool, offer_id).await {
        Ok(Some((offer, confirmation_ref))) => Json(ApiResponse::ok(RateConfirmationResponse {
            success: true,
            offer_id,
            leg_id: offer.load_leg_id,
            confirmation_ref: Some(confirmation_ref.clone()),
            message: format!("Rate confirmation {} is ready.", confirmation_ref),
        })),
        Ok(None) => Json(ApiResponse::ok(RateConfirmationResponse {
            success: false,
            offer_id,
            leg_id: 0,
            confirmation_ref: None,
            message: "The requested offer was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(RateConfirmationResponse {
            success: false,
            offer_id,
            leg_id: existing_offer.load_leg_id,
            confirmation_ref: None,
            message: format!("Rate confirmation failed: {}", error),
        })),
    }
}

async fn send_message_handler(
    State(state): State<AppState>,
    Path(conversation_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ChatSendMessageRequest>,
) -> Json<ApiResponse<ChatSendMessageResponse>> {
    let trimmed_body = payload.body.trim().to_string();
    if trimmed_body.is_empty() {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "Message body cannot be empty.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: format!(
                "Message send is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "Sign in before sending chat messages through the Rust marketplace route."
                .into(),
        }));
    };

    let Ok(Some(conversation)) = find_conversation_by_id(pool, conversation_id).await else {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "The requested conversation was not found.".into(),
        }));
    };

    let same_organization = find_load_leg_scope(pool, conversation.load_leg_id)
        .await
        .ok()
        .flatten()
        .map(|scope| {
            crate::auth_session::session_matches_organization(&session, scope.organization_id)
        })
        .unwrap_or(false);
    if !same_organization {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "This conversation belongs to another organization.".into(),
        }));
    }

    if session.user.id != conversation.shipper_id && session.user.id != conversation.carrier_id {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message:
                "This authenticated account is not a participant in the selected conversation."
                    .into(),
        }));
    }

    match create_message(pool, conversation_id, session.user.id, &trimmed_body).await {
        Ok(Some(message)) => {
            let mut target_user_ids = conversation_participant_user_ids(&conversation);
            target_user_ids.sort_unstable();
            target_user_ids.dedup();

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::MessageSent,
                    leg_id: Some(conversation.load_leg_id.max(0) as u64),
                    conversation_id: Some(conversation_id.max(0) as u64),
                    offer_id: None,
                    message_id: Some(message.id.max(0) as u64),
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: Some(message.id.max(0) as u64),
                    summary: format!("{} sent a chat message.", session.user.name),
                })
                .for_user_ids(target_user_ids)
                .with_topics([RealtimeTopic::Conversation.as_key()]),
            );

            Json(ApiResponse::ok(ChatSendMessageResponse {
                success: true,
                conversation_id,
                message_id: message.id,
                message: "Message stored through the authenticated Rust marketplace route.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "The requested conversation was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: format!("Message send failed: {}", error),
        })),
    }
}

async fn mark_conversation_read_handler(
    State(state): State<AppState>,
    Path(conversation_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<ConversationReadResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: format!(
                "Read receipts are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: "Sign in before marking a conversation as read.".into(),
        }));
    };

    let Ok(Some(conversation)) = find_conversation_by_id(pool, conversation_id).await else {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: "The requested conversation was not found.".into(),
        }));
    };

    let same_organization = find_load_leg_scope(pool, conversation.load_leg_id)
        .await
        .ok()
        .flatten()
        .map(|scope| {
            crate::auth_session::session_matches_organization(&session, scope.organization_id)
        })
        .unwrap_or(false);
    if !same_organization {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: "This conversation belongs to another organization.".into(),
        }));
    }

    if session.user.id != conversation.shipper_id && session.user.id != conversation.carrier_id {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message:
                "This authenticated account is not a participant in the selected conversation."
                    .into(),
        }));
    }

    match mark_conversation_read(pool, conversation_id, session.user.id).await {
        Ok(Some(read_state)) => {
            let target_user_ids = conversation_participant_user_ids(&conversation);
            let last_read_message_id = read_state.last_read_message_id;

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::ConversationRead,
                    leg_id: Some(conversation.load_leg_id.max(0) as u64),
                    conversation_id: Some(conversation.id.max(0) as u64),
                    offer_id: None,
                    message_id: last_read_message_id.map(|value| value.max(0) as u64),
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: last_read_message_id.map(|value| value.max(0) as u64),
                    summary: format!("{} read the active conversation.", session.user.name),
                })
                .for_user_ids(target_user_ids)
                .with_topics([RealtimeTopic::Conversation.as_key()]),
            );

            Json(ApiResponse::ok(ConversationReadResponse {
                success: true,
                conversation_id,
                last_read_message_id,
                message: "Conversation marked as read for the authenticated Rust session.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: "No read receipt state could be stored for this conversation.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: format!("Read receipt update failed: {}", error),
        })),
    }
}

fn collect_scope_user_ids(scope: Option<&LoadLegScopeRecord>) -> Vec<u64> {
    let mut user_ids = Vec::new();

    if let Some(scope) = scope {
        if let Some(owner_id) = scope.load_owner_user_id
            && owner_id > 0
        {
            user_ids.push(owner_id as u64);
        }

        if let Some(booked_carrier_id) = scope.booked_carrier_id
            && booked_carrier_id > 0
        {
            user_ids.push(booked_carrier_id as u64);
        }
    }

    user_ids
}

fn conversation_participant_user_ids(
    conversation: &db::marketplace::ConversationRecord,
) -> Vec<u64> {
    let mut user_ids = Vec::new();

    if conversation.shipper_id > 0 {
        user_ids.push(conversation.shipper_id as u64);
    }

    if conversation.carrier_id > 0 {
        user_ids.push(conversation.carrier_id as u64);
    }

    user_ids
}

fn parse_offer_datetime(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M"))
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                .map(|date| date.and_hms_opt(23, 59, 59))
                .map(Option::unwrap)
        })
        .ok()
}
