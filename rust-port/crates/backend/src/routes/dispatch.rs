use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use db::dispatch::{book_load_leg, find_load_leg_by_id, find_load_leg_scope};
use domain::{
    auth::UserRole,
    dispatch::{
        LegacyLoadLegStatusDescriptor, LoadModuleContract, legacy_load_leg_status_descriptors,
        load_module_contract,
    },
};
use serde::{Deserialize, Serialize};
use shared::{
    ApiResponse, BookLoadLegRequest, BookLoadLegResponse, LoadBoardScreen, RealtimeEvent,
    RealtimeEventKind, RealtimeTopic,
};

use crate::{auth_session, realtime_bus::RoutedRealtimeEvent, screen_data, state::AppState};

#[derive(Debug, Serialize)]
struct DispatchOverview {
    contract: LoadModuleContract,
    legacy_status_count: usize,
    document_kinds: usize,
    screen_routes: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct LoadBoardQuery {
    tab: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/legacy-statuses", get(legacy_statuses))
        .route("/load-board", get(load_board))
        .route("/load-board/{leg_id}/book", post(book_leg))
}

async fn index() -> Json<ApiResponse<DispatchOverview>> {
    let contract = load_module_contract();
    Json(ApiResponse::ok(DispatchOverview {
        document_kinds: contract.document_kinds.len(),
        legacy_status_count: legacy_load_leg_status_descriptors().len(),
        screen_routes: vec!["/dispatch/load-board"],
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("dispatch route group ready"))
}

async fn contract() -> Json<ApiResponse<LoadModuleContract>> {
    Json(ApiResponse::ok(load_module_contract()))
}

async fn legacy_statuses() -> Json<ApiResponse<Vec<LegacyLoadLegStatusDescriptor>>> {
    Json(ApiResponse::ok(
        legacy_load_leg_status_descriptors().to_vec(),
    ))
}

async fn load_board(
    State(state): State<AppState>,
    Query(query): Query<LoadBoardQuery>,
    headers: HeaderMap,
) -> Json<ApiResponse<LoadBoardScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        screen_data::load_board_screen(&state, viewer.as_ref(), query.tab).await,
    ))
}

async fn book_leg(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<BookLoadLegRequest>,
) -> Json<ApiResponse<BookLoadLegResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Unavailable".into(),
            message: format!(
                "Booking action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Unauthorized".into(),
            message: "Sign in as a carrier before booking a leg from the Rust load board.".into(),
        }));
    };

    if session.user.primary_role() != Some(UserRole::Carrier) {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Forbidden".into(),
            message:
                "Only authenticated carrier accounts can self-book a load leg in this Rust slice."
                    .into(),
        }));
    }

    let Ok(Some(existing_leg)) = find_load_leg_by_id(pool, leg_id).await else {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Missing".into(),
            message: "The requested load leg was not found.".into(),
        }));
    };

    if existing_leg.booked_carrier_id == Some(session.user.id) {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: true,
            leg_id,
            status_label: "Booked".into(),
            message: "This load leg is already booked by the authenticated carrier account.".into(),
        }));
    }

    if existing_leg.booked_carrier_id.is_some() || existing_leg.status_id >= 4 {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Locked".into(),
            message: "This load leg is no longer open for carrier self-booking.".into(),
        }));
    }

    match book_load_leg(
        pool,
        leg_id,
        session.user.id,
        payload.booked_amount,
        Some(session.user.id),
    )
    .await
    {
        Ok(Some(updated_leg)) => {
            let mut target_user_ids = vec![session.user.id.max(0) as u64];
            if let Ok(Some(scope)) = find_load_leg_scope(pool, leg_id).await {
                if let Some(owner_id) = scope.load_owner_user_id {
                    if owner_id > 0 {
                        target_user_ids.push(owner_id as u64);
                    }
                }
                if let Some(booked_carrier_id) = scope.booked_carrier_id {
                    if booked_carrier_id > 0 {
                        target_user_ids.push(booked_carrier_id as u64);
                    }
                }
            }
            target_user_ids.sort_unstable();
            target_user_ids.dedup();

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::LoadLegBooked,
                    leg_id: Some(leg_id.max(0) as u64),
                    conversation_id: None,
                    offer_id: updated_leg
                        .accepted_offer_id
                        .map(|value| value.max(0) as u64),
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!("{} booked load leg #{}.", session.user.name, leg_id),
                })
                .for_user_ids(target_user_ids)
                .for_role_keys(["carrier"])
                .with_topics([RealtimeTopic::LoadBoard.as_key()]),
            );

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::PaymentsOperationsUpdated,
                    leg_id: Some(leg_id.max(0) as u64),
                    conversation_id: None,
                    offer_id: updated_leg
                        .accepted_offer_id
                        .map(|value| value.max(0) as u64),
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!(
                        "Booking changed payment follow-up readiness for load leg #{}.",
                        leg_id
                    ),
                })
                .for_permission_keys(["manage_payments"])
                .with_topics([RealtimeTopic::AdminPayments.as_key()]),
            );

            Json(ApiResponse::ok(BookLoadLegResponse {
                success: true,
                leg_id,
                status_label: "Booked".into(),
                message: "Load leg booked from the authenticated Rust dispatch route; the board will refresh through scoped realtime updates.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Missing".into(),
            message: "The requested load leg was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Error".into(),
            message: format!("Booking action failed: {}", error),
        })),
    }
}
