use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use db::tms::{
    MaterializedHandoffResult, TmsWebhookMutationResult, apply_status_webhook, close_handoff,
    create_sync_error, find_active_handoff_by_tms_load, find_handoff_by_id, push_handoff,
    queue_handoff, requeue_handoff, withdraw_handoff,
};
use domain::tms::{
    HandoffStatus, HandoffStatusDescriptor, ReconciliationActionDescriptor, TmsModuleContract,
    TmsStatus, TmsStatusDescriptor, TmsWebhookSurfaceDescriptor, handoff_status_descriptors,
    reconciliation_action_descriptors, tms_module_contract, tms_status_descriptors,
    tms_webhook_surfaces,
};
use serde::{Deserialize, Serialize};
use shared::{
    ApiResponse, RealtimeEvent, RealtimeEventKind, RealtimeTopic, TmsBulkStatusWebhookRequest,
    TmsBulkStatusWebhookResponse, TmsCloseRequest, TmsHandoffPayload, TmsHandoffResponse,
    TmsRequeueRequest, TmsStatusWebhookRequest, TmsWebhookResponse, TmsWithdrawRequest,
};

use crate::{
    auth_session, auth_session::ResolvedSession, realtime_bus::RoutedRealtimeEvent, state::AppState,
};

#[derive(Debug, Serialize)]
struct TmsOverview {
    contract: TmsModuleContract,
    handoff_statuses: usize,
    tms_statuses: usize,
    webhook_surfaces: usize,
}

#[derive(Debug, Deserialize)]
struct LifecycleWebhookRequest {
    tms_load_id: String,
    tenant_id: String,
    reason: Option<String>,
    pushed_by: Option<String>,
    source_module: Option<String>,
}

pub fn router() -> Router<crate::state::AppState> {
    metadata_router().merge(integration_router())
}

pub fn integration_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/push", post(push))
        .route("/queue", post(queue))
        .route("/requeue", post(requeue))
        .route("/withdraw", post(withdraw))
        .route("/close", post(close))
        .route("/webhook/status", post(webhook_status))
        .route("/webhook/bulk-status", post(webhook_bulk_status))
        .route("/webhook/cancel", post(webhook_cancel))
        .route("/webhook/close", post(webhook_close))
}

fn metadata_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/handoff-statuses", get(handoff_statuses))
        .route("/dispatch-statuses", get(dispatch_statuses))
        .route("/webhook-surfaces", get(webhook_surfaces))
        .route("/reconciliation-actions", get(reconciliation_actions))
}

async fn index() -> Json<ApiResponse<TmsOverview>> {
    let contract = tms_module_contract();
    Json(ApiResponse::ok(TmsOverview {
        handoff_statuses: handoff_status_descriptors().len(),
        tms_statuses: tms_status_descriptors().len(),
        webhook_surfaces: contract.webhook_surfaces.len(),
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("tms route group ready"))
}

async fn contract() -> Json<ApiResponse<TmsModuleContract>> {
    Json(ApiResponse::ok(tms_module_contract()))
}

async fn handoff_statuses() -> Json<ApiResponse<Vec<HandoffStatusDescriptor>>> {
    Json(ApiResponse::ok(handoff_status_descriptors().to_vec()))
}

async fn dispatch_statuses() -> Json<ApiResponse<Vec<TmsStatusDescriptor>>> {
    Json(ApiResponse::ok(tms_status_descriptors().to_vec()))
}

async fn webhook_surfaces() -> Json<ApiResponse<Vec<TmsWebhookSurfaceDescriptor>>> {
    Json(ApiResponse::ok(tms_webhook_surfaces().to_vec()))
}

async fn reconciliation_actions() -> Json<ApiResponse<Vec<ReconciliationActionDescriptor>>> {
    Json(ApiResponse::ok(
        reconciliation_action_descriptors().to_vec(),
    ))
}

async fn push(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TmsHandoffPayload>,
) -> Result<Json<ApiResponse<TmsHandoffResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(unavailable_handoff_response(
            &state, "Push",
        ))));
    };

    if let Some(message) = validate_handoff_payload(&payload) {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: None,
            load_id: None,
            load_number: None,
            status_label: "Validation Error".into(),
            message,
        })));
    }

    if let Some(existing_handoff) =
        find_active_handoff_by_tms_load(pool, &payload.tms_load_id, &payload.tenant_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let _ = create_sync_error(
            pool,
            Some(existing_handoff.id),
            "duplicate_publish",
            "critical",
            "TMS tried to publish an already active handoff.",
            Some("A duplicate push was rejected because the handoff is already active."),
            payload.source_module.as_deref(),
            payload.pushed_by.as_deref(),
        )
        .await;

        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(existing_handoff.id),
            load_id: existing_handoff.load_id,
            load_number: None,
            status_label: "Duplicate".into(),
            message: "This TMS load is already published or active in the Rust handoff layer."
                .into(),
        })));
    }

    match push_handoff(pool, &payload).await {
        Ok(result) => {
            publish_tms_lifecycle_events(
                &state,
                &result,
                actor_session.as_ref(),
                true,
                false,
                true,
                format!(
                    "Rust TMS push published handoff #{} for {}.",
                    result.handoff.id, payload.tms_load_id
                ),
            );

            Ok(Json(ApiResponse::ok(TmsHandoffResponse {
                success: true,
                handoff_id: Some(result.handoff.id),
                load_id: result.load_id,
                load_number: result.load_number,
                status_label: "Published".into(),
                message: "Handoff pushed and materialized through the Rust TMS route.".into(),
            })))
        }
        Err(error) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: None,
            load_id: None,
            load_number: None,
            status_label: "Push Failed".into(),
            message: format!("Push failed: {}", error),
        }))),
    }
}

