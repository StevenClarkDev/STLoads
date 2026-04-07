use axum::{Json, Router, extract::State, routing::get};
use db::inventory;
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::{routes, state::AppState};

#[derive(Debug, Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
    deployment_target: String,
    environment: String,
    public_base_url: Option<String>,
    database_state: &'static str,
    tracked_tables: usize,
    drift_notes: usize,
    route_groups: Vec<&'static str>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/auth", routes::auth::router())
        .nest("/dispatch", routes::dispatch::router())
        .nest("/marketplace", routes::marketplace::router())
        .nest("/execution", routes::execution::router())
        .nest("/payments", routes::payments::router())
        .nest("/tms", routes::tms::router())
        .nest("/api/stloads", routes::tms::integration_router())
        .nest("/admin", routes::admin::router())
        .nest("/master-data", routes::master_data::router())
        .nest("/realtime", routes::realtime::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let deployment_target = state.config.deployment_target.clone();
    let environment = state.config.environment.clone();
    let public_base_url = state.config.public_base_url.clone();

    Json(HealthResponse {
        service: "stloads-backend",
        status: "ok",
        deployment_target,
        environment,
        public_base_url,
        database_state: state.database_state(),
        tracked_tables: inventory::tracked_table_count(),
        drift_notes: inventory::DRIFT_NOTES.len(),
        route_groups: routes::GROUP_NAMES.to_vec(),
    })
}
