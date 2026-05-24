use axum::{
    Json, Router,
    extract::State,
    http::{
        HeaderValue, Method, StatusCode,
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

#[derive(Debug, Serialize)]
struct LivenessResponse {
    service: &'static str,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ReadinessResponse {
    service: &'static str,
    status: &'static str,
    deployment_target: String,
    environment: String,
    dependencies: Vec<DependencyStatus>,
}

#[derive(Debug, Serialize)]
struct DependencyStatus {
    name: &'static str,
    status: &'static str,
    detail: String,
}

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state.config);

    Router::new()
        .route("/health", get(health))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
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

async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        service: "stloads-backend",
        status: "alive",
    })
}

async fn readiness(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let (is_ready, response) = readiness_report(&state);
    let status = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
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

fn readiness_report(state: &AppState) -> (bool, ReadinessResponse) {
    let mut dependencies = Vec::new();

    dependencies.push(if state.pool.is_some() {
        ready_dependency("database", "database pool is connected")
    } else {
        failed_dependency("database", "database pool is not connected")
    });

    dependencies.push(if state.config.has_required_object_storage_config() {
        ready_dependency(
            "object_storage",
            format!(
                "{} backend has the required object-storage settings",
                state.document_storage.backend()
            ),
        )
    } else {
        failed_dependency(
            "object_storage",
            format!(
                "{} backend is not ready for production document storage",
                state.document_storage.backend()
            ),
        )
    });

    dependencies.push(if state.config.has_required_mail_config() {
        ready_dependency(
            "mail",
            format!("{} mailer is configured", state.email.mode_label()),
        )
    } else {
        failed_dependency(
            "mail",
            format!(
                "{} mailer is not ready for production notifications",
                state.email.mode_label()
            ),
        )
    });

    dependencies.push(
        if state.stripe.is_configured() && state.config.has_required_stripe_config() {
            ready_dependency("stripe", "Stripe secrets and Connect URLs are configured")
        } else {
            failed_dependency("stripe", "Stripe secrets or Connect URLs are missing")
        },
    );

    dependencies.push(if state.config.has_required_worker_config() {
        ready_dependency(
            "workers",
            "TMS shared secret, retry worker, and reconciliation worker are configured",
        )
    } else {
        failed_dependency(
            "workers",
            "TMS shared secret or required worker switches are not configured",
        )
    });

    dependencies.push(match state.config.validate_for_startup() {
        Ok(()) => ready_dependency(
            "runtime_config",
            "runtime configuration passes startup checks",
        ),
        Err(error) => failed_dependency("runtime_config", error),
    });

    let is_ready = dependencies
        .iter()
        .all(|dependency| dependency.status == "ok");

    let response = ReadinessResponse {
        service: "stloads-backend",
        status: if is_ready { "ready" } else { "not_ready" },
        deployment_target: state.config.deployment_target.clone(),
        environment: state.config.environment.clone(),
        dependencies,
    };

    (is_ready, response)
}

fn ready_dependency(name: &'static str, detail: impl Into<String>) -> DependencyStatus {
    DependencyStatus {
        name,
        status: "ok",
        detail: detail.into(),
    }
}

fn failed_dependency(name: &'static str, detail: impl Into<String>) -> DependencyStatus {
    DependencyStatus {
        name,
        status: "failed",
        detail: detail.into(),
    }
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

#[cfg(test)]
mod tests {
    use tokio::sync::broadcast;

    use crate::{
        app::readiness_report, config::RuntimeConfig, document_storage::DocumentStorageService,
        email::EmailService, rate_limit::RateLimiter, realtime_bus::RoutedRealtimeEvent,
        state::AppState, stripe::StripeService,
    };

    #[test]
    fn readiness_fails_when_database_pool_is_missing() {
        let state = test_state(production_config(), false);

        let (is_ready, report) = readiness_report(&state);

        assert!(!is_ready);
        assert_eq!(report.status, "not_ready");
        assert!(
            report.dependencies.iter().any(|dependency| {
                dependency.name == "database" && dependency.status == "failed"
            })
        );
    }

    #[test]
    fn readiness_reports_runtime_config_errors() {
        let mut config = production_config();
        config.cors_allowed_origins.clear();
        let state = test_state(config, false);

        let (is_ready, report) = readiness_report(&state);

        assert!(!is_ready);
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "runtime_config"
                && dependency.status == "failed"
                && dependency.detail.contains("CORS_ALLOWED_ORIGINS")
        }));
    }

    fn test_state(config: RuntimeConfig, include_pool: bool) -> AppState {
        let (realtime_tx, _) = broadcast::channel::<RoutedRealtimeEvent>(32);
        let document_storage = DocumentStorageService::from_config(&config);
        let email = EmailService::from_config(&config);
        let stripe = StripeService::from_config(&config);

        AppState {
            config,
            pool: if include_pool {
                panic!("readiness unit tests do not create live database pools")
            } else {
                None
            },
            document_storage,
            email,
            stripe,
            rate_limiter: RateLimiter::default(),
            realtime_tx,
        }
    }

    fn production_config() -> RuntimeConfig {
        RuntimeConfig {
            bind_addr: "0.0.0.0".into(),
            port: 3001,
            deployment_target: "ibm-code-engine".into(),
            environment: "production".into(),
            public_base_url: Some("https://api.stloads.com".into()),
            cors_allowed_origins: vec!["https://portal.stloads.com".into()],
            run_migrations: false,
            database_url: Some("postgres://stloads:secret@db.stloads.internal:5432/stloads".into()),
            database_schema: Some("public".into()),
            document_storage_backend: "ibm_cos".into(),
            document_storage_root: "./runtime/document-storage".into(),
            object_storage_bucket: Some("stloads-prod-documents".into()),
            object_storage_region: "us-south".into(),
            object_storage_endpoint: Some(
                "https://s3.us-south.cloud-object-storage.appdomain.cloud".into(),
            ),
            object_storage_access_key_id: Some("prod-access-key".into()),
            object_storage_secret_access_key: Some("prod-secret-key".into()),
            object_storage_session_token: None,
            object_storage_force_path_style: true,
            object_storage_prefix: "load-documents".into(),
            stripe_webhook_shared_secret: Some("whsec_platform_secret".into()),
            stripe_webhook_connect_secret: Some("whsec_connect_secret".into()),
            stripe_secret_key: Some("sk_live_real_secret".into()),
            stripe_api_base_url: "https://api.stripe.com/v1".into(),
            stripe_connect_refresh_url: Some(
                "https://portal.stloads.com/settings/payouts/refresh".into(),
            ),
            stripe_connect_return_url: Some(
                "https://portal.stloads.com/settings/payouts/return".into(),
            ),
            stripe_live_transfers_required: true,
            tms_shared_secret: Some("prod-tms-secret".into()),
            tms_reconciliation_worker_enabled: true,
            tms_reconciliation_interval_seconds: 21_600,
            tms_retry_worker_enabled: true,
            tms_retry_interval_seconds: 300,
            tms_retry_batch_size: 10,
            tms_retry_max_attempts: 5,
            tms_stale_handoff_days: 30,
            mail_mailer: "smtp".into(),
            mail_host: Some("smtp.stloads.com".into()),
            mail_port: 587,
            mail_username: Some("mailer".into()),
            mail_password: Some("mail-secret".into()),
            mail_encryption: Some("tls".into()),
            mail_from_address: "noreply@stloads.com".into(),
            mail_from_name: "STLoads".into(),
            mail_fail_open: false,
            mail_outbox_enabled: true,
            mail_outbox_worker_enabled: true,
            mail_outbox_batch_size: 25,
            mail_outbox_retry_interval_seconds: 30,
            mail_outbox_max_attempts: 8,
            portal_url: "https://portal.stloads.com".into(),
            kill_switch_payments: false,
            kill_switch_booking: false,
            kill_switch_tms_pushes: false,
            kill_switch_notifications: false,
            kill_switch_document_uploads: false,
        }
    }
}