async fn queue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TmsHandoffPayload>,
) -> Result<Json<ApiResponse<TmsHandoffResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(unavailable_handoff_response(
            &state, "Queue",
        ))));
    };

    if let Some(message) = validate_handoff_payload(&payload) {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: None,
            load_id: None,
            load_number: None,
            status_label: "Validation Error".into(),
            message,
        })));
    }

    match queue_handoff(pool, &payload).await {
        Ok(handoff) => {
            let result = MaterializedHandoffResult {
                handoff,
                load_id: None,
                load_number: None,
                action_label: "queued".into(),
            };
            publish_tms_lifecycle_events(
                &state,
                &result,
                actor_session.as_ref(),
                true,
                false,
                false,
                format!("Rust TMS queue accepted handoff #{}.", result.handoff.id),
            );

            Ok(Json(ApiResponse::ok(TmsHandoffResponse {
                success: true,
                handoff_id: Some(result.handoff.id),
                load_id: None,
                load_number: None,
                status_label: "Queued".into(),
                message: "Handoff queued through the Rust TMS route.".into(),
            })))
        }
        Err(error) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: None,
            load_id: None,
            load_number: None,
            status_label: "Queue Failed".into(),
            message: format!("Queue failed: {}", error),
        }))),
    }
}

async fn requeue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TmsRequeueRequest>,
) -> Result<Json<ApiResponse<TmsHandoffResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(unavailable_handoff_response(
            &state, "Requeue",
        ))));
    };

    let Some(existing_handoff) = find_handoff_by_id(pool, payload.handoff_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Missing".into(),
            message: "The requested handoff was not found.".into(),
        })));
    };

    if !matches!(
        existing_handoff.status.as_str(),
        "push_failed" | "requeue_required"
    ) {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(existing_handoff.id),
            load_id: existing_handoff.load_id,
            load_number: None,
            status_label: "Invalid State".into(),
            message: format!(
                "Handoff #{} is in status '{}' and cannot be requeued.",
                existing_handoff.id, existing_handoff.status
            ),
        })));
    }

    match requeue_handoff(
        pool,
        payload.handoff_id,
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
    )
    .await
    {
        Ok(Some(result)) => {
            publish_tms_lifecycle_events(
                &state,
                &result,
                actor_session.as_ref(),
                true,
                false,
                true,
                format!(
                    "Rust TMS requeue republished handoff #{}.",
                    result.handoff.id
                ),
            );

            Ok(Json(ApiResponse::ok(TmsHandoffResponse {
                success: true,
                handoff_id: Some(result.handoff.id),
                load_id: result.load_id,
                load_number: result.load_number,
                status_label: "Published".into(),
                message: "Handoff requeued and republished through the Rust TMS route.".into(),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Missing".into(),
            message: "The requested handoff was not found.".into(),
        }))),
        Err(error) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Push Failed".into(),
            message: format!("Requeue failed: {}", error),
        }))),
    }
}

