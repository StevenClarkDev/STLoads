use axum::http::{HeaderMap, Method};
use chrono::NaiveDateTime;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use std::time::Duration;

use crate::{rate_limit::RateLimitPolicy, state::AppState};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, FromRow)]
pub struct PartnerApiClient {
    pub id: i64,
    pub organization_id: i64,
    pub actor_user_id: i64,
    pub client_name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub status: String,
    pub rate_limit_per_minute: i32,
    pub require_request_signature: bool,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPartnerClient {
    pub client: PartnerApiClient,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartnerAuthFailure {
    pub message: String,
}

pub async fn resolve_partner_client(
    state: &AppState,
    headers: &HeaderMap,
    method: Method,
    route_path: &str,
    required_scope: &str,
) -> Result<ResolvedPartnerClient, PartnerAuthFailure> {
    let Some(pool) = state.pool.as_ref() else {
        return Err(failure("Partner API auth requires a database connection."));
    };
    let Some(raw_key) = header_value(headers, "x-stloads-api-key") else {
        record_auth_event(
            state,
            None,
            None,
            None,
            headers,
            &method,
            route_path,
            "rejected",
            Some("missing API key"),
            &[required_scope.to_string()],
            true,
            None,
        )
        .await;
        return Err(failure("Partner API key is required."));
    };
    let Some(parsed) = parse_partner_api_key(&raw_key) else {
        record_auth_event(
            state,
            None,
            None,
            None,
            headers,
            &method,
            route_path,
            "rejected",
            Some("malformed API key"),
            &[required_scope.to_string()],
            true,
            None,
        )
        .await;
        return Err(failure("Partner API key format is invalid."));
    };
    let key_hash = sha256_hex(parsed.secret.as_bytes());

    let client = sqlx::query_as::<_, PartnerApiClient>(
        "SELECT id, organization_id, actor_user_id, client_name, key_prefix, scopes, status,
                rate_limit_per_minute, require_request_signature, expires_at
         FROM partner_api_clients
         WHERE key_prefix = $1 AND key_hash = $2
         LIMIT 1",
    )
    .bind(parsed.prefix)
    .bind(&key_hash)
    .fetch_optional(pool)
    .await
    .map_err(|error| failure(format!("Partner API key lookup failed: {}", error)))?;

    let Some(client) = client else {
        record_auth_event(
            state,
            None,
            None,
            Some(parsed.prefix.to_string()),
            headers,
            &method,
            route_path,
            "rejected",
            Some("unknown API key"),
            &[required_scope.to_string()],
            true,
            None,
        )
        .await;
        return Err(failure("Partner API key is not recognized."));
    };

    if client.status != "active"
        || client
            .expires_at
            .map(|expires_at| expires_at <= chrono::Utc::now().naive_utc())
            .unwrap_or(false)
    {
        record_auth_event(
            state,
            Some(client.id),
            Some(client.organization_id),
            Some(client.key_prefix.clone()),
            headers,
            &method,
            route_path,
            "rejected",
            Some("inactive or expired API key"),
            &[required_scope.to_string()],
            client.require_request_signature,
            Some(client.rate_limit_per_minute),
        )
        .await;
        return Err(failure("Partner API key is inactive or expired."));
    }

    if !scope_allows(&client.scopes, required_scope) {
        record_auth_event(
            state,
            Some(client.id),
            Some(client.organization_id),
            Some(client.key_prefix.clone()),
            headers,
            &method,
            route_path,
            "rejected",
            Some("missing required scope"),
            &[required_scope.to_string()],
            client.require_request_signature,
            Some(client.rate_limit_per_minute),
        )
        .await;
        return Err(failure(
            "Partner API key does not include the required scope.",
        ));
    }

    if client.require_request_signature
        && !verify_request_signature(headers, &method, route_path, &raw_key)
    {
        record_auth_event(
            state,
            Some(client.id),
            Some(client.organization_id),
            Some(client.key_prefix.clone()),
            headers,
            &method,
            route_path,
            "rejected",
            Some("missing or invalid request signature"),
            &[required_scope.to_string()],
            client.require_request_signature,
            Some(client.rate_limit_per_minute),
        )
        .await;
        return Err(failure(
            "Partner request signature is missing or invalid for this endpoint.",
        ));
    }

    let rate_decision = state
        .check_rate_limit(
            RateLimitPolicy::new(
                "partner_api",
                client.rate_limit_per_minute.max(1) as u32,
                Duration::from_secs(60),
            ),
            format!("{}:{}", client.organization_id, client.key_prefix),
        )
        .await;
    if !rate_decision.allowed {
        record_auth_event(
            state,
            Some(client.id),
            Some(client.organization_id),
            Some(client.key_prefix.clone()),
            headers,
            &method,
            route_path,
            "rate_limited",
            Some("partner-specific rate limit exceeded"),
            &[required_scope.to_string()],
            client.require_request_signature,
            Some(client.rate_limit_per_minute),
        )
        .await;
        return Err(failure(format!(
            "Partner API rate limit exceeded. Retry in about {} seconds.",
            rate_decision.retry_after_seconds
        )));
    }

    let _ = sqlx::query(
        "UPDATE partner_api_clients
         SET last_used_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(client.id)
    .execute(pool)
    .await;

    record_auth_event(
        state,
        Some(client.id),
        Some(client.organization_id),
        Some(client.key_prefix.clone()),
        headers,
        &method,
        route_path,
        "accepted",
        None,
        &[required_scope.to_string()],
        client.require_request_signature,
        Some(client.rate_limit_per_minute),
    )
    .await;

    Ok(ResolvedPartnerClient { client })
}

pub fn expected_signature(
    api_key: &str,
    method: &Method,
    route_path: &str,
    request_id: &str,
    idempotency_key: &str,
) -> String {
    let canonical = canonical_request(method, route_path, request_id, idempotency_key);
    let mut mac =
        HmacSha256::new_from_slice(api_key.as_bytes()).expect("HMAC accepts arbitrary key length");
    mac.update(canonical.as_bytes());
    format!("v1={}", bytes_to_hex(&mac.finalize().into_bytes()))
}

fn verify_request_signature(
    headers: &HeaderMap,
    method: &Method,
    route_path: &str,
    api_key: &str,
) -> bool {
    let Some(signature) = header_value(headers, "x-stloads-signature") else {
        return false;
    };
    let request_id = header_value(headers, "x-request-id").unwrap_or_default();
    let idempotency_key = header_value(headers, "idempotency-key").unwrap_or_default();
    let expected = expected_signature(api_key, method, route_path, &request_id, &idempotency_key);
    constant_time_eq(signature.as_bytes(), expected.as_bytes())
        || constant_time_eq(
            signature.as_bytes(),
            expected.trim_start_matches("v1=").as_bytes(),
        )
}

fn canonical_request(
    method: &Method,
    route_path: &str,
    request_id: &str,
    idempotency_key: &str,
) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        method.as_str().to_ascii_uppercase(),
        route_path,
        request_id.trim(),
        idempotency_key.trim()
    )
}

fn parse_partner_api_key(raw_key: &str) -> Option<ParsedPartnerApiKey<'_>> {
    let raw_key = raw_key.trim();
    let rest = raw_key.strip_prefix("stlp_")?;
    let (prefix, secret) = rest.split_once('.')?;
    (!prefix.is_empty() && secret.len() >= 24).then_some(ParsedPartnerApiKey { prefix, secret })
}

