use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{
        HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use db::inventory;
use serde::Serialize;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use uuid::Uuid;

use crate::{config::RuntimeConfig, integration_auth, routes, state::AppState};

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
pub struct ReadinessResponse {
    service: &'static str,
    status: &'static str,
    checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadinessCheck {
    pub key: &'static str,
    pub label: &'static str,
    pub status: &'static str,
    pub detail: String,
}

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state.config);
    let signed_tms_routes = routes::tms::integration_router().route_layer(
        middleware::from_fn_with_state(state.clone(), integration_auth::require_atmp_signature),
    );

    Router::new()
        .route("/health", get(health))
        .route("/readiness", get(readiness))
        .nest("/auth", routes::auth::router())
        .nest("/dispatch", routes::dispatch::router())
        .nest("/marketplace", routes::marketplace::router())
        .nest("/execution", routes::execution::router())
        .nest("/payments", routes::payments::router())
        .nest(
            "/tms",
            routes::tms::metadata_router().merge(signed_tms_routes.clone()),
        )
        .nest("/api/stloads", signed_tms_routes)
        .nest("/admin", routes::admin::router())
        .nest("/master-data", routes::master_data::router())
        .nest("/realtime", routes::realtime::router())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            security_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            observability_middleware,
        ))
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

async fn readiness(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let checks = readiness_checks(&state);
    let ready = checks.iter().all(|check| check.status == "ok");
    let status = if ready { "ready" } else { "degraded" };
    let http_status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        http_status,
        Json(ReadinessResponse {
            service: "stloads-backend",
            status,
            checks,
        }),
    )
}

pub fn readiness_checks(state: &AppState) -> Vec<ReadinessCheck> {
    let config = &state.config;
    vec![
        ReadinessCheck {
            key: "database",
            label: "Database",
            status: if state.pool.is_some() {
                "ok"
            } else {
                "degraded"
            },
            detail: state.database_state().to_string(),
        },
        ReadinessCheck {
            key: "object_storage",
            label: "Object Storage",
            status: if state.document_storage.is_ready_for_environment(config) {
                "ok"
            } else {
                "degraded"
            },
            detail: state.document_storage.readiness_detail(config),
        },
        ReadinessCheck {
            key: "stripe",
            label: "Stripe",
            status: if state.stripe.is_configured() || !config.is_production() {
                "ok"
            } else {
                "degraded"
            },
            detail: if state.stripe.is_configured() {
                "configured".into()
            } else if config.is_production() {
                "missing STRIPE_SECRET_KEY".into()
            } else {
                "not required outside production".into()
            },
        },
        ReadinessCheck {
            key: "atmp_outbound",
            label: "ATMP Outbound",
            status: if config.atmp_outbound_base_url.is_some() || !config.is_production() {
                "ok"
            } else {
                "degraded"
            },
            detail: config
                .atmp_outbound_base_url
                .clone()
                .unwrap_or_else(|| "not configured".into()),
        },
        ReadinessCheck {
            key: "queue_health",
            label: "Queue Health",
            status: if state.pool.is_some() || !config.is_production() {
                "ok"
            } else {
                "degraded"
            },
            detail: format!(
                "mail_outbox={}, tms_retry={}, tms_reconciliation={}",
                state.email.outbox_label(),
                enabled_label(config.tms_retry_worker_enabled),
                enabled_label(config.tms_reconciliation_worker_enabled)
            ),
        },
        ReadinessCheck {
            key: "realtime",
            label: "Realtime",
            status: "ok",
            detail: format!("broadcast receivers={}", state.realtime_receiver_count()),
        },
    ]
}

async fn observability_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let correlation_id = header_value(request.headers(), "x-atmp-correlation-id")
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let tenant_id = header_value(request.headers(), "x-atmp-tenant");
    let event_id = header_value(request.headers(), "x-atmp-event-id");
    let idempotency_key = header_value(request.headers(), "x-atmp-idempotency-key");
    let atmp_load_id = header_value(request.headers(), "x-atmp-load-id");
    let posting_id = header_value(request.headers(), "x-stloads-posting-id");
    let actor_id = header_value(request.headers(), "x-stloads-actor-id");
    let method = request.method().clone();
    let route = request.uri().path().to_string();
    let audit_category = audit_category_for_route(&route);

    let response = next.run(request).await;
    let status = response.status().as_u16();

    info!(
        tenant_id = tenant_id.as_deref().unwrap_or("unknown"),
        actor_id = actor_id.as_deref().unwrap_or("unknown"),
        route = %route,
        method = %method,
        status = status,
        correlation_id = %correlation_id,
        event_id = event_id.as_deref().unwrap_or("none"),
        atmp_load_id = atmp_load_id.as_deref().unwrap_or("none"),
        posting_id = posting_id.as_deref().unwrap_or("none"),
        idempotency_key = idempotency_key.as_deref().unwrap_or("none"),
        audit_category,
        immutable = true,
        "stloads request audit event"
    );

    state.record_audit_event(
        audit_category,
        &route,
        tenant_id.as_deref(),
        actor_id.as_deref(),
        Some(&correlation_id),
        event_id.as_deref(),
        atmp_load_id.as_deref(),
        posting_id.as_deref(),
        idempotency_key.as_deref(),
    );

    response
}