async fn withdraw(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TmsWithdrawRequest>,
) -> Result<Json<ApiResponse<TmsHandoffResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(unavailable_handoff_response(
            &state, "Withdraw",
        ))));
    };

    let Some(existing_handoff) = find_handoff_by_id(pool, payload.handoff_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Missing".into(),
            message: "The requested handoff was not found.".into(),
        })));
    };

    if existing_handoff.status != HandoffStatus::Published.as_legacy_label() {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(existing_handoff.id),
            load_id: existing_handoff.load_id,
            load_number: None,
            status_label: "Invalid State".into(),
            message: format!(
                "Handoff #{} is in status '{}' and cannot be withdrawn.",
                existing_handoff.id, existing_handoff.status
            ),
        })));
    }

    match withdraw_handoff(
        pool,
        payload.handoff_id,
        payload.reason.as_deref(),
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
    )
    .await
    {
        Ok(Some(handoff)) => {
            let result = MaterializedHandoffResult {
                handoff,
                load_id: existing_handoff.load_id,
                load_number: None,
                action_label: "withdrawn".into(),
            };
            publish_tms_lifecycle_events(
                &state,
                &result,
                actor_session.as_ref(),
                true,
                true,
                true,
                format!("Rust TMS withdrew handoff #{}.", result.handoff.id),
            );

            Ok(Json(ApiResponse::ok(TmsHandoffResponse {
                success: true,
                handoff_id: Some(result.handoff.id),
                load_id: result.load_id,
                load_number: None,
                status_label: "Withdrawn".into(),
                message: "Handoff withdrawn through the Rust TMS route.".into(),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Missing".into(),
            message: "The requested handoff was not found.".into(),
        }))),
        Err(error) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Withdraw Failed".into(),
            message: format!("Withdraw failed: {}", error),
        }))),
    }
}

async fn close(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TmsCloseRequest>,
) -> Result<Json<ApiResponse<TmsHandoffResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(unavailable_handoff_response(
            &state, "Close",
        ))));
    };

    let Some(existing_handoff) = find_handoff_by_id(pool, payload.handoff_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Missing".into(),
            message: "The requested handoff was not found.".into(),
        })));
    };

    if !matches!(existing_handoff.status.as_str(), "published" | "withdrawn") {
        return Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(existing_handoff.id),
            load_id: existing_handoff.load_id,
            load_number: None,
            status_label: "Invalid State".into(),
            message: format!(
                "Handoff #{} is in status '{}' and cannot be closed.",
                existing_handoff.id, existing_handoff.status
            ),
        })));
    }

    match close_handoff(
        pool,
        payload.handoff_id,
        payload.reason.as_deref(),
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
    )
    .await
    {
        Ok(Some(handoff)) => {
            let result = MaterializedHandoffResult {
                handoff,
                load_id: existing_handoff.load_id,
                load_number: None,
                action_label: "closed".into(),
            };
            publish_tms_lifecycle_events(
                &state,
                &result,
                actor_session.as_ref(),
                true,
                true,
                true,
                format!("Rust TMS closed handoff #{}.", result.handoff.id),
            );

            Ok(Json(ApiResponse::ok(TmsHandoffResponse {
                success: true,
                handoff_id: Some(result.handoff.id),
                load_id: result.load_id,
                load_number: None,
                status_label: "Closed".into(),
                message: "Handoff closed through the Rust TMS route.".into(),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Missing".into(),
            message: "The requested handoff was not found.".into(),
        }))),
        Err(error) => Ok(Json(ApiResponse::ok(TmsHandoffResponse {
            success: false,
            handoff_id: Some(payload.handoff_id),
            load_id: None,
            load_number: None,
            status_label: "Close Failed".into(),
            message: format!("Close failed: {}", error),
        }))),
    }
}

