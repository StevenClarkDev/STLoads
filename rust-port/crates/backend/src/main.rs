use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "backend=info,tower_http=info".to_string()),
        )
        .init();

    let state = backend::state::AppState::from_env().await;
    let bind_target = format!("{}:{}", state.config.bind_addr, state.config.port);
    let app = backend::app::router(state.clone());

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
