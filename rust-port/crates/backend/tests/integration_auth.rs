use axum::http::{HeaderMap, HeaderValue, StatusCode};
use backend::{
    config::RuntimeConfig,
    integration_auth::{IntegrationAuthState, check_integration_request, sign_atmp_body_for_test},
};
use chrono::{Duration, Utc};

fn config() -> RuntimeConfig {
    RuntimeConfig {
        bind_addr: "127.0.0.1".into(),
        port: 3001,
        deployment_target: "backend-test".into(),
        environment: "production".into(),
        public_base_url: Some("https://backend.stloads.test".into()),
        cors_allowed_origins: vec!["https://portal.stloads.test".into()],
        run_migrations: false,
        database_url: Some("postgres://stloads:secret@db.example/stloads".into()),
        database_schema: None,
        document_storage_backend: "ibm_cos".into(),
        document_storage_root: "./runtime/document-storage".into(),
        object_storage_bucket: Some("stloads-documents".into()),
        object_storage_region: "us-south".into(),
        object_storage_endpoint: Some(
            "https://s3.us-south.cloud-object-storage.appdomain.cloud".into(),
        ),
        object_storage_access_key_id: Some("access".into()),
        object_storage_secret_access_key: Some("secret".into()),
        object_storage_session_token: None,
        object_storage_force_path_style: false,
        object_storage_prefix: "tests".into(),
        stripe_webhook_shared_secret: Some("whsec_test".into()),
        stripe_webhook_connect_secret: None,
        stripe_secret_key: Some("sk_test".into()),
        stripe_api_base_url: "https://api.stripe.com/v1".into(),
        stripe_connect_refresh_url: None,
        stripe_connect_return_url: None,
        stripe_live_transfers_required: false,
        atmp_outbound_base_url: Some("https://dispatch-api.example.test".into()),
        atmp_integration_shared_secret: Some("integration-secret".into()),
        atmp_integration_require_signature: true,
        atmp_integration_replay_window_seconds: 300,
        atmp_integration_rate_limit_per_minute: 2,
        tms_shared_secret: Some("integration-secret".into()),
        tms_reconciliation_worker_enabled: false,
        tms_reconciliation_interval_seconds: 21_600,
        tms_retry_worker_enabled: false,
        tms_retry_interval_seconds: 300,
        tms_retry_batch_size: 10,
        tms_retry_max_attempts: 5,
        tms_stale_handoff_days: 30,
        mail_mailer: "smtp".into(),
        mail_host: Some("smtp.example.test".into()),
        mail_port: 587,
        mail_username: Some("mailer".into()),
        mail_password: Some("secret".into()),
        mail_encryption: Some("tls".into()),
        mail_from_address: "noreply@stloads.test".into(),
        mail_from_name: "STLoads Tests".into(),
        mail_fail_open: false,
        mail_outbox_enabled: false,
        mail_outbox_worker_enabled: false,
        mail_outbox_batch_size: 25,
        mail_outbox_retry_interval_seconds: 30,
        mail_outbox_max_attempts: 8,
        portal_url: "https://portal.stloads.test".into(),
    }
}

fn signed_headers(tenant: &str, event_id: &str, timestamp: &str, body: &[u8]) -> HeaderMap {
    let config = config();
    let mut headers = HeaderMap::new();
    headers.insert("x-atmp-tenant", HeaderValue::from_str(tenant).unwrap());
    headers.insert("x-atmp-event-id", HeaderValue::from_str(event_id).unwrap());
    headers.insert("x-atmp-correlation-id", HeaderValue::from_static("corr-p5"));
    headers.insert(
        "x-atmp-idempotency-key",
        HeaderValue::from_static("idem-p5"),
    );
    headers.insert(
        "x-atmp-timestamp",
        HeaderValue::from_str(timestamp).unwrap(),
    );
    headers.insert(
        "x-atmp-signature",
        HeaderValue::from_str(&sign_atmp_body_for_test(
            config.atmp_integration_shared_secret.as_deref().unwrap(),
            tenant,
            event_id,
            "corr-p5",
            "idem-p5",
            timestamp,
            body,
        ))
        .unwrap(),
    );
    headers
}

