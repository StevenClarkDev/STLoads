use axum::{Json, Router, routing::get};
use domain::tracking::{
    LegEventType, TrackingModuleContract, leg_event_types, tracking_module_contract,
};
use serde::Serialize;
use shared::ApiResponse;

#[derive(Debug, Serialize)]
struct ExecutionOverview {
    contract: TrackingModuleContract,
    leg_event_types: usize,
    trackable_statuses: usize,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/leg-event-types", get(event_types))
}

async fn index() -> Json<ApiResponse<ExecutionOverview>> {
    let contract = tracking_module_contract();
    Json(ApiResponse::ok(ExecutionOverview {
        leg_event_types: leg_event_types().len(),
        trackable_statuses: contract.trackable_status_codes.len(),
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("execution route group ready"))
}

async fn contract() -> Json<ApiResponse<TrackingModuleContract>> {
    Json(ApiResponse::ok(tracking_module_contract()))
}

async fn event_types() -> Json<ApiResponse<Vec<LegEventType>>> {
    Json(ApiResponse::ok(leg_event_types().to_vec()))
}
