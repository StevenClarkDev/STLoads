use serde_json::Value;
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct ExternalEventDedupeRecord {
    pub id: i64,
    pub processing_status: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct WebhookDeliveryLogRecord {
    pub id: i64,
    pub organization_id: i64,
    pub endpoint_id: Option<i64>,
    pub event_type: String,
    pub event_id: String,
    pub delivery_status: String,
    pub attempt_count: i32,
    pub next_retry_at: Option<chrono::NaiveDateTime>,
    pub last_attempt_at: Option<chrono::NaiveDateTime>,
    pub response_status_code: Option<i32>,
    pub response_latency_ms: Option<i32>,
    pub response_body_excerpt: Option<String>,
    pub dead_letter_reason: Option<String>,
    pub replay_of_delivery_id: Option<i64>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct EnqueueWebhookDeliveryParams<'a> {
    pub organization_id: i64,
    pub endpoint_id: Option<i64>,
    pub event_type: &'a str,
    pub event_id: &'a str,
    pub request_payload: Value,
    pub request_headers: Value,
}

pub async fn claim_external_event(
    pool: &DbPool,
    organization_id: Option<i64>,
    source_system: &str,
    event_type: &str,
    external_event_id: &str,
    request_id: Option<&str>,
    payload: Option<&Value>,
) -> Result<bool, sqlx::Error> {
    let inserted = sqlx::query_as::<_, ExternalEventDedupeRecord>(
        "INSERT INTO external_event_dedupe_records (
             organization_id, source_system, event_type, external_event_id, request_id,
             processing_status, payload, first_seen_at, last_seen_at
         )
         VALUES ($1, $2, $3, $4, $5, 'processing', $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (source_system, event_type, external_event_id)
         DO NOTHING
         RETURNING id, processing_status",
    )
    .bind(organization_id)
    .bind(source_system)
    .bind(event_type)
    .bind(external_event_id)
    .bind(request_id)
    .bind(payload.cloned())
    .fetch_optional(pool)
    .await?;

    if inserted.is_some() {
        return Ok(true);
    }

    sqlx::query(
        "UPDATE external_event_dedupe_records
         SET last_seen_at = CURRENT_TIMESTAMP,
             processing_status = CASE
                 WHEN processing_status = 'completed' THEN 'ignored_duplicate'
                 ELSE processing_status
             END
         WHERE source_system = $1
           AND event_type = $2
           AND external_event_id = $3",
    )
    .bind(source_system)
    .bind(event_type)
    .bind(external_event_id)
    .execute(pool)
    .await?;

    Ok(false)
}

pub async fn complete_external_event(
    pool: &DbPool,
    _organization_id: Option<i64>,
    source_system: &str,
    event_type: &str,
    external_event_id: &str,
    result_summary: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE external_event_dedupe_records
         SET processing_status = 'completed',
             completed_at = CURRENT_TIMESTAMP,
             last_seen_at = CURRENT_TIMESTAMP,
             result_summary = $4
         WHERE source_system = $1
           AND event_type = $2
           AND external_event_id = $3",
    )
    .bind(source_system)
    .bind(event_type)
    .bind(external_event_id)
    .bind(result_summary)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn enqueue_webhook_delivery(
    pool: &DbPool,
    params: EnqueueWebhookDeliveryParams<'_>,
) -> Result<WebhookDeliveryLogRecord, sqlx::Error> {
    sqlx::query_as::<_, WebhookDeliveryLogRecord>(
        "INSERT INTO webhook_delivery_logs (
             organization_id, endpoint_id, event_type, event_id, delivery_status,
             request_payload, request_headers, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, 'queued', $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, organization_id, endpoint_id, event_type, event_id, delivery_status,
             attempt_count, next_retry_at, last_attempt_at, response_status_code,
             response_latency_ms, response_body_excerpt, dead_letter_reason,
             replay_of_delivery_id, created_at, updated_at",
    )
    .bind(params.organization_id)
    .bind(params.endpoint_id)
    .bind(params.event_type)
    .bind(params.event_id)
    .bind(params.request_payload)
    .bind(params.request_headers)
    .fetch_one(pool)
    .await
}

pub async fn list_webhook_delivery_logs(
    pool: &DbPool,
    organization_id: i64,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<WebhookDeliveryLogRecord>, sqlx::Error> {
    sqlx::query_as::<_, WebhookDeliveryLogRecord>(
        "SELECT id, organization_id, endpoint_id, event_type, event_id, delivery_status,
             attempt_count, next_retry_at, last_attempt_at, response_status_code,
             response_latency_ms, response_body_excerpt, dead_letter_reason,
             replay_of_delivery_id, created_at, updated_at
         FROM webhook_delivery_logs
         WHERE organization_id = $1
           AND ($2::TEXT IS NULL OR delivery_status = $2)
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(organization_id)
    .bind(status)
    .bind(limit.clamp(1, 250))
    .fetch_all(pool)
    .await
}

pub async fn mark_webhook_delivery_for_replay(
    pool: &DbPool,
    organization_id: i64,
    delivery_id: i64,
) -> Result<Option<WebhookDeliveryLogRecord>, sqlx::Error> {
    sqlx::query_as::<_, WebhookDeliveryLogRecord>(
        "INSERT INTO webhook_delivery_logs (
             organization_id, endpoint_id, event_type, event_id, delivery_status,
             request_payload, request_headers, replay_of_delivery_id, created_at, updated_at
         )
         SELECT organization_id, endpoint_id, event_type,
                CONCAT(event_id, ':replay:', $3::TEXT), 'replay_queued',
                request_payload, request_headers, id, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         FROM webhook_delivery_logs
         WHERE organization_id = $1 AND id = $2
         RETURNING id, organization_id, endpoint_id, event_type, event_id, delivery_status,
             attempt_count, next_retry_at, last_attempt_at, response_status_code,
             response_latency_ms, response_body_excerpt, dead_letter_reason,
             replay_of_delivery_id, created_at, updated_at",
    )
    .bind(organization_id)
    .bind(delivery_id)
    .bind(chrono::Utc::now().timestamp_millis())
    .fetch_optional(pool)
    .await
}