async fn security_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let route = request.uri().path().to_string();
    let method = request.method().clone();
    let headers = request.headers();
    let class = security_class_for_route(&route);
    let rate_key = rate_limit_key(headers, class);

    if let Err(message) = state.enforce_rate_limit(class, &rate_key, rate_limit_per_minute(class)) {
        return with_security_headers((StatusCode::TOO_MANY_REQUESTS, message).into_response());
    }

    if let Some(content_length) = headers
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
    {
        let max_body_bytes = max_body_bytes_for_class(class);
        if content_length > max_body_bytes {
            return with_security_headers(
                (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("Request body exceeds the {} byte limit.", max_body_bytes),
                )
                    .into_response(),
            );
        }
    }

    if let Err(message) = validate_cookie_origin_strategy(&state.config, &method, headers) {
        return with_security_headers((StatusCode::FORBIDDEN, message).into_response());
    }

    with_security_headers(next.run(request).await)
}

fn with_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    let header_values = [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "strict-origin-when-cross-origin"),
        ("cross-origin-opener-policy", "same-origin"),
        ("cross-origin-resource-policy", "same-site"),
        (
            "permissions-policy",
            "camera=(), microphone=(), geolocation=(self), payment=(self)",
        ),
        (
            "content-security-policy",
            "default-src 'self'; connect-src 'self' https: wss:; img-src 'self' data: https:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'",
        ),
    ];

    for (name, value) in header_values {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_lowercase(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            headers.insert(name, value);
        }
    }

    response
}

fn security_class_for_route(route: &str) -> &'static str {
    match route {
        path if path == "/auth/login" => "login",
        path if path.contains("/otp") || path.contains("password") => "otp",
        path if path.contains("/book") => "booking",
        path if path.contains("/offers") => "offer",
        path if path.contains("saved-search")
            || path.contains("search")
            || path.contains("/load-board") =>
        {
            "search"
        }
        path if path.contains("upload") || path.contains("documents") => "upload",
        path if path.contains("webhook")
            || path.contains("/api/stloads")
            || path.contains("/tms/") =>
        {
            "webhook"
        }
        path if path.contains("/admin/")
            && (path.contains("replay") || path.contains("resolve")) =>
        {
            "admin_replay"
        }
        _ => "default",
    }
}

fn rate_limit_per_minute(class: &str) -> u32 {
    match class {
        "login" | "otp" => 10,
        "search" => 120,
        "offer" => 60,
        "booking" => 30,
        "upload" => 20,
        "webhook" => 120,
        "admin_replay" => 20,
        _ => 600,
    }
}

fn max_body_bytes_for_class(class: &str) -> u64 {
    match class {
        "upload" => 25 * 1024 * 1024,
        "webhook" => 2 * 1024 * 1024,
        "offer" | "booking" | "admin_replay" => 512 * 1024,
        "login" | "otp" => 64 * 1024,
        _ => 1024 * 1024,
    }
}

fn rate_limit_key(headers: &HeaderMap, class: &str) -> String {
    let client = header_value(headers, "x-forwarded-for")
        .and_then(|value| value.split(',').next().map(str::trim).map(str::to_string))
        .filter(|value| !value.is_empty())
        .or_else(|| header_value(headers, "x-real-ip"))
        .unwrap_or_else(|| "anonymous".into());
    format!("{class}:{client}")
}

