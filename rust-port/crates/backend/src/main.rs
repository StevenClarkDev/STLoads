#![recursion_limit = "256"]

use tracing::info;
use tracing_subscriber::EnvFilter;

mod api_contract;
mod app;
mod auth_session;
mod config;
mod document_storage;
mod document_validation;
mod email;
mod partner_auth;
mod rate_limit;
mod realtime_bus;
mod routes;
mod screen_data;
mod state;
mod stripe;
#[cfg(test)]
mod test_support;
mod tms_scheduler;

#[tokio::main]
async fn main() {
    let config = config::RuntimeConfig::from_env();
    init_tracing(&config);
    let state = state::AppState::from_config(config).await;

    if !state.config.should_start_web() {
        info!(
            runtime_mode = %state.config.runtime_mode,
            deployment_target = %state.config.deployment_target,
            environment = %state.config.environment,
            otel_configured = state.config.otel_exporter_endpoint.is_some(),
            "backend worker runtime started without HTTP listener"
        );
        tokio::signal::ctrl_c()
            .await
            .expect("wait for worker shutdown signal");
        info!("backend worker runtime shutdown signal received");
        return;
    }

    let bind_target = format!("{}:{}", state.config.bind_addr, state.config.port);
    let app = app::router(state.clone());

    let listener = tokio::net::TcpListener::bind(&bind_target)
        .await
        .expect("bind backend listener");

    info!(
        bind_target = %bind_target,
        deployment_target = %state.config.deployment_target,
        environment = %state.config.environment,
        database_state = %state.database_state(),
        "backend listening"
    );

    axum::serve(listener, app).await.expect("serve backend");
}

fn init_tracing(config: &config::RuntimeConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("backend=info,tower_http=info"));

    if config.should_emit_json_logs() {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }
}