async fn webhook_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TmsStatusWebhookRequest>,
) -> Result<Json<ApiResponse<TmsWebhookResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(TmsWebhookResponse {
            success: false,
            handoff_id: None,
            action_label: "unavailable".into(),
            message: format!(
                "Status webhook handling is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    match apply_status_webhook(pool, &payload).await {
        Ok(Some(result)) => {
            publish_tms_webhook_events(&state, &result, actor_session.as_ref());
            Ok(Json(ApiResponse::ok(TmsWebhookResponse {
                success: true,
                handoff_id: Some(result.handoff.id),
                action_label: result.action_label,
                message: result.message,
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(TmsWebhookResponse {
            success: false,
            handoff_id: None,
            action_label: "missing".into(),
            message: "No active handoff matched the webhook identity.".into(),
        }))),
        Err(error) => Ok(Json(ApiResponse::ok(TmsWebhookResponse {
            success: false,
            handoff_id: None,
            action_label: "error".into(),
            message: format!("Status webhook failed: {}", error),
        }))),
    }
}

async fn webhook_bulk_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TmsBulkStatusWebhookRequest>,
) -> Result<Json<ApiResponse<TmsBulkStatusWebhookResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(TmsBulkStatusWebhookResponse {
            processed: 0,
            updated: 0,
            missing: payload.updates.len(),
            failed: 0,
            messages: vec![format!(
                "Bulk webhook handling is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let mut updated = 0_usize;
    let mut missing = 0_usize;
    let mut failed = 0_usize;
    let mut messages = Vec::new();

    for update in &payload.updates {
        match apply_status_webhook(pool, update).await {
            Ok(Some(result)) => {
                updated += 1;
                messages.push(format!(
                    "Updated handoff #{} with action {}.",
                    result.handoff.id, result.action_label
                ));
                publish_tms_webhook_events(&state, &result, actor_session.as_ref());
            }
            Ok(None) => {
                missing += 1;
                messages.push(format!(
                    "No active handoff matched {} / {}.",
                    update.tms_load_id, update.tenant_id
                ));
            }
            Err(error) => {
                failed += 1;
                messages.push(format!(
                    "Webhook update failed for {} / {}: {}",
                    update.tms_load_id, update.tenant_id, error
                ));
            }
        }
    }

    Ok(Json(ApiResponse::ok(TmsBulkStatusWebhookResponse {
        processed: payload.updates.len(),
        updated,
        missing,
        failed,
        messages,
    })))
}

async fn webhook_cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LifecycleWebhookRequest>,
) -> Result<Json<ApiResponse<TmsWebhookResponse>>, StatusCode> {
    webhook_status(
        State(state),
        headers,
        Json(TmsStatusWebhookRequest {
            tms_load_id: payload.tms_load_id,
            tenant_id: payload.tenant_id,
            tms_status: TmsStatus::Cancelled.as_legacy_label().into(),
            status_at: None,
            source_module: payload.source_module,
            pushed_by: payload.pushed_by,
            detail: payload.reason,
            rate_update: None,
        }),
    )
    .await
}

async fn webhook_close(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LifecycleWebhookRequest>,
) -> Result<Json<ApiResponse<TmsWebhookResponse>>, StatusCode> {
    let actor_session = authorize_tms_request(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(TmsWebhookResponse {
            success: false,
            handoff_id: None,
            action_label: "unavailable".into(),
            message: format!(
                "Close webhook handling is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let Some(existing_handoff) =
        find_active_handoff_by_tms_load(pool, &payload.tms_load_id, &payload.tenant_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(TmsWebhookResponse {
            success: false,
            handoff_id: None,
            action_label: "missing".into(),
            message: "No active handoff matched the close webhook identity.".into(),
        })));
    };

    match close_handoff(
        pool,
        existing_handoff.id,
        payload.reason.as_deref(),
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
    )
    .await
    {
        Ok(Some(handoff)) => {
            let result = MaterializedHandoffResult {
                handoff,
                load_id: existing_handoff.load_id,
                load_number: None,
                action_label: "closed".into(),
            };
            publish_tms_lifecycle_events(
                &state,
                &result,
                actor_session.as_ref(),
                true,
                true,
                true,
                format!("TMS close webhook archived handoff #{}.", result.handoff.id),
            );

            Ok(Json(ApiResponse::ok(TmsWebhookResponse {
                success: true,
                handoff_id: Some(result.handoff.id),
                action_label: "closed".into(),
                message: "Close webhook archived the STLOADS handoff through Rust.".into(),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(TmsWebhookResponse {
            success: false,
            handoff_id: None,
            action_label: "missing".into(),
            message: "No active handoff matched the close webhook identity.".into(),
        }))),
        Err(error) => Ok(Json(ApiResponse::ok(TmsWebhookResponse {
            success: false,
            handoff_id: Some(existing_handoff.id),
            action_label: "error".into(),
            message: format!("Close webhook failed: {}", error),
        }))),
    }
}

async fn authorize_tms_request(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<ResolvedSession>, StatusCode> {
    if let Some(expected_secret) = state.config.tms_shared_secret.as_deref() {
        let supplied_secret = headers
            .get("x-tms-shared-secret")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if supplied_secret == Some(expected_secret) {
            return Ok(None);
        }
    }

    let Some(session) = auth_session::resolve_session_from_headers(state, headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let allowed = session.session.permissions.iter().any(|permission| {
        permission == "manage_tms_operations" || permission == "access_admin_portal"
    });

    if allowed {
        Ok(Some(session))
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn validate_handoff_payload(payload: &TmsHandoffPayload) -> Option<String> {
    if payload.tms_load_id.trim().is_empty() {
        return Some("tms_load_id is required.".into());
    }
    if payload.tenant_id.trim().is_empty() {
        return Some("tenant_id is required.".into());
    }
    if payload.party_type.trim().is_empty() {
        return Some("party_type is required.".into());
    }
    if payload.freight_mode.trim().is_empty() {
        return Some("freight_mode is required.".into());
    }
    if payload.equipment_type.trim().is_empty() {
        return Some("equipment_type is required.".into());
    }
    if payload.pickup_address.trim().is_empty() || payload.dropoff_address.trim().is_empty() {
        return Some("pickup_address and dropoff_address are required.".into());
    }
    if payload.pickup_city.trim().is_empty() || payload.dropoff_city.trim().is_empty() {
        return Some("pickup_city and dropoff_city are required.".into());
    }
    if payload.weight <= 0.0 {
        return Some("weight must be greater than zero.".into());
    }
    if payload.board_rate < 0.0 {
        return Some("board_rate cannot be negative.".into());
    }
    if !matches!(payload.bid_type.trim(), "Fixed" | "Open" | "fixed" | "open") {
        return Some("bid_type must be Fixed or Open.".into());
    }

    None
}

fn publish_tms_lifecycle_events(
    state: &AppState,
    _result: &MaterializedHandoffResult,
    actor_session: Option<&ResolvedSession>,
    notify_operations: bool,
    notify_reconciliation: bool,
    notify_load_board: bool,
    summary: String,
) {
    if notify_operations {
        let mut event = RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::TmsOperationsUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: actor_session.map(|session| session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_tms_operations"])
        .with_topics([
            RealtimeTopic::AdminTmsOperations.as_key(),
            RealtimeTopic::AdminDashboard.as_key(),
        ]);

        if notify_load_board {
            event = event
                .for_permission_keys(["manage_marketplace", "manage_loads", "manage_dispatch_desk"])
                .with_topics([RealtimeTopic::LoadBoard.as_key()]);
        }

        state.publish_realtime(event);
    }

    if notify_reconciliation {
        state.publish_realtime(
            RoutedRealtimeEvent::new(RealtimeEvent {
                kind: RealtimeEventKind::TmsReconciliationUpdated,
                leg_id: None,
                conversation_id: None,
                offer_id: None,
                message_id: None,
                actor_user_id: actor_session.map(|session| session.user.id.max(0) as u64),
                subject_user_id: None,
                presence_state: None,
                last_read_message_id: None,
                summary,
            })
            .for_permission_keys(["manage_tms_operations"])
            .with_topics([
                RealtimeTopic::AdminTmsReconciliation.as_key(),
                RealtimeTopic::AdminDashboard.as_key(),
            ]),
        );
    }
}

fn publish_tms_webhook_events(
    state: &AppState,
    result: &TmsWebhookMutationResult,
    actor_session: Option<&ResolvedSession>,
) {
    let notify_reconciliation = result.action_label != "status_update";
    let lifecycle = MaterializedHandoffResult {
        handoff: result.handoff.clone(),
        load_id: result.handoff.load_id,
        load_number: None,
        action_label: result.action_label.clone(),
    };

    publish_tms_lifecycle_events(
        state,
        &lifecycle,
        actor_session,
        true,
        notify_reconciliation,
        true,
        result.message.clone(),
    );
}

fn unavailable_handoff_response(state: &AppState, action_label: &str) -> TmsHandoffResponse {
    TmsHandoffResponse {
        success: false,
        handoff_id: None,
        load_id: None,
        load_number: None,
        status_label: "Unavailable".into(),
        message: format!(
            "{} is unavailable because the database is {} on {}.",
            action_label,
            state.database_state(),
            state.config.deployment_target
        ),
    }
}