fn validate_cookie_origin_strategy(
    config: &RuntimeConfig,
    method: &Method,
    headers: &HeaderMap,
) -> Result<(), String> {
    if matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(());
    }

    if headers.get("cookie").is_none() {
        return Ok(());
    }

    let Some(origin) = header_value(headers, "origin") else {
        return Err("Cookie-authenticated unsafe requests require an Origin header.".into());
    };

    if config
        .cors_allowed_origins
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(&origin))
    {
        Ok(())
    } else {
        Err("Cookie-authenticated unsafe request origin is not allowed.".into())
    }
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn audit_category_for_route(route: &str) -> &'static str {
    match route {
        path if path.contains("/auth/") => "auth",
        path if path.contains("/book") => "booking",
        path if path.contains("/offers") => "offer",
        path if path.contains("compliance") => "compliance",
        path if path.contains("document") || path.contains("kyc") => "document",
        path if path.contains("/payments") || path.contains("stripe") => "payment",
        path if path.contains("/tms") || path.contains("/api/stloads") => "integration",
        path if path.contains("impersonation") => "impersonation",
        path if path.contains("/load-board") || path.contains("/loads") => "listing",
        path if path.contains("/admin") => "admin",
        _ => "system",
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::broadcast;

    use super::*;
    use crate::{
        document_storage::DocumentStorageService, email::EmailService,
        integration_auth::IntegrationAuthState, realtime_bus::RoutedRealtimeEvent,
        stripe::StripeService,
    };

    fn test_config(environment: &str) -> RuntimeConfig {
        RuntimeConfig {
            bind_addr: "127.0.0.1".into(),
            port: 3001,
            deployment_target: "backend-test".into(),
            environment: environment.into(),
            public_base_url: Some("https://backend.stloads.test".into()),
            cors_allowed_origins: vec!["https://portal.stloads.test".into()],
            run_migrations: false,
            database_url: None,
            database_schema: None,
            document_storage_backend: "local".into(),
            document_storage_root: "./runtime/document-storage".into(),
            object_storage_bucket: None,
            object_storage_region: "us-south".into(),
            object_storage_endpoint: None,
            object_storage_access_key_id: None,
            object_storage_secret_access_key: None,
            object_storage_session_token: None,
            object_storage_force_path_style: false,
            object_storage_prefix: "tests".into(),
            stripe_webhook_shared_secret: None,
            stripe_webhook_connect_secret: None,
            stripe_secret_key: None,
            stripe_api_base_url: "https://api.stripe.com/v1".into(),
            stripe_connect_refresh_url: None,
            stripe_connect_return_url: None,
            stripe_live_transfers_required: false,
            atmp_outbound_base_url: None,
            atmp_integration_shared_secret: None,
            atmp_integration_require_signature: false,
            atmp_integration_replay_window_seconds: 300,
            atmp_integration_rate_limit_per_minute: 120,
            atmp_outbound_worker_enabled: false,
            atmp_outbound_interval_seconds: 30,
            atmp_outbound_batch_size: 25,
            atmp_outbound_max_attempts: 8,
            tms_shared_secret: None,
            tms_reconciliation_worker_enabled: false,
            tms_reconciliation_interval_seconds: 21_600,
            tms_retry_worker_enabled: false,
            tms_retry_interval_seconds: 300,
            tms_retry_batch_size: 10,
            tms_retry_max_attempts: 5,
            tms_stale_handoff_days: 30,
            mail_mailer: "log".into(),
            mail_host: None,
            mail_port: 587,
            mail_username: None,
            mail_password: None,
            mail_encryption: None,
            mail_from_address: "noreply@stloads.test".into(),
            mail_from_name: "STLoads Tests".into(),
            mail_fail_open: true,
            mail_outbox_enabled: false,
            mail_outbox_worker_enabled: false,
            mail_outbox_batch_size: 25,
            mail_outbox_retry_interval_seconds: 30,
            mail_outbox_max_attempts: 8,
            portal_url: "https://portal.stloads.test".into(),
        }
    }

    fn test_state(config: RuntimeConfig) -> AppState {
        let (realtime_tx, _) = broadcast::channel::<RoutedRealtimeEvent>(16);
        AppState {
            document_storage: DocumentStorageService::from_config(&config),
            email: EmailService::from_config_with_pool(&config, None),
            stripe: StripeService::from_config(&config),
            config,
            pool: None,
            integration_auth: IntegrationAuthState::default(),
            realtime_tx,
            security: crate::state::SecurityState::default(),
        }
    }

    #[test]
    fn readiness_degrades_when_production_dependencies_are_missing() {
        let state = test_state(test_config("production"));
        let checks = readiness_checks(&state);

        assert!(
            checks
                .iter()
                .any(|check| check.key == "database" && check.status == "degraded")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.key == "object_storage" && check.status == "degraded")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.key == "stripe" && check.status == "degraded")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.key == "atmp_outbound" && check.status == "degraded")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.key == "realtime" && check.status == "ok")
        );
    }

    #[test]
    fn readiness_allows_local_services_outside_production() {
        let state = test_state(test_config("development"));
        let checks = readiness_checks(&state);

        assert!(
            checks
                .iter()
                .any(|check| check.key == "object_storage" && check.status == "ok")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.key == "stripe" && check.status == "ok")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.key == "atmp_outbound" && check.status == "ok")
        );
    }

    #[test]
    fn audit_category_mapping_covers_production_workflows() {
        let routes = [
            ("/auth/login", "auth"),
            ("/dispatch/load-board", "listing"),
            ("/marketplace/postings/4/offers", "offer"),
            ("/dispatch/load-board/4/book", "booking"),
            ("/admin/compliance/overrides", "compliance"),
            ("/dispatch/loads/9/documents", "document"),
            ("/payments/webhooks/stripe", "payment"),
            ("/api/stloads/push", "integration"),
            ("/admin/support/impersonations", "impersonation"),
            ("/admin/users", "admin"),
        ];

        for (route, expected) in routes {
            assert_eq!(audit_category_for_route(route), expected);
        }
    }

    #[test]
    fn security_class_limits_cover_p19_route_groups() {
        let routes = [
            ("/auth/login", "login", 10, 64 * 1024),
            ("/auth/otp/resend", "otp", 10, 64 * 1024),
            ("/dispatch/load-board", "search", 120, 1024 * 1024),
            ("/marketplace/postings/4/offers", "offer", 60, 512 * 1024),
            ("/dispatch/load-board/4/book", "booking", 30, 512 * 1024),
            (
                "/dispatch/loads/4/documents/upload",
                "upload",
                20,
                25 * 1024 * 1024,
            ),
            ("/tms/webhook/status", "webhook", 120, 2 * 1024 * 1024),
            (
                "/admin/stloads/outbound-events/4/replay",
                "admin_replay",
                20,
                512 * 1024,
            ),
        ];

        for (route, class, expected_limit, expected_body_limit) in routes {
            assert_eq!(security_class_for_route(route), class);
            assert_eq!(rate_limit_per_minute(class), expected_limit);
            assert_eq!(max_body_bytes_for_class(class), expected_body_limit);
        }
    }

    #[test]
    fn cookie_unsafe_requests_require_allowed_origin() {
        let config = test_config("production");
        let mut headers = HeaderMap::new();
        headers.insert("cookie", HeaderValue::from_static("sid=test"));

        assert!(validate_cookie_origin_strategy(&config, &Method::POST, &headers).is_err());

        headers.insert("origin", HeaderValue::from_static("https://evil.test"));
        assert!(validate_cookie_origin_strategy(&config, &Method::POST, &headers).is_err());

        headers.insert(
            "origin",
            HeaderValue::from_static("https://portal.stloads.test"),
        );
        assert!(validate_cookie_origin_strategy(&config, &Method::POST, &headers).is_ok());
    }

    #[test]
    fn security_headers_are_attached_to_responses() {
        let response = with_security_headers(StatusCode::OK.into_response());
        let headers = response.headers();

        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert!(headers.get("content-security-policy").is_some());
    }

    #[test]
    fn auth_failure_lockout_triggers_after_repeated_failures() {
        let state = test_state(test_config("development"));
        let email = "locked@example.test";

        for _ in 0..4 {
            state.record_auth_failure(email);
            assert!(state.auth_failure_locked_message(email).is_none());
        }

        state.record_auth_failure(email);
        assert!(state.auth_failure_locked_message(email).is_some());

        state.clear_auth_failures(email);
        assert!(state.auth_failure_locked_message(email).is_none());
    }
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

    let integration_headers = [
        "x-atmp-tenant",
        "x-atmp-event-id",
        "x-atmp-correlation-id",
        "x-atmp-idempotency-key",
        "x-atmp-timestamp",
        "x-atmp-signature",
    ]
    .into_iter()
    .filter_map(|name| HeaderName::from_lowercase(name.as_bytes()).ok())
    .collect::<Vec<_>>();

    cors = cors.allow_headers(
        [AUTHORIZATION, CONTENT_TYPE, ACCEPT]
            .into_iter()
            .chain(integration_headers)
            .collect::<Vec<_>>(),
    );

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
