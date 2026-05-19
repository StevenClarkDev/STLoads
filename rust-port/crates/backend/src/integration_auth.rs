use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, TimeZone, Utc};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tracing::warn;

use crate::{config::RuntimeConfig, state::AppState};

type HmacSha256 = Hmac<Sha256>;

const HEADER_AUTHENTICATED: &str = "x-atmp-authenticated";
const HEADER_TENANT: &str = "x-atmp-tenant";
const HEADER_EVENT_ID: &str = "x-atmp-event-id";
const HEADER_CORRELATION_ID: &str = "x-atmp-correlation-id";
const HEADER_IDEMPOTENCY_KEY: &str = "x-atmp-idempotency-key";
const HEADER_TIMESTAMP: &str = "x-atmp-timestamp";
const HEADER_SIGNATURE: &str = "x-atmp-signature";

#[derive(Clone, Default)]
pub struct IntegrationAuthState {
    seen_events: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    rate_buckets: Arc<Mutex<HashMap<String, RateBucket>>>,
}

#[derive(Debug, Clone)]
struct RateBucket {
    window_start: DateTime<Utc>,
    count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedIntegrationRequest {
    pub tenant_id: String,
    pub event_id: String,
    pub correlation_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationAuthRejection {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
struct IntegrationAuthErrorBody<'a> {
    error: &'a str,
    message: &'a str,
}

pub async fn require_atmp_signature(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if !state.config.atmp_integration_require_signature {
        return next.run(request).await;
    }

    let (parts, body) = request.into_parts();
    let bytes = match to_bytes(body, 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(error) => {
            warn!(error = %error, "rejected ATMP integration request with unreadable body");
            return integration_rejection_response(IntegrationAuthRejection {
                status: StatusCode::BAD_REQUEST,
                code: "unreadable_body",
                message: "The integration request body could not be read.".into(),
            });
        }
    };

    match check_integration_request(
        &state.config,
        &state.integration_auth,
        &parts.headers,
        &bytes,
    ) {
        Ok(verified) => {
            let mut request = Request::from_parts(parts, Body::from(bytes));
            request.extensions_mut().insert(verified);
            request
                .headers_mut()
                .insert(HEADER_AUTHENTICATED, HeaderValue::from_static("1"));
            next.run(request).await
        }
        Err(rejection) => {
            warn!(
                status = rejection.status.as_u16(),
                code = rejection.code,
                "rejected ATMP integration request"
            );
            integration_rejection_response(rejection)
        }
    }
}

pub fn check_integration_request(
    config: &RuntimeConfig,
    state: &IntegrationAuthState,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<VerifiedIntegrationRequest, IntegrationAuthRejection> {
    if !config.atmp_integration_require_signature {
        return Ok(VerifiedIntegrationRequest {
            tenant_id: "unsigned-development".into(),
            event_id: "unsigned-development".into(),
            correlation_id: "unsigned-development".into(),
            idempotency_key: "unsigned-development".into(),
        });
    }

    let tenant_id = required_header(headers, HEADER_TENANT)?;
    let event_id = required_header(headers, HEADER_EVENT_ID)?;
    let correlation_id = required_header(headers, HEADER_CORRELATION_ID)?;
    let idempotency_key = required_header(headers, HEADER_IDEMPOTENCY_KEY)?;
    let timestamp = required_header(headers, HEADER_TIMESTAMP)?;
    let signature = required_header(headers, HEADER_SIGNATURE)?;
    let secret = config
        .atmp_integration_shared_secret
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| IntegrationAuthRejection {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "signature_secret_unavailable",
            message: "ATMP integration signature verification is not configured.".into(),
        })?;

    let parsed_timestamp = parse_timestamp(&timestamp)?;
    enforce_replay_window(config, parsed_timestamp)?;
    verify_signature(
        secret,
        &tenant_id,
        &event_id,
        &correlation_id,
        &idempotency_key,
        &timestamp,
        body,
        &signature,
    )?;
    enforce_rate_limit(config, state, &tenant_id)?;
    remember_event(config, state, &tenant_id, &event_id)?;

    Ok(VerifiedIntegrationRequest {
        tenant_id,
        event_id,
        correlation_id,
        idempotency_key,
    })
}

pub fn atmp_authenticated(headers: &HeaderMap) -> bool {
    headers
        .get(HEADER_AUTHENTICATED)
        .and_then(|value| value.to_str().ok())
        == Some("1")
}

pub fn sign_atmp_body_for_test(
    secret: &str,
    tenant_id: &str,
    event_id: &str,
    correlation_id: &str,
    idempotency_key: &str,
    timestamp: &str,
    body: &[u8],
) -> String {
    format!(
        "sha256={}",
        compute_signature_hex(
            secret,
            tenant_id,
            event_id,
            correlation_id,
            idempotency_key,
            timestamp,
            body
        )
    )
}

fn required_header(
    headers: &HeaderMap,
    name: &'static str,
) -> Result<String, IntegrationAuthRejection> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| IntegrationAuthRejection {
            status: StatusCode::BAD_REQUEST,
            code: "missing_header",
            message: format!("Missing required integration header {name}."),
        })
}

fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>, IntegrationAuthRejection> {
    if let Ok(epoch_seconds) = timestamp.parse::<i64>() {
        return Utc
            .timestamp_opt(epoch_seconds, 0)
            .single()
            .ok_or_else(|| invalid_timestamp(timestamp));
    }

    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| invalid_timestamp(timestamp))
}

fn invalid_timestamp(timestamp: &str) -> IntegrationAuthRejection {
    IntegrationAuthRejection {
        status: StatusCode::BAD_REQUEST,
        code: "invalid_timestamp",
        message: format!("Invalid ATMP timestamp {timestamp}."),
    }
}

fn enforce_replay_window(
    config: &RuntimeConfig,
    timestamp: DateTime<Utc>,
) -> Result<(), IntegrationAuthRejection> {
    let now = Utc::now();
    let max_age = chrono::Duration::seconds(config.atmp_integration_replay_window_seconds as i64);
    if timestamp < now - max_age || timestamp > now + chrono::Duration::seconds(30) {
        return Err(IntegrationAuthRejection {
            status: StatusCode::UNAUTHORIZED,
            code: "timestamp_expired",
            message: "The integration timestamp is outside the allowed replay window.".into(),
        });
    }
    Ok(())
}

fn verify_signature(
    secret: &str,
    tenant_id: &str,
    event_id: &str,
    correlation_id: &str,
    idempotency_key: &str,
    timestamp: &str,
    body: &[u8],
    supplied_signature: &str,
) -> Result<(), IntegrationAuthRejection> {
    let expected = compute_signature_hex(
        secret,
        tenant_id,
        event_id,
        correlation_id,
        idempotency_key,
        timestamp,
        body,
    );
    let normalized = supplied_signature
        .trim()
        .strip_prefix("sha256=")
        .or_else(|| supplied_signature.trim().strip_prefix("v1="))
        .unwrap_or_else(|| supplied_signature.trim());

    if normalized.eq_ignore_ascii_case(&expected) {
        Ok(())
    } else {
        Err(IntegrationAuthRejection {
            status: StatusCode::UNAUTHORIZED,
            code: "bad_signature",
            message: "The integration signature did not match the request body.".into(),
        })
    }
}

fn compute_signature_hex(
    secret: &str,
    tenant_id: &str,
    event_id: &str,
    correlation_id: &str,
    idempotency_key: &str,
    timestamp: &str,
    body: &[u8],
) -> String {
    let body_hash = Sha256::digest(body);
    let canonical = format!(
        "{tenant_id}\n{event_id}\n{correlation_id}\n{idempotency_key}\n{timestamp}\n{}",
        hex_encode(&body_hash)
    );
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(canonical.as_bytes());
    hex_encode(&mac.finalize().into_bytes())
}

fn enforce_rate_limit(
    config: &RuntimeConfig,
    state: &IntegrationAuthState,
    tenant_id: &str,
) -> Result<(), IntegrationAuthRejection> {
    let now = Utc::now();
    let mut buckets = state
        .rate_buckets
        .lock()
        .map_err(|_| state_lock_rejection("rate_limit_lock"))?;
    let bucket = buckets.entry(tenant_id.to_string()).or_insert(RateBucket {
        window_start: now,
        count: 0,
    });

    if now - bucket.window_start >= chrono::Duration::seconds(60) {
        bucket.window_start = now;
        bucket.count = 0;
    }

    if bucket.count >= config.atmp_integration_rate_limit_per_minute {
        return Err(IntegrationAuthRejection {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
            message: "The integration route rate limit has been exceeded.".into(),
        });
    }

    bucket.count += 1;
    Ok(())
}

fn remember_event(
    config: &RuntimeConfig,
    state: &IntegrationAuthState,
    tenant_id: &str,
    event_id: &str,
) -> Result<(), IntegrationAuthRejection> {
    let now = Utc::now();
    let max_age = chrono::Duration::seconds(config.atmp_integration_replay_window_seconds as i64);
    let key = format!("{tenant_id}:{event_id}");
    let mut seen = state
        .seen_events
        .lock()
        .map_err(|_| state_lock_rejection("event_replay_lock"))?;
    seen.retain(|_, seen_at| *seen_at >= now - max_age);

    if seen.contains_key(&key) {
        return Err(IntegrationAuthRejection {
            status: StatusCode::CONFLICT,
            code: "event_replay",
            message: "The integration event id has already been accepted.".into(),
        });
    }

    seen.insert(key, now);
    Ok(())
}

fn state_lock_rejection(code: &'static str) -> IntegrationAuthRejection {
    IntegrationAuthRejection {
        status: StatusCode::SERVICE_UNAVAILABLE,
        code,
        message: "Integration authentication state is temporarily unavailable.".into(),
    }
}

fn integration_rejection_response(rejection: IntegrationAuthRejection) -> Response {
    (
        rejection.status,
        axum::Json(IntegrationAuthErrorBody {
            error: rejection.code,
            message: &rejection.message,
        }),
    )
        .into_response()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