#[test]
fn signed_atmp_request_is_accepted() {
    let config = config();
    let state = IntegrationAuthState::default();
    let body = br#"{"contract_version":"2026-05-01","action":"publish"}"#;
    let timestamp = Utc::now().to_rfc3339();
    let headers = signed_headers("tenant-p5", "evt-p5-valid", &timestamp, body);

    let verified = check_integration_request(&config, &state, &headers, body).unwrap();

    assert_eq!(verified.tenant_id, "tenant-p5");
    assert_eq!(verified.event_id, "evt-p5-valid");
}

#[test]
fn bad_signature_is_rejected() {
    let config = config();
    let state = IntegrationAuthState::default();
    let body = br#"{"ok":true}"#;
    let timestamp = Utc::now().to_rfc3339();
    let mut headers = signed_headers("tenant-p5", "evt-p5-bad-sig", &timestamp, body);
    headers.insert("x-atmp-signature", HeaderValue::from_static("sha256=bad"));

    let rejection = check_integration_request(&config, &state, &headers, body).unwrap_err();

    assert_eq!(rejection.status, StatusCode::UNAUTHORIZED);
    assert_eq!(rejection.code, "bad_signature");
}

#[test]
fn expired_timestamp_is_rejected() {
    let config = config();
    let state = IntegrationAuthState::default();
    let body = br#"{"ok":true}"#;
    let timestamp = (Utc::now() - Duration::seconds(600)).to_rfc3339();
    let headers = signed_headers("tenant-p5", "evt-p5-expired", &timestamp, body);

    let rejection = check_integration_request(&config, &state, &headers, body).unwrap_err();

    assert_eq!(rejection.status, StatusCode::UNAUTHORIZED);
    assert_eq!(rejection.code, "timestamp_expired");
}

#[test]
fn reused_event_id_is_rejected() {
    let config = config();
    let state = IntegrationAuthState::default();
    let body = br#"{"ok":true}"#;
    let timestamp = Utc::now().to_rfc3339();
    let headers = signed_headers("tenant-p5", "evt-p5-replay", &timestamp, body);

    check_integration_request(&config, &state, &headers, body).unwrap();
    let rejection = check_integration_request(&config, &state, &headers, body).unwrap_err();

    assert_eq!(rejection.status, StatusCode::CONFLICT);
    assert_eq!(rejection.code, "event_replay");
}

#[test]
fn missing_tenant_is_rejected() {
    let config = config();
    let state = IntegrationAuthState::default();
    let body = br#"{"ok":true}"#;
    let timestamp = Utc::now().to_rfc3339();
    let mut headers = signed_headers("tenant-p5", "evt-p5-missing-tenant", &timestamp, body);
    headers.remove("x-atmp-tenant");

    let rejection = check_integration_request(&config, &state, &headers, body).unwrap_err();

    assert_eq!(rejection.status, StatusCode::BAD_REQUEST);
    assert_eq!(rejection.code, "missing_header");
}

#[test]
fn rate_limit_is_enforced_per_tenant() {
    let config = config();
    let state = IntegrationAuthState::default();
    let body = br#"{"ok":true}"#;
    let timestamp = Utc::now().to_rfc3339();

    for event_id in ["evt-p5-rate-1", "evt-p5-rate-2"] {
        let headers = signed_headers("tenant-p5", event_id, &timestamp, body);
        check_integration_request(&config, &state, &headers, body).unwrap();
    }

    let headers = signed_headers("tenant-p5", "evt-p5-rate-3", &timestamp, body);
    let rejection = check_integration_request(&config, &state, &headers, body).unwrap_err();

    assert_eq!(rejection.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(rejection.code, "rate_limited");
}
