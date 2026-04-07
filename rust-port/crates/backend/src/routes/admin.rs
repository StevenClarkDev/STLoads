use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use db::tms::resolve_sync_error;
use serde::{Deserialize, Serialize};
use shared::{
    ApiResponse, RealtimeEvent, RealtimeEventKind, RealtimeTopic, ResolveSyncErrorRequest,
    ResolveSyncErrorResponse, StloadsOperationsScreen, StloadsReconciliationScreen,
};

use crate::{
    auth_session, auth_session::ResolvedSession, realtime_bus::RoutedRealtimeEvent, screen_data,
    state::AppState,
};

#[derive(Debug, Serialize)]
struct AdminOverview {
    screen_routes: Vec<&'static str>,
    operational_views: usize,
    notes: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct OperationsQuery {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReconciliationQuery {
    action: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/stloads/operations", get(stloads_operations))
        .route("/stloads/reconciliation", get(stloads_reconciliation))
        .route(
            "/stloads/sync-errors/{sync_error_id}/resolve",
            post(resolve_sync_error_handler),
        )
}

async fn index(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminOverview>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_payments",
            "manage_master_data",
        ],
    )
    .await?;

    Ok(Json(ApiResponse::ok(AdminOverview {
        screen_routes: vec![
            "/admin/stloads/operations",
            "/admin/stloads/reconciliation",
            "/admin/payments",
            "/admin/master-data",
        ],
        operational_views: 4,
        notes: vec![
            "Admin is now the route home for ops dashboards rather than a single placeholder.",
            "Master-data visibility now lives alongside payments and TMS so load-builder dependencies can be migrated in sequence.",
            "IBM-targeted runtime config is environment-driven so these routes can boot on fresh servers without code edits.",
        ],
    })))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("admin route group ready"))
}

async fn stloads_operations(
    State(state): State<AppState>,
    Query(query): Query<OperationsQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StloadsOperationsScreen>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        screen_data::stloads_operations_screen(&state, query.status).await,
    )))
}

async fn stloads_reconciliation(
    State(state): State<AppState>,
    Query(query): Query<ReconciliationQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StloadsReconciliationScreen>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        screen_data::stloads_reconciliation_screen(&state, query.action).await,
    )))
}

async fn resolve_sync_error_handler(
    State(state): State<AppState>,
    Path(sync_error_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ResolveSyncErrorRequest>,
) -> Result<Json<ApiResponse<ResolveSyncErrorResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: None,
            message: format!(
                "Resolve action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let resolved_by = format!("{} <{}>", session.user.name, session.user.email);
    let resolved_record = resolve_sync_error(
        pool,
        sync_error_id,
        &resolved_by,
        payload.resolution_note.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(sync_error) = resolved_record else {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: None,
            message: "The requested sync error was not found.".into(),
        })));
    };

    if !sync_error.resolved {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: sync_error.handoff_id,
            message: "The selected sync error could not be resolved yet.".into(),
        })));
    }

    let summary = format!(
        "{} resolved {} on sync error #{}.",
        session.user.name, sync_error.error_class, sync_error_id
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::TmsOperationsUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_tms_operations"])
        .with_topics([RealtimeTopic::AdminTmsOperations.as_key()]),
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::TmsReconciliationUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary,
        })
        .for_permission_keys(["manage_tms_operations"])
        .with_topics([RealtimeTopic::AdminTmsReconciliation.as_key()]),
    );

    Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
        success: true,
        sync_error_id,
        handoff_id: sync_error.handoff_id,
        message: "Sync error resolved from the Rust STLOADS admin route.".into(),
    })))
}

async fn require_any_permission(
    state: &AppState,
    headers: &HeaderMap,
    permission_keys: &[&str],
) -> Result<ResolvedSession, StatusCode> {
    let Some(session) = auth_session::resolve_session_from_headers(state, headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let allowed = session.session.permissions.iter().any(|permission| {
        permission_keys
            .iter()
            .any(|expected| permission == expected)
    });

    if allowed {
        Ok(session)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