fn scope_allows(scopes: &[String], required_scope: &str) -> bool {
    scopes
        .iter()
        .any(|scope| scope == "*" || scope == required_scope)
}

fn header_value(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    bytes_to_hex(&hasher.finalize())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0_u8, |acc, (left, right)| acc | (left ^ right))
        == 0
}

fn failure(message: impl Into<String>) -> PartnerAuthFailure {
    PartnerAuthFailure {
        message: message.into(),
    }
}

#[allow(clippy::too_many_arguments)]
async fn record_auth_event(
    state: &AppState,
    client_id: Option<i64>,
    organization_id: Option<i64>,
    key_prefix: Option<String>,
    headers: &HeaderMap,
    method: &Method,
    route_path: &str,
    auth_result: &str,
    failure_reason: Option<&str>,
    scopes_checked: &[String],
    signature_required: bool,
    rate_limit_per_minute: Option<i32>,
) {
    let Some(pool) = state.pool.as_ref() else {
        return;
    };
    let _ = sqlx::query(
        "INSERT INTO partner_api_auth_events (
             partner_api_client_id, organization_id, key_prefix, request_id, route_path, method,
             auth_result, failure_reason, scopes_checked, signature_required,
             rate_limit_per_minute, created_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP)",
    )
    .bind(client_id)
    .bind(organization_id)
    .bind(key_prefix)
    .bind(header_value(headers, "x-request-id"))
    .bind(route_path)
    .bind(method.as_str())
    .bind(auth_result)
    .bind(failure_reason)
    .bind(scopes_checked.to_vec())
    .bind(signature_required)
    .bind(rate_limit_per_minute)
    .execute(pool)
    .await;
}

struct ParsedPartnerApiKey<'a> {
    prefix: &'a str,
    secret: &'a str,
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, Method};

    use super::{expected_signature, verify_request_signature};

    #[test]
    fn partner_signature_uses_method_route_request_and_idempotency() {
        let api_key = "stlp_demo.abcdefghijklmnopqrstuvwxyz123456";
        let request_id = "req_partner_1";
        let idempotency_key = "idem_partner_1";
        let signature = expected_signature(
            api_key,
            &Method::POST,
            "/dispatch/loads/api-post",
            request_id,
            idempotency_key,
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("req_partner_1"));
        headers.insert(
            "idempotency-key",
            HeaderValue::from_static("idem_partner_1"),
        );
        headers.insert(
            "x-stloads-signature",
            HeaderValue::from_str(&signature).unwrap(),
        );

        assert!(verify_request_signature(
            &headers,
            &Method::POST,
            "/dispatch/loads/api-post",
            api_key
        ));

        assert!(!verify_request_signature(
            &headers,
            &Method::POST,
            "/dispatch/loads/other",
            api_key
        ));
    }
}
