use axum::{
    Json, Router,
    extract::State,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use db::inventory;
use serde::Serialize;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{config::RuntimeConfig, routes, state::AppState};

#[derive(Debug, Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
    deployment_target: String,
    environment: String,
    public_base_url: Option<String>,
    database_state: &'static str,
    mailer_mode: &'static str,
    mail_outbox: &'static str,
    tms_retry_worker: &'static str,
    tms_reconciliation_worker: &'static str,
    tracked_tables: usize,
    drift_notes: usize,
    route_groups: Vec<&'static str>,
}

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state.config);

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
        .layer(cors)
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
        mailer_mode: state.email.mode_label(),
        mail_outbox: state.email.outbox_label(),
        tms_retry_worker: enabled_label(state.config.tms_retry_worker_enabled),
        tms_reconciliation_worker: enabled_label(state.config.tms_reconciliation_worker_enabled),
        tracked_tables: inventory::tracked_table_count(),
        drift_notes: inventory::DRIFT_NOTES.len(),
        route_groups: routes::GROUP_NAMES.to_vec(),
    })
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled { "enabled" } else { "disabled" }
}

fn cors_layer(config: &RuntimeConfig) -> CorsLayer {
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]);

    let origins = config
        .cors_allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect::<Vec<_>>();

    if origins.is_empty() {
        cors = cors.allow_origin(Any);
    } else {
        cors = cors.allow_origin(origins);
    }

    cors
}
