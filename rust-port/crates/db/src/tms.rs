use chrono::NaiveDateTime;
use domain::{
    atmp_contract::{AtmpContractAction, AtmpContractEnvelope, stable_json_hash},
    tms::{
        HandoffStatus, HandoffStatusDescriptor, ReconciliationAction,
        ReconciliationActionDescriptor, TmsModuleContract, TmsStatus, TmsStatusDescriptor,
        TmsWebhookSurfaceDescriptor, handoff_status_descriptors, reconciliation_action_descriptors,
        tms_module_contract, tms_status_descriptors, tms_webhook_surfaces,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StloadsHandoffRecord {
    pub id: i64,
    pub tms_load_id: String,
    pub tenant_id: Option<String>,
    pub external_handoff_id: Option<String>,
    pub load_id: Option<i64>,
    pub status: String,
    pub tms_status: Option<String>,
    pub tms_status_at: Option<NaiveDateTime>,
    pub party_type: Option<String>,
    pub freight_mode: Option<String>,
    pub equipment_type: Option<String>,
    pub commodity_description: Option<String>,
    pub weight: Option<f64>,
    pub weight_unit: String,
    pub piece_count: Option<i32>,
    pub temperature_data: Option<Value>,
    pub container_data: Option<Value>,
    pub securement_data: Option<Value>,
    pub is_hazardous: bool,
    pub pickup_city: Option<String>,
    pub pickup_state: Option<String>,
    pub pickup_zip: Option<String>,
    pub pickup_country: String,
    pub pickup_address: Option<String>,
    pub pickup_window_start: Option<NaiveDateTime>,
    pub pickup_window_end: Option<NaiveDateTime>,
    pub pickup_instructions: Option<String>,
    pub pickup_appointment_ref: Option<String>,
    pub dropoff_city: Option<String>,
    pub dropoff_state: Option<String>,
    pub dropoff_zip: Option<String>,
    pub dropoff_country: String,
    pub dropoff_address: Option<String>,
    pub dropoff_window_start: Option<NaiveDateTime>,
    pub dropoff_window_end: Option<NaiveDateTime>,
    pub dropoff_instructions: Option<String>,
    pub dropoff_appointment_ref: Option<String>,
    pub board_rate: Option<f64>,
    pub rate_currency: String,
    pub accessorial_flags: Option<Value>,
    pub bid_type: String,
    pub quote_status: Option<String>,
    pub tender_posture: Option<String>,
    pub compliance_passed: bool,
    pub compliance_summary: Option<Value>,
    pub required_documents_status: Option<Value>,
    pub readiness: String,
    pub pushed_by: Option<String>,
    pub push_reason: Option<String>,
    pub source_module: Option<String>,
    pub queued_at: Option<NaiveDateTime>,
    pub published_at: Option<NaiveDateTime>,
    pub withdrawn_at: Option<NaiveDateTime>,
    pub closed_at: Option<NaiveDateTime>,
    pub retry_count: i32,
    pub last_push_result: Option<String>,
    pub payload_version: String,
    pub last_webhook_at: Option<NaiveDateTime>,
    pub raw_payload: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl StloadsHandoffRecord {
    pub fn handoff_status(&self) -> Option<HandoffStatus> {
        HandoffStatus::from_legacy_label(&self.status)
    }

    pub fn upstream_tms_status(&self) -> Option<TmsStatus> {
        self.tms_status
            .as_deref()
            .and_then(TmsStatus::from_legacy_label)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StloadsHandoffEventRecord {
    pub id: i64,
    pub handoff_id: i64,
    pub event_type: String,
    pub performed_by: Option<String>,
    pub source_module: Option<String>,
    pub payload_snapshot: Option<String>,
    pub result: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AtmpOutboundEventRecord {
    pub id: i64,
    pub tenant_id: String,
    pub event_id: String,
    pub correlation_id: Option<String>,
    pub event_type: String,
    pub posting_id: Option<i64>,
    pub booking_award_id: Option<i64>,
    pub target_url: Option<String>,
    pub payload: Value,
    pub payload_hash: String,
    pub status: String,
    pub attempt_count: i32,
    pub next_attempt_at: Option<NaiveDateTime>,
    pub last_attempt_at: Option<NaiveDateTime>,
    pub last_error: Option<String>,
    pub sent_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct EnqueueAtmpOutboundEvent<'a> {
    pub tenant_id: &'a str,
    pub event_type: &'a str,
    pub posting_id: Option<i64>,
    pub booking_award_id: Option<i64>,
    pub target_url: Option<&'a str>,
    pub payload: Value,
    pub correlation_id: Option<&'a str>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AtmpOutboundStatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StloadsExternalRefRecord {
    pub id: i64,
    pub handoff_id: i64,
    pub ref_type: String,
    pub ref_value: String,
    pub ref_source: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StloadsSyncErrorRecord {
    pub id: i64,
    pub handoff_id: Option<i64>,
    pub error_class: String,
    pub severity: String,
    pub title: String,
    pub detail: Option<String>,
    pub source_module: Option<String>,
    pub performed_by: Option<String>,
    pub resolved: bool,
    pub resolved_at: Option<NaiveDateTime>,
    pub resolved_by: Option<String>,
    pub resolution_note: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StloadsReconciliationLogRecord {
    pub id: i64,
    pub handoff_id: i64,
    pub action: String,
    pub tms_status_from: Option<String>,
    pub tms_status_to: Option<String>,
    pub stloads_status_from: Option<String>,
    pub stloads_status_to: Option<String>,
    pub detail: Option<String>,
    pub triggered_by: Option<String>,
    pub webhook_payload: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HandoffStatusCountRecord {
    pub status: String,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StloadsHandoffListRecord {
    pub id: i64,
    pub tms_load_id: String,
    pub load_id: Option<i64>,
    pub load_number: Option<String>,
    pub status: String,
    pub tms_status: Option<String>,
    pub freight_mode: Option<String>,
    pub equipment_type: Option<String>,
    pub pickup_city: Option<String>,
    pub pickup_state: Option<String>,
    pub dropoff_city: Option<String>,
    pub dropoff_state: Option<String>,
    pub board_rate: Option<f64>,
    pub retry_count: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StloadsMismatchCountsRecord {
    pub total_published: i64,
    pub tms_cancelled: i64,
    pub tms_delivered: i64,
    pub tms_invoiced: i64,
    pub no_tms_status: i64,
    pub stale_30d: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AtmpContractApplyResult {
    pub inbound_event_id: Option<i64>,
    pub posting_id: Option<i64>,
    pub handoff_id: Option<i64>,
    pub action: String,
    pub status_label: String,
    pub replayed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncErrorBreakdownRecord {
    pub error_class: String,
    pub severity: String,
    pub count: i64,
}

pub async fn find_active_handoff_by_tms_load(
    pool: &DbPool,
    tms_load_id: &str,
    tenant_id: &str,
) -> Result<Option<StloadsHandoffRecord>, sqlx::Error> {
    sqlx::query_as::<_, StloadsHandoffRecord>(
        "SELECT id, tms_load_id, tenant_id, external_handoff_id, load_id, status, tms_status,
                tms_status_at, party_type, freight_mode, equipment_type, commodity_description,
                weight::double precision AS weight, weight_unit, piece_count, temperature_data, container_data, securement_data,
                is_hazardous, pickup_city, pickup_state, pickup_zip, pickup_country, pickup_address,
                pickup_window_start, pickup_window_end, pickup_instructions, pickup_appointment_ref,
                dropoff_city, dropoff_state, dropoff_zip, dropoff_country, dropoff_address,
                dropoff_window_start, dropoff_window_end, dropoff_instructions, dropoff_appointment_ref,
                board_rate::double precision AS board_rate, rate_currency, accessorial_flags, bid_type, quote_status, tender_posture,
                compliance_passed, compliance_summary, required_documents_status, readiness, pushed_by,
                push_reason, source_module, queued_at, published_at, withdrawn_at, closed_at,
                retry_count, last_push_result, payload_version, last_webhook_at, raw_payload,
                created_at, updated_at
         FROM stloads_handoffs
         WHERE tms_load_id = $1 AND tenant_id = $2 AND status <> 'closed'
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(tms_load_id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_recent_handoffs(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<StloadsHandoffRecord>, sqlx::Error> {
    sqlx::query_as::<_, StloadsHandoffRecord>(
        "SELECT id, tms_load_id, tenant_id, external_handoff_id, load_id, status, tms_status,
                tms_status_at, party_type, freight_mode, equipment_type, commodity_description,
                weight::double precision AS weight, weight_unit, piece_count, temperature_data, container_data, securement_data,
                is_hazardous, pickup_city, pickup_state, pickup_zip, pickup_country, pickup_address,
                pickup_window_start, pickup_window_end, pickup_instructions, pickup_appointment_ref,
                dropoff_city, dropoff_state, dropoff_zip, dropoff_country, dropoff_address,
                dropoff_window_start, dropoff_window_end, dropoff_instructions, dropoff_appointment_ref,
                board_rate::double precision AS board_rate, rate_currency, accessorial_flags, bid_type, quote_status, tender_posture,
                compliance_passed, compliance_summary, required_documents_status, readiness, pushed_by,
                push_reason, source_module, queued_at, published_at, withdrawn_at, closed_at,
                retry_count, last_push_result, payload_version, last_webhook_at, raw_payload,
                created_at, updated_at
         FROM stloads_handoffs
         ORDER BY created_at DESC, id DESC
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_handoffs_filtered(
    pool: &DbPool,
    status_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<StloadsHandoffListRecord>, sqlx::Error> {
    if let Some(status) = status_filter {
        sqlx::query_as::<_, StloadsHandoffListRecord>(
            "SELECT h.id, h.tms_load_id, h.load_id, l.load_number, h.status, h.tms_status, h.freight_mode,
                    h.equipment_type, h.pickup_city, h.pickup_state, h.dropoff_city, h.dropoff_state,
                    h.board_rate::double precision AS board_rate, h.retry_count, h.created_at
             FROM stloads_handoffs h
             LEFT JOIN loads l ON l.id = h.load_id
             WHERE h.status = $1
             ORDER BY h.created_at DESC, h.id DESC
             LIMIT $2",
        )
        .bind(status)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, StloadsHandoffListRecord>(
            "SELECT h.id, h.tms_load_id, h.load_id, l.load_number, h.status, h.tms_status, h.freight_mode,
                    h.equipment_type, h.pickup_city, h.pickup_state, h.dropoff_city, h.dropoff_state,
                    h.board_rate::double precision AS board_rate, h.retry_count, h.created_at
             FROM stloads_handoffs h
             LEFT JOIN loads l ON l.id = h.load_id
             ORDER BY h.created_at DESC, h.id DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

pub async fn count_handoffs_by_status(
    pool: &DbPool,
) -> Result<Vec<HandoffStatusCountRecord>, sqlx::Error> {
    sqlx::query_as::<_, HandoffStatusCountRecord>(
        "SELECT status, COUNT(*) AS total
         FROM stloads_handoffs
         GROUP BY status",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_unresolved_sync_errors(
    pool: &DbPool,
) -> Result<Vec<StloadsSyncErrorRecord>, sqlx::Error> {
    sqlx::query_as::<_, StloadsSyncErrorRecord>(
        "SELECT id, handoff_id, error_class, severity, title, detail, source_module,
                performed_by, resolved, resolved_at, resolved_by, resolution_note,
                created_at, updated_at
         FROM stloads_sync_errors
         WHERE resolved = FALSE
         ORDER BY created_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_unresolved_sync_error_breakdown(
    pool: &DbPool,
) -> Result<Vec<SyncErrorBreakdownRecord>, sqlx::Error> {
    sqlx::query_as::<_, SyncErrorBreakdownRecord>(
        "SELECT error_class, severity, COUNT(*) AS count
         FROM stloads_sync_errors
         WHERE resolved = FALSE
         GROUP BY error_class, severity
         ORDER BY count DESC, error_class ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn count_unresolved_sync_errors_by_class(
    pool: &DbPool,
    error_class: &str,
) -> Result<i64, sqlx::Error> {
    let (total,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM stloads_sync_errors
        WHERE resolved = FALSE AND error_class = $1
        "#,
    )
    .bind(error_class)
    .fetch_one(pool)
    .await?;

    Ok(total)
}

pub async fn published_mismatch_counts(
    pool: &DbPool,
) -> Result<StloadsMismatchCountsRecord, sqlx::Error> {
    sqlx::query_as::<_, StloadsMismatchCountsRecord>(
        "SELECT
            SUM(CASE WHEN status = 'published' THEN 1 ELSE 0 END) AS total_published,
            SUM(CASE WHEN status = 'published' AND tms_status = 'cancelled' THEN 1 ELSE 0 END) AS tms_cancelled,
            SUM(CASE WHEN status = 'published' AND tms_status = 'delivered' THEN 1 ELSE 0 END) AS tms_delivered,
            SUM(CASE WHEN status = 'published' AND tms_status IN ('invoiced', 'settled') THEN 1 ELSE 0 END) AS tms_invoiced,
            SUM(CASE WHEN status = 'published' AND tms_status IS NULL THEN 1 ELSE 0 END) AS no_tms_status,
            SUM(CASE WHEN status = 'published'
                AND created_at < CURRENT_TIMESTAMP - INTERVAL '30 days'
                AND (last_webhook_at < CURRENT_TIMESTAMP - INTERVAL '30 days' OR last_webhook_at IS NULL)
                THEN 1 ELSE 0 END) AS stale_30d
         FROM stloads_handoffs",
    )
    .fetch_one(pool)
    .await
}

pub async fn list_recent_reconciliation_logs(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<StloadsReconciliationLogRecord>, sqlx::Error> {
    sqlx::query_as::<_, StloadsReconciliationLogRecord>(
        "SELECT id, handoff_id, action, tms_status_from, tms_status_to, stloads_status_from,
                stloads_status_to, detail, triggered_by, webhook_payload, created_at, updated_at
         FROM stloads_reconciliation_log
         ORDER BY created_at DESC, id DESC
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_reconciliation_logs_filtered(
    pool: &DbPool,
    action_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<StloadsReconciliationLogRecord>, sqlx::Error> {
    if let Some(action) = action_filter {
        sqlx::query_as::<_, StloadsReconciliationLogRecord>(
            "SELECT id, handoff_id, action, tms_status_from, tms_status_to, stloads_status_from,
                    stloads_status_to, detail, triggered_by, webhook_payload, created_at, updated_at
             FROM stloads_reconciliation_log
             WHERE action = $1
             ORDER BY created_at DESC, id DESC
             LIMIT $2",
        )
        .bind(action)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        list_recent_reconciliation_logs(pool, limit).await
    }
}

pub async fn tms_contract_summary() -> TmsModuleContract {
    tms_module_contract()
}

pub async fn handoff_status_catalog() -> &'static [HandoffStatusDescriptor] {
    handoff_status_descriptors()
}

pub async fn tms_status_catalog() -> &'static [TmsStatusDescriptor] {
    tms_status_descriptors()
}

pub async fn webhook_surface_catalog() -> &'static [TmsWebhookSurfaceDescriptor] {
    tms_webhook_surfaces()
}

pub async fn reconciliation_action_catalog() -> &'static [ReconciliationActionDescriptor] {
    reconciliation_action_descriptors()
}

pub async fn enqueue_atmp_outbound_event(
    pool: &DbPool,
    event: EnqueueAtmpOutboundEvent<'_>,
) -> Result<i64, sqlx::Error> {
    let event_type = event.event_type.trim();
    if !supported_atmp_outbound_event_types().contains(&event_type) {
        return Err(sqlx::Error::Protocol(
            format!("unsupported ATMP outbound event type: {event_type}").into(),
        ));
    }

    let tenant_id = event.tenant_id.trim();
    let event_id = format!("stloads-{}", uuid::Uuid::new_v4());
    let payload_hash = stable_json_hash(&event.payload);
    let mut tx = pool.begin().await?;
    ensure_contract_tenant(&mut tx, tenant_id).await?;
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO atmp_outbound_events (
            tenant_id, event_id, correlation_id, event_type, posting_id, booking_award_id,
            target_url, payload, payload_hash, status, next_attempt_at, created_at, updated_at
         ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, 'queued', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(&event_id)
    .bind(event.correlation_id)
    .bind(event_type)
    .bind(event.posting_id)
    .bind(event.booking_award_id)
    .bind(event.target_url)
    .bind(&event.payload)
    .bind(&payload_hash)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO atmp_contract_payloads (
            tenant_id, outbound_event_id, contract_version, direction, payload_hash, payload, created_at
         ) VALUES ($1, $2, '2026-05-01', 'outbound', $3, $4, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(id)
    .bind(&payload_hash)
    .bind(&event.payload)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(id)
}

pub async fn claim_due_atmp_outbound_events(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<AtmpOutboundEventRecord>, sqlx::Error> {
    let events = sqlx::query_as::<_, AtmpOutboundEventRecord>(
        "SELECT id, tenant_id, event_id, correlation_id, event_type, posting_id, booking_award_id,
                target_url, payload, payload_hash, status, attempt_count, next_attempt_at,
                last_attempt_at, last_error, sent_at, created_at, updated_at
         FROM atmp_outbound_events
         WHERE status IN ('queued', 'retry')
           AND (next_attempt_at IS NULL OR next_attempt_at <= CURRENT_TIMESTAMP)
         ORDER BY created_at ASC
         LIMIT $1",
    )
    .bind(limit.clamp(1, 250))
    .fetch_all(pool)
    .await?;

    for event in &events {
        sqlx::query(
            "UPDATE atmp_outbound_events
             SET status = 'processing', last_attempt_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE id = $1 AND status IN ('queued', 'retry')",
        )
        .bind(event.id)
        .execute(pool)
        .await?;
    }

    Ok(events)
}

pub async fn mark_atmp_outbound_event_delivered(
    pool: &DbPool,
    event_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE atmp_outbound_events
         SET status = 'delivered',
             attempt_count = attempt_count + 1,
             sent_at = CURRENT_TIMESTAMP,
             last_error = NULL,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_atmp_outbound_event_failed(
    pool: &DbPool,
    event_id: i64,
    error: &str,
    attempt_count: i32,
    dead_lettered: bool,
) -> Result<(), sqlx::Error> {
    let status = if dead_lettered {
        "dead_letter"
    } else {
        "retry"
    };
    let delay_seconds = retry_delay_seconds(attempt_count);
    sqlx::query(
        "UPDATE atmp_outbound_events
         SET status = $1,
             attempt_count = $2,
             next_attempt_at = CASE WHEN $1 = 'dead_letter' THEN NULL ELSE CURRENT_TIMESTAMP + ($3 * INTERVAL '1 second') END,
             last_error = $4,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $5",
    )
    .bind(status)
    .bind(attempt_count)
    .bind(delay_seconds)
    .bind(error)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn replay_atmp_outbound_event(
    pool: &DbPool,
    event_id: i64,
) -> Result<Option<AtmpOutboundEventRecord>, sqlx::Error> {
    sqlx::query(
        "UPDATE atmp_outbound_events
         SET status = 'queued',
             attempt_count = 0,
             next_attempt_at = CURRENT_TIMESTAMP,
             last_error = NULL,
             sent_at = NULL,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(event_id)
    .execute(pool)
    .await?;

    find_atmp_outbound_event_by_id(pool, event_id).await
}

pub async fn find_atmp_outbound_event_by_id(
    pool: &DbPool,
    event_id: i64,
) -> Result<Option<AtmpOutboundEventRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtmpOutboundEventRecord>(
        "SELECT id, tenant_id, event_id, correlation_id, event_type, posting_id, booking_award_id,
                target_url, payload, payload_hash, status, attempt_count, next_attempt_at,
                last_attempt_at, last_error, sent_at, created_at, updated_at
         FROM atmp_outbound_events
         WHERE id = $1",
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await
}

pub async fn count_atmp_outbound_events_by_status(
    pool: &DbPool,
) -> Result<Vec<AtmpOutboundStatusCount>, sqlx::Error> {
    sqlx::query_as::<_, AtmpOutboundStatusCount>(
        "SELECT status, COUNT(*) AS count
         FROM atmp_outbound_events
         GROUP BY status
         ORDER BY status",
    )
    .fetch_all(pool)
    .await
}

pub async fn force_atmp_outbound_event_due_for_test(
    pool: &DbPool,
    event_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE atmp_outbound_events
         SET next_attempt_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub fn supported_atmp_outbound_event_types() -> &'static [&'static str] {
    &[
        "listing_published",
        "listing_updated",
        "listing_withdrawn",
        "listing_closed",
        "listing_canceled",
        "listing_failed",
        "offer_submitted",
        "counteroffer_submitted",
        "offer_accepted",
        "offer_declined",
        "carrier_booked",
        "booking_canceled",
        "tracking_event",
        "exception_event",
        "document_uploaded",
        "document_approved",
        "document_rejected",
        "escrow_funded",
        "payment_hold",
        "payment_released",
        "settlement_ready",
        "sync_error",
    ]
}

fn retry_delay_seconds(attempt_count: i32) -> i64 {
    let exponent = attempt_count.saturating_sub(1).clamp(0, 8) as u32;
    30_i64.saturating_mul(2_i64.saturating_pow(exponent))
}

pub async fn apply_atmp_contract_event(
    pool: &DbPool,
    raw_payload: Value,
) -> Result<AtmpContractApplyResult, sqlx::Error> {
    let envelope = AtmpContractEnvelope::try_from_value(raw_payload.clone())
        .map_err(|error| sqlx::Error::Protocol(format!("{}: {}", error.code(), error).into()))?;

    if let Some(existing) = find_inbound_event_by_idempotency(
        pool,
        &envelope.tenant_id,
        &envelope.idempotency_key,
        envelope.action.as_str(),
    )
    .await?
    {
        return Ok(existing);
    }

    let normalized_payload = envelope.normalized_payload();
    let payload_hash = envelope.payload_hash();
    let mut tx = pool.begin().await?;

    ensure_contract_tenant(&mut tx, &envelope.tenant_id).await?;

    let inbound_event_id = insert_atmp_inbound_event(
        &mut tx,
        &envelope,
        &raw_payload,
        &normalized_payload,
        &payload_hash,
        "validated",
    )
    .await?;

    insert_contract_payload(
        &mut tx,
        &envelope.tenant_id,
        inbound_event_id,
        &envelope.contract_version,
        &payload_hash,
        &normalized_payload,
    )
    .await?;

    let (posting_id, status_label, message) = match envelope.action {
        AtmpContractAction::Publish => {
            let posting_id = upsert_atmp_posting(&mut tx, &envelope, &payload_hash, true).await?;
            insert_atmp_outbound_event_tx(
                &mut tx,
                &envelope.tenant_id,
                "listing_published",
                Some(posting_id),
                None,
                &serde_json::json!({
                    "event_type": "listing_published",
                    "tenant_id": envelope.tenant_id,
                    "atmp_load_id": envelope.atmp_load_id,
                    "atmp_leg_id": envelope.atmp_leg_id,
                    "posting_id": posting_id,
                    "correlation_id": envelope.correlation_id
                }),
                Some(&envelope.correlation_id),
            )
            .await?;
            (
                Some(posting_id),
                "Published".into(),
                "ATMP publish mapped to one active STLoads posting.".into(),
            )
        }
        AtmpContractAction::Update => {
            let posting_id = upsert_atmp_posting(&mut tx, &envelope, &payload_hash, false).await?;
            insert_listing_lifecycle_outbound_event(
                &mut tx,
                &envelope,
                "listing_updated",
                Some(posting_id),
            )
            .await?;
            (
                Some(posting_id),
                "Updated".into(),
                "ATMP update mapped into a new STLoads posting version.".into(),
            )
        }
        AtmpContractAction::Withdraw | AtmpContractAction::Cancel | AtmpContractAction::Close => {
            let posting_id = transition_atmp_posting(&mut tx, &envelope).await?;
            if let Some(posting_id) = posting_id {
                insert_listing_lifecycle_outbound_event(
                    &mut tx,
                    &envelope,
                    listing_lifecycle_event_type(envelope.action),
                    Some(posting_id),
                )
                .await?;
            }
            let status_label = envelope
                .action
                .terminal_status()
                .unwrap_or("terminal")
                .replace('_', " ");
            (
                posting_id,
                status_label,
                "ATMP terminal signal applied to the STLoads posting.".into(),
            )
        }
        AtmpContractAction::Status | AtmpContractAction::Document | AtmpContractAction::Finance => {
            let posting_id = find_posting_id_for_atmp_load(
                &mut tx,
                &envelope.tenant_id,
                &envelope.atmp_load_id,
                envelope.atmp_leg_id.as_deref(),
            )
            .await?;
            if let Some(posting_id) = posting_id {
                append_posting_version(&mut tx, &envelope, posting_id, &payload_hash).await?;
            }
            (
                posting_id,
                envelope.action.as_str().into(),
                "ATMP signal persisted and linked when a posting exists.".into(),
            )
        }
    };

    sqlx::query(
        "UPDATE atmp_inbound_events
         SET validation_status = 'processed',
             processed_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(inbound_event_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(AtmpContractApplyResult {
        inbound_event_id: Some(inbound_event_id),
        posting_id,
        handoff_id: None,
        action: envelope.action.as_str().into(),
        status_label,
        replayed: false,
        message,
    })
}

pub async fn find_sync_error_by_id(
    pool: &DbPool,
    sync_error_id: i64,
) -> Result<Option<StloadsSyncErrorRecord>, sqlx::Error> {
    sqlx::query_as::<_, StloadsSyncErrorRecord>(
        "SELECT id, handoff_id, error_class, severity, title, detail, source_module,
                performed_by, resolved, resolved_at, resolved_by, resolution_note,
                created_at, updated_at
         FROM stloads_sync_errors
         WHERE id = $1
         LIMIT 1",
    )
    .bind(sync_error_id)
    .fetch_optional(pool)
    .await
}

pub async fn resolve_sync_error(
    pool: &DbPool,
    sync_error_id: i64,
    resolved_by: &str,
    resolution_note: Option<&str>,
) -> Result<Option<StloadsSyncErrorRecord>, sqlx::Error> {
    let normalized_note = resolution_note
        .map(str::trim)
        .filter(|note| !note.is_empty())
        .map(str::to_string);

    sqlx::query(
        "UPDATE stloads_sync_errors
         SET resolved = TRUE,
             resolved_at = CURRENT_TIMESTAMP,
             resolved_by = $1,
             resolution_note = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3 AND resolved = FALSE",
    )
    .bind(resolved_by)
    .bind(normalized_note.as_deref())
    .bind(sync_error_id)
    .execute(pool)
    .await?;

    find_sync_error_by_id(pool, sync_error_id).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedHandoffResult {
    pub handoff: StloadsHandoffRecord,
    pub load_id: Option<i64>,
    pub load_number: Option<String>,
    pub action_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsWebhookMutationResult {
    pub handoff: StloadsHandoffRecord,
    pub action_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TmsRetryRunSummary {
    pub scanned: usize,
    pub published: usize,
    pub failed: usize,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TmsReconciliationScanSummary {
    pub auto_archived: usize,
    pub cancelled_still_live: usize,
    pub delivered_still_open: usize,
    pub stale_handoffs: usize,
}

pub async fn find_latest_handoff_for_load(
    pool: &DbPool,
    load_id: i64,
) -> Result<Option<StloadsHandoffRecord>, sqlx::Error> {
    sqlx::query_as::<_, StloadsHandoffRecord>(
        "SELECT id, tms_load_id, tenant_id, external_handoff_id, load_id, status, tms_status,
                tms_status_at, party_type, freight_mode, equipment_type, commodity_description,
                weight::double precision AS weight, weight_unit, piece_count, temperature_data, container_data, securement_data,
                is_hazardous, pickup_city, pickup_state, pickup_zip, pickup_country, pickup_address,
                pickup_window_start, pickup_window_end, pickup_instructions, pickup_appointment_ref,
                dropoff_city, dropoff_state, dropoff_zip, dropoff_country, dropoff_address,
                dropoff_window_start, dropoff_window_end, dropoff_instructions, dropoff_appointment_ref,
                board_rate::double precision AS board_rate, rate_currency, accessorial_flags, bid_type, quote_status, tender_posture,
                compliance_passed, compliance_summary, required_documents_status, readiness, pushed_by,
                push_reason, source_module, queued_at, published_at, withdrawn_at, closed_at,
                retry_count, last_push_result, payload_version, last_webhook_at, raw_payload,
                created_at, updated_at
         FROM stloads_handoffs
         WHERE load_id = $1
         ORDER BY id DESC
         LIMIT 1"
    )
    .bind(load_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_handoff_by_id(
    pool: &DbPool,
    handoff_id: i64,
) -> Result<Option<StloadsHandoffRecord>, sqlx::Error> {
    sqlx::query_as::<_, StloadsHandoffRecord>(
        "SELECT id, tms_load_id, tenant_id, external_handoff_id, load_id, status, tms_status,
                tms_status_at, party_type, freight_mode, equipment_type, commodity_description,
                weight::double precision AS weight, weight_unit, piece_count, temperature_data, container_data, securement_data,
                is_hazardous, pickup_city, pickup_state, pickup_zip, pickup_country, pickup_address,
                pickup_window_start, pickup_window_end, pickup_instructions, pickup_appointment_ref,
                dropoff_city, dropoff_state, dropoff_zip, dropoff_country, dropoff_address,
                dropoff_window_start, dropoff_window_end, dropoff_instructions, dropoff_appointment_ref,
                board_rate::double precision AS board_rate, rate_currency, accessorial_flags, bid_type, quote_status, tender_posture,
                compliance_passed, compliance_summary, required_documents_status, readiness, pushed_by,
                push_reason, source_module, queued_at, published_at, withdrawn_at, closed_at,
                retry_count, last_push_result, payload_version, last_webhook_at, raw_payload,
                created_at, updated_at
         FROM stloads_handoffs
         WHERE id = $1
         LIMIT 1",
    )
    .bind(handoff_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_sync_error(
    pool: &DbPool,
    handoff_id: Option<i64>,
    error_class: &str,
    severity: &str,
    title: &str,
    detail: Option<&str>,
    source_module: Option<&str>,
    performed_by: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO stloads_sync_errors (
            handoff_id, error_class, severity, title, detail, source_module, performed_by,
            resolved, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(handoff_id)
    .bind(error_class)
    .bind(severity)
    .bind(title)
    .bind(detail)
    .bind(source_module)
    .bind(performed_by)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn queue_handoff(
    pool: &DbPool,
    payload: &shared::TmsHandoffPayload,
) -> Result<StloadsHandoffRecord, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let raw_payload = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
    let handoff_id = insert_handoff_row(
        &mut tx,
        payload,
        HandoffStatus::Queued.as_legacy_label(),
        &raw_payload,
    )
    .await?;
    insert_handoff_event_row(
        &mut tx,
        handoff_id,
        "queued",
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
        Some(&raw_payload),
        Some("accepted into queue"),
    )
    .await?;
    if let Some(external_refs) = payload.external_refs.as_deref() {
        insert_external_refs_rows(&mut tx, handoff_id, external_refs).await?;
    }
    tx.commit().await?;

    find_handoff_by_id(pool, handoff_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn push_handoff(
    pool: &DbPool,
    payload: &shared::TmsHandoffPayload,
) -> Result<MaterializedHandoffResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let raw_payload = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
    let handoff_id = insert_handoff_row(
        &mut tx,
        payload,
        HandoffStatus::PushInProgress.as_legacy_label(),
        &raw_payload,
    )
    .await?;

    insert_handoff_event_row(
        &mut tx,
        handoff_id,
        "push_started",
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
        Some(&raw_payload),
        Some("materializing local load"),
    )
    .await?;

    if let Some(external_refs) = payload.external_refs.as_deref() {
        insert_external_refs_rows(&mut tx, handoff_id, external_refs).await?;
    }

    let (load_id, load_number) = materialize_load_for_handoff(&mut tx, payload, handoff_id).await?;

    sqlx::query(
        "UPDATE stloads_handoffs
         SET load_id = $1,
             status = $2,
             published_at = CURRENT_TIMESTAMP,
             last_push_result = $3,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4",
    )
    .bind(load_id)
    .bind(HandoffStatus::Published.as_legacy_label())
    .bind("published from Rust TMS push route")
    .bind(handoff_id)
    .execute(&mut *tx)
    .await?;

    insert_handoff_event_row(
        &mut tx,
        handoff_id,
        "published",
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
        Some(&raw_payload),
        Some("success"),
    )
    .await?;

    tx.commit().await?;

    let handoff = find_handoff_by_id(pool, handoff_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(MaterializedHandoffResult {
        handoff,
        load_id: Some(load_id),
        load_number: Some(load_number),
        action_label: "published".into(),
    })
}

pub async fn requeue_handoff(
    pool: &DbPool,
    handoff_id: i64,
    performed_by: Option<&str>,
    source_module: Option<&str>,
) -> Result<Option<MaterializedHandoffResult>, sqlx::Error> {
    let Some(existing_handoff) = find_handoff_by_id(pool, handoff_id).await? else {
        return Ok(None);
    };

    let mut tx = pool.begin().await?;
    let payload = existing_handoff
        .raw_payload
        .clone()
        .and_then(|value| serde_json::from_value::<shared::TmsHandoffPayload>(value).ok())
        .ok_or(sqlx::Error::Protocol(
            "handoff raw payload is missing or invalid".into(),
        ))?;
    let raw_payload = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);

    sqlx::query(
        "UPDATE stloads_handoffs
         SET status = $1,
             retry_count = retry_count + 1,
             last_push_result = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3",
    )
    .bind(HandoffStatus::PushInProgress.as_legacy_label())
    .bind("requeue started from Rust TMS route")
    .bind(handoff_id)
    .execute(&mut *tx)
    .await?;

    insert_handoff_event_row(
        &mut tx,
        handoff_id,
        "requeue_started",
        performed_by.or(payload.pushed_by.as_deref()),
        source_module.or(payload.source_module.as_deref()),
        Some(&raw_payload),
        Some("retrying handoff publication"),
    )
    .await?;

    let (load_id, load_number) = match existing_handoff.load_id {
        Some(load_id) => {
            restore_load_projection(&mut tx, load_id).await?;
            let load_number = fetch_load_number(&mut tx, load_id).await?;
            (
                load_id,
                load_number.unwrap_or_else(|| {
                    format!(
                        "TMS-{}-H{}",
                        sanitize_load_token(&existing_handoff.tms_load_id),
                        handoff_id
                    )
                }),
            )
        }
        None => materialize_load_for_handoff(&mut tx, &payload, handoff_id).await?,
    };

    sqlx::query(
        "UPDATE stloads_handoffs
         SET load_id = $1,
             status = $2,
             published_at = CURRENT_TIMESTAMP,
             last_push_result = $3,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4",
    )
    .bind(load_id)
    .bind(HandoffStatus::Published.as_legacy_label())
    .bind("published after Rust TMS requeue")
    .bind(handoff_id)
    .execute(&mut *tx)
    .await?;

    insert_handoff_event_row(
        &mut tx,
        handoff_id,
        "published",
        performed_by.or(payload.pushed_by.as_deref()),
        source_module.or(payload.source_module.as_deref()),
        Some(&raw_payload),
        Some("success"),
    )
    .await?;

    tx.commit().await?;

    let handoff = find_handoff_by_id(pool, handoff_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(Some(MaterializedHandoffResult {
        handoff,
        load_id: Some(load_id),
        load_number: Some(load_number),
        action_label: "published".into(),
    }))
}

pub async fn withdraw_handoff(
    pool: &DbPool,
    handoff_id: i64,
    reason: Option<&str>,
    performed_by: Option<&str>,
    source_module: Option<&str>,
) -> Result<Option<StloadsHandoffRecord>, sqlx::Error> {
    let Some(existing_handoff) = find_handoff_by_id(pool, handoff_id).await? else {
        return Ok(None);
    };

    let mut tx = pool.begin().await?;

    if let Some(load_id) = existing_handoff.load_id {
        soft_delete_load_projection(&mut tx, load_id).await?;
    }

    sqlx::query(
        "UPDATE stloads_handoffs
         SET status = $1,
             withdrawn_at = CURRENT_TIMESTAMP,
             last_push_result = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3",
    )
    .bind(HandoffStatus::Withdrawn.as_legacy_label())
    .bind(reason.unwrap_or("withdrawn by Rust TMS route"))
    .bind(handoff_id)
    .execute(&mut *tx)
    .await?;

    insert_handoff_event_row(
        &mut tx,
        handoff_id,
        "withdrawn",
        performed_by,
        source_module,
        None,
        reason,
    )
    .await?;

    insert_reconciliation_log_row(
        &mut tx,
        handoff_id,
        ReconciliationAction::AutoWithdraw.as_legacy_label(),
        existing_handoff.tms_status.as_deref(),
        existing_handoff.tms_status.as_deref(),
        Some(existing_handoff.status.as_str()),
        Some(HandoffStatus::Withdrawn.as_legacy_label()),
        reason.unwrap_or("Withdrawn from Rust TMS route."),
        performed_by.or(source_module).unwrap_or("operator"),
        None,
    )
    .await?;

    tx.commit().await?;
    find_handoff_by_id(pool, handoff_id).await
}

pub async fn close_handoff(
    pool: &DbPool,
    handoff_id: i64,
    reason: Option<&str>,
    performed_by: Option<&str>,
    source_module: Option<&str>,
) -> Result<Option<StloadsHandoffRecord>, sqlx::Error> {
    let Some(existing_handoff) = find_handoff_by_id(pool, handoff_id).await? else {
        return Ok(None);
    };

    let mut tx = pool.begin().await?;

    if existing_handoff.status == HandoffStatus::Published.as_legacy_label() {
        if let Some(load_id) = existing_handoff.load_id {
            soft_delete_load_projection(&mut tx, load_id).await?;
        }
    }

    sqlx::query(
        "UPDATE stloads_handoffs
         SET status = $1,
             closed_at = CURRENT_TIMESTAMP,
             last_push_result = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3",
    )
    .bind(HandoffStatus::Closed.as_legacy_label())
    .bind(reason.unwrap_or("closed by Rust TMS route"))
    .bind(handoff_id)
    .execute(&mut *tx)
    .await?;

    insert_handoff_event_row(
        &mut tx,
        handoff_id,
        "closed",
        performed_by,
        source_module,
        None,
        reason,
    )
    .await?;

    insert_reconciliation_log_row(
        &mut tx,
        handoff_id,
        ReconciliationAction::AutoArchive.as_legacy_label(),
        existing_handoff.tms_status.as_deref(),
        existing_handoff.tms_status.as_deref(),
        Some(existing_handoff.status.as_str()),
        Some(HandoffStatus::Closed.as_legacy_label()),
        reason.unwrap_or("Closed from Rust TMS route."),
        performed_by.or(source_module).unwrap_or("operator"),
        None,
    )
    .await?;

    tx.commit().await?;
    find_handoff_by_id(pool, handoff_id).await
}

pub async fn apply_status_webhook(
    pool: &DbPool,
    payload: &shared::TmsStatusWebhookRequest,
) -> Result<Option<TmsWebhookMutationResult>, sqlx::Error> {
    let Some(existing_handoff) =
        find_active_handoff_by_tms_load(pool, &payload.tms_load_id, &payload.tenant_id).await?
    else {
        return Ok(None);
    };

    let mut tx = pool.begin().await?;
    let webhook_payload = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
    let normalized_tms_status = payload.tms_status.trim().to_lowercase();
    let parsed_status_at = parse_optional_datetime(payload.status_at.as_deref());
    let previous_status = existing_handoff.status.clone();
    let previous_tms_status = existing_handoff.tms_status.clone();
    let mut next_handoff_status = existing_handoff.status.clone();
    let mut action_label = ReconciliationAction::StatusUpdate
        .as_legacy_label()
        .to_string();
    let mut detail = payload.detail.clone().unwrap_or_else(|| {
        format!(
            "Received {} from the Rust TMS webhook route.",
            normalized_tms_status
        )
    });

    if let Some(rate_update) = payload.rate_update {
        sqlx::query(
            "UPDATE stloads_handoffs
             SET board_rate = $1,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $2",
        )
        .bind(rate_update)
        .bind(existing_handoff.id)
        .execute(&mut *tx)
        .await?;

        if let Some(load_id) = existing_handoff.load_id {
            sqlx::query(
                "UPDATE load_legs
                 SET price = $1,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE load_id = $2 AND deleted_at IS NULL",
            )
            .bind(rate_update)
            .bind(load_id)
            .execute(&mut *tx)
            .await?;
        }

        if existing_handoff.status == HandoffStatus::Published.as_legacy_label() {
            next_handoff_status = HandoffStatus::RequeueRequired.as_legacy_label().to_string();
            action_label = ReconciliationAction::RateUpdate
                .as_legacy_label()
                .to_string();
            detail = format!(
                "Rate update received from TMS and the handoff now requires requeue at {}.",
                rate_update
            );
        }
    }

    if normalized_tms_status == TmsStatus::Cancelled.as_legacy_label()
        && existing_handoff.status == HandoffStatus::Published.as_legacy_label()
    {
        next_handoff_status = HandoffStatus::Withdrawn.as_legacy_label().to_string();
        action_label = ReconciliationAction::AutoWithdraw
            .as_legacy_label()
            .to_string();
        detail = payload
            .detail
            .clone()
            .unwrap_or_else(|| "Cancellation webhook auto-withdrew the board posting.".into());

        if let Some(load_id) = existing_handoff.load_id {
            soft_delete_load_projection(&mut tx, load_id).await?;
        }
    } else if matches!(normalized_tms_status.as_str(), "invoiced" | "settled")
        && matches!(existing_handoff.status.as_str(), "published" | "withdrawn")
    {
        next_handoff_status = HandoffStatus::Closed.as_legacy_label().to_string();
        action_label = ReconciliationAction::AutoArchive
            .as_legacy_label()
            .to_string();
        detail = payload
            .detail
            .clone()
            .unwrap_or_else(|| "Finance-complete webhook closed the STLOADS handoff.".into());

        if existing_handoff.status == HandoffStatus::Published.as_legacy_label() {
            if let Some(load_id) = existing_handoff.load_id {
                soft_delete_load_projection(&mut tx, load_id).await?;
            }
        }
    }

    sqlx::query(
        "UPDATE stloads_handoffs
         SET tms_status = $1,
             tms_status_at = $2,
             status = $3,
             last_webhook_at = CURRENT_TIMESTAMP,
             source_module = $4,
             last_push_result = $5,
             withdrawn_at = CASE WHEN $6 = 'withdrawn' AND withdrawn_at IS NULL THEN CURRENT_TIMESTAMP ELSE withdrawn_at END,
             closed_at = CASE WHEN $7 = 'closed' AND closed_at IS NULL THEN CURRENT_TIMESTAMP ELSE closed_at END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $8",
    )
    .bind(normalized_tms_status.as_str())
    .bind(parsed_status_at)
    .bind(next_handoff_status.as_str())
    .bind(payload.source_module.as_deref())
    .bind(detail.as_str())
    .bind(next_handoff_status.as_str())
    .bind(next_handoff_status.as_str())
    .bind(existing_handoff.id)
    .execute(&mut *tx)
    .await?;

    insert_handoff_event_row(
        &mut tx,
        existing_handoff.id,
        format!("webhook_{}", action_label).as_str(),
        payload.pushed_by.as_deref(),
        payload.source_module.as_deref(),
        Some(&webhook_payload),
        Some(detail.as_str()),
    )
    .await?;

    insert_reconciliation_log_row(
        &mut tx,
        existing_handoff.id,
        action_label.as_str(),
        previous_tms_status.as_deref(),
        Some(normalized_tms_status.as_str()),
        Some(previous_status.as_str()),
        Some(next_handoff_status.as_str()),
        detail.as_str(),
        payload
            .pushed_by
            .as_deref()
            .or(payload.source_module.as_deref())
            .unwrap_or("webhook"),
        Some(&webhook_payload),
    )
    .await?;

    tx.commit().await?;

    let handoff = find_handoff_by_id(pool, existing_handoff.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(Some(TmsWebhookMutationResult {
        handoff,
        action_label,
        message: detail,
    }))
}

pub async fn process_retryable_handoffs(
    pool: &DbPool,
    limit: i64,
    max_attempts: i32,
) -> Result<TmsRetryRunSummary, sqlx::Error> {
    let handoff_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM stloads_handoffs
        WHERE status IN ('queued', 'push_failed')
          AND retry_count < $1
        ORDER BY COALESCE(queued_at, updated_at, created_at) ASC, id ASC
        LIMIT $2
        "#,
    )
    .bind(max_attempts.max(1))
    .bind(limit.max(1))
    .fetch_all(pool)
    .await?;

    let mut summary = TmsRetryRunSummary {
        scanned: handoff_ids.len(),
        ..TmsRetryRunSummary::default()
    };

    for handoff_id in handoff_ids {
        match requeue_handoff(
            pool,
            handoff_id,
            Some("rust_tms_retry_worker"),
            Some("rust_scheduler"),
        )
        .await
        {
            Ok(Some(result)) => {
                summary.published += 1;
                summary.messages.push(format!(
                    "Published handoff #{} from retry worker.",
                    result.handoff.id
                ));
            }
            Ok(None) => {
                summary.failed += 1;
                summary.messages.push(format!(
                    "Retry handoff #{} disappeared before processing.",
                    handoff_id
                ));
            }
            Err(error) => {
                summary.failed += 1;
                record_handoff_retry_failure(pool, handoff_id, &error.to_string()).await?;
                summary
                    .messages
                    .push(format!("Retry handoff #{} failed: {}", handoff_id, error));
            }
        }
    }

    Ok(summary)
}

pub async fn run_reconciliation_scan(
    pool: &DbPool,
    stale_days: i64,
) -> Result<TmsReconciliationScanSummary, sqlx::Error> {
    let mut summary = TmsReconciliationScanSummary::default();

    let archive_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM stloads_handoffs
        WHERE status IN ('published', 'withdrawn')
          AND tms_status IN ('invoiced', 'settled')
        ORDER BY updated_at ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    for handoff_id in archive_ids {
        if close_handoff(
            pool,
            handoff_id,
            Some("Reconciliation scan: TMS reached a financial terminal status."),
            Some("rust_tms_reconciliation_worker"),
            Some("rust_scheduler"),
        )
        .await?
        .is_some()
        {
            summary.auto_archived += 1;
        }
    }

    let cancelled_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM stloads_handoffs
        WHERE status = 'published'
          AND tms_status = 'cancelled'
        ORDER BY updated_at ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    for handoff_id in cancelled_ids {
        if withdraw_handoff(
            pool,
            handoff_id,
            Some("Reconciliation scan: TMS cancelled but STLOADS was still published."),
            Some("rust_tms_reconciliation_worker"),
            Some("rust_scheduler"),
        )
        .await?
        .is_some()
        {
            summary.cancelled_still_live += 1;
        }
    }

    summary.delivered_still_open = flag_delivered_still_open(pool).await?;
    summary.stale_handoffs = flag_stale_handoffs(pool, stale_days).await?;

    Ok(summary)
}

async fn record_handoff_retry_failure(
    pool: &DbPool,
    handoff_id: i64,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE stloads_handoffs
        SET status = 'push_failed',
            retry_count = retry_count + 1,
            last_push_result = $2,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
    )
    .bind(handoff_id)
    .bind(error.chars().take(2000).collect::<String>())
    .execute(pool)
    .await?;

    create_sync_error(
        pool,
        Some(handoff_id),
        "push_failed",
        "error",
        &format!("Retry failed for STLOADS handoff #{}", handoff_id),
        Some(error),
        Some("rust_scheduler"),
        Some("rust_tms_retry_worker"),
    )
    .await
}

async fn flag_delivered_still_open(pool: &DbPool) -> Result<usize, sqlx::Error> {
    let handoff_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT h.id
        FROM stloads_handoffs h
        WHERE h.status = 'published'
          AND h.load_id IS NOT NULL
          AND EXISTS (
              SELECT 1 FROM load_legs leg
              WHERE leg.load_id = h.load_id
                AND leg.deleted_at IS NULL
          )
          AND NOT EXISTS (
              SELECT 1 FROM load_legs leg
              WHERE leg.load_id = h.load_id
                AND leg.deleted_at IS NULL
                AND leg.status_id < 10
          )
          AND NOT EXISTS (
              SELECT 1 FROM stloads_sync_errors err
              WHERE err.handoff_id = h.id
                AND err.error_class = 'delivered_still_open'
                AND err.resolved = FALSE
          )
        "#,
    )
    .fetch_all(pool)
    .await?;

    for handoff_id in &handoff_ids {
        create_sync_error(
            pool,
            Some(*handoff_id),
            "delivered_still_open",
            "warning",
            &format!("Load delivered but STLOADS handoff #{} is still open.", handoff_id),
            Some("All load legs are completed but the STLOADS handoff is still published. Consider closing or withdrawing."),
            Some("rust_scheduler"),
            Some("rust_tms_reconciliation_worker"),
        )
        .await?;
    }

    Ok(handoff_ids.len())
}

async fn flag_stale_handoffs(pool: &DbPool, stale_days: i64) -> Result<usize, sqlx::Error> {
    let handoff_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT h.id
        FROM stloads_handoffs h
        WHERE h.status = 'published'
          AND h.published_at < CURRENT_TIMESTAMP - (($1::TEXT || ' days')::INTERVAL)
          AND (h.last_webhook_at IS NULL OR h.last_webhook_at < CURRENT_TIMESTAMP - (($1::TEXT || ' days')::INTERVAL))
          AND NOT EXISTS (
              SELECT 1 FROM stloads_sync_errors err
              WHERE err.handoff_id = h.id
                AND err.error_class = 'stale_handoff'
                AND err.resolved = FALSE
          )
        "#,
    )
    .bind(stale_days.max(1))
    .fetch_all(pool)
    .await?;

    for handoff_id in &handoff_ids {
        create_sync_error(
            pool,
            Some(*handoff_id),
            "stale_handoff",
            "warning",
            &format!("Stale STLOADS handoff #{} has no recent TMS activity.", handoff_id),
            Some("Published handoff has exceeded the stale webhook threshold. Investigate TMS status or close the handoff."),
            Some("rust_scheduler"),
            Some("rust_tms_reconciliation_worker"),
        )
        .await?;
    }

    Ok(handoff_ids.len())
}

async fn insert_handoff_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    payload: &shared::TmsHandoffPayload,
    status: &str,
    raw_payload: &serde_json::Value,
) -> Result<i64, sqlx::Error> {
    let pickup_window_start = parse_required_datetime(&payload.pickup_window_start)?;
    let pickup_window_end = parse_optional_datetime(payload.pickup_window_end.as_deref());
    let dropoff_window_start = parse_required_datetime(&payload.dropoff_window_start)?;
    let dropoff_window_end = parse_optional_datetime(payload.dropoff_window_end.as_deref());

    let handoff_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO stloads_handoffs (
            tms_load_id, tenant_id, external_handoff_id, status, party_type, freight_mode,
            equipment_type, commodity_description, weight, weight_unit, piece_count,
            temperature_data, container_data, securement_data, is_hazardous,
            pickup_city, pickup_state, pickup_zip, pickup_country, pickup_address,
            pickup_window_start, pickup_window_end, pickup_instructions, pickup_appointment_ref,
            dropoff_city, dropoff_state, dropoff_zip, dropoff_country, dropoff_address,
            dropoff_window_start, dropoff_window_end, dropoff_instructions, dropoff_appointment_ref,
            board_rate, rate_currency, accessorial_flags, bid_type, quote_status, tender_posture,
            compliance_passed, compliance_summary, required_documents_status, readiness,
            pushed_by, push_reason, source_module, queued_at, retry_count, last_push_result,
            payload_version, raw_payload, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42, $43, $44, $45, $46, $47, 0, $48, $49, $50, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id",
    )
    .bind(payload.tms_load_id.as_str())
    .bind(payload.tenant_id.as_str())
    .bind(payload.external_handoff_id.as_deref())
    .bind(status)
    .bind(payload.party_type.as_str())
    .bind(payload.freight_mode.as_str())
    .bind(payload.equipment_type.as_str())
    .bind(payload.commodity_description.as_deref())
    .bind(payload.weight)
    .bind(payload.weight_unit.as_str())
    .bind(payload.piece_count)
    .bind(payload.temperature_data.clone())
    .bind(payload.container_data.clone())
    .bind(payload.securement_data.clone())
    .bind(payload.is_hazardous.unwrap_or(false))
    .bind(payload.pickup_city.as_str())
    .bind(payload.pickup_state.as_deref())
    .bind(payload.pickup_zip.as_deref())
    .bind(payload.pickup_country.as_str())
    .bind(payload.pickup_address.as_str())
    .bind(pickup_window_start)
    .bind(pickup_window_end)
    .bind(payload.pickup_instructions.as_deref())
    .bind(payload.pickup_appointment_ref.as_deref())
    .bind(payload.dropoff_city.as_str())
    .bind(payload.dropoff_state.as_deref())
    .bind(payload.dropoff_zip.as_deref())
    .bind(payload.dropoff_country.as_str())
    .bind(payload.dropoff_address.as_str())
    .bind(dropoff_window_start)
    .bind(dropoff_window_end)
    .bind(payload.dropoff_instructions.as_deref())
    .bind(payload.dropoff_appointment_ref.as_deref())
    .bind(payload.board_rate)
    .bind(payload.rate_currency.as_deref().unwrap_or("USD"))
    .bind(payload.accessorial_flags.clone())
    .bind(payload.bid_type.as_str())
    .bind(payload.quote_status.as_deref())
    .bind(payload.tender_posture.as_deref())
    .bind(payload.compliance_passed.unwrap_or(false))
    .bind(payload.compliance_summary.clone())
    .bind(payload.required_documents_status.clone())
    .bind(payload.readiness.as_deref().unwrap_or("pending"))
    .bind(payload.pushed_by.as_deref())
    .bind(payload.push_reason.as_deref())
    .bind(payload.source_module.as_deref())
    .bind(if status == HandoffStatus::Queued.as_legacy_label() { Some(chrono::Utc::now().naive_utc()) } else { None })
    .bind(format!("{} created from Rust TMS route", status))
    .bind(payload.payload_version.as_deref().unwrap_or("1.0"))
    .bind(raw_payload)
    .fetch_one(&mut **tx)
    .await?;

    Ok(handoff_id)
}

async fn materialize_load_for_handoff(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    payload: &shared::TmsHandoffPayload,
    handoff_id: i64,
) -> Result<(i64, String), sqlx::Error> {
    let pickup_location_id = insert_location_row(&mut *tx, &payload.pickup_address).await?;
    let delivery_location_id = insert_location_row(&mut *tx, &payload.dropoff_address).await?;
    let pickup_date = parse_required_datetime(&payload.pickup_window_start)?;
    let delivery_date = parse_required_datetime(&payload.dropoff_window_start)?;
    let load_number = format!(
        "TMS-{}-H{}",
        sanitize_load_token(&payload.tms_load_id),
        handoff_id
    );
    let load_title = format!(
        "{}: {} -> {}",
        payload.freight_mode, payload.pickup_city, payload.dropoff_city
    );
    let owner_user_id = resolve_tms_owner_user_id(&mut *tx, payload).await?;

    let load_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO loads (
            load_number, title, user_id, load_type_id, equipment_id, commodity_type_id,
            weight_unit, weight, special_instructions, is_hazardous, is_temperature_controlled,
            status, leg_count, created_at, updated_at
         ) VALUES ($1, $2, $3, NULL, NULL, NULL, $4, $5, $6, $7, $8, 1, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(load_number.as_str())
    .bind(load_title.as_str())
    .bind(owner_user_id)
    .bind(payload.weight_unit.as_str())
    .bind(payload.weight)
    .bind(payload.pickup_instructions.as_deref())
    .bind(payload.is_hazardous.unwrap_or(false))
    .bind(payload.temperature_data.is_some())
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_legs (
            load_id, leg_no, leg_code, pickup_location_id, delivery_location_id,
            pickup_date, delivery_date, bid_status, price, status_id, created_at, updated_at
         ) VALUES ($1, 1, $2, $3, $4, $5, $6, $7, $8, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(format!("{}-1", load_number))
    .bind(pickup_location_id)
    .bind(delivery_location_id)
    .bind(pickup_date)
    .bind(delivery_date)
    .bind(payload.bid_type.as_str())
    .bind(payload.board_rate)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, NULL, 1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind("Rust TMS push created a local load projection")
    .execute(&mut **tx)
    .await?;

    Ok((load_id, load_number))
}

async fn resolve_tms_owner_user_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    payload: &shared::TmsHandoffPayload,
) -> Result<Option<i64>, sqlx::Error> {
    let Some(owner_email) = payload.external_refs.as_deref().and_then(|refs| {
        refs.iter()
            .find(|external_ref| {
                matches!(
                    external_ref.ref_type.as_str(),
                    "stloads_owner_email" | "owner_email" | "dispatch_account_email"
                )
            })
            .map(|external_ref| external_ref.ref_value.trim())
            .filter(|value| !value.is_empty())
    }) else {
        return Ok(None);
    };

    sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM users
         WHERE LOWER(email) = LOWER($1)
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(owner_email)
    .fetch_optional(&mut **tx)
    .await
}

async fn insert_location_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    name: &str,
) -> Result<i64, sqlx::Error> {
    let location_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO locations (name, city_id, country_id, created_at, updated_at)
         VALUES ($1, NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(name)
    .fetch_one(&mut **tx)
    .await?;

    Ok(location_id)
}

async fn restore_load_projection(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    load_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE loads
         SET deleted_at = NULL,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(load_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "UPDATE load_legs
         SET deleted_at = NULL,
             updated_at = CURRENT_TIMESTAMP
         WHERE load_id = $1",
    )
    .bind(load_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn soft_delete_load_projection(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    load_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE loads
         SET deleted_at = COALESCE(deleted_at, CURRENT_TIMESTAMP),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(load_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "UPDATE load_legs
         SET deleted_at = COALESCE(deleted_at, CURRENT_TIMESTAMP),
             updated_at = CURRENT_TIMESTAMP
         WHERE load_id = $1",
    )
    .bind(load_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn fetch_load_number(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    load_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct LoadNumberRow {
        load_number: Option<String>,
    }

    sqlx::query_as::<_, LoadNumberRow>("SELECT load_number FROM loads WHERE id = $1 LIMIT 1")
        .bind(load_id)
        .fetch_optional(&mut **tx)
        .await
        .map(|maybe_row| maybe_row.and_then(|row| row.load_number))
}

async fn insert_handoff_event_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    handoff_id: i64,
    event_type: &str,
    performed_by: Option<&str>,
    source_module: Option<&str>,
    payload_snapshot: Option<&serde_json::Value>,
    result: Option<&str>,
) -> Result<(), sqlx::Error> {
    let payload_snapshot = payload_snapshot.and_then(|value| serde_json::to_string(value).ok());

    sqlx::query(
        "INSERT INTO stloads_handoff_events (
            handoff_id, event_type, performed_by, source_module, payload_snapshot, result,
            created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(handoff_id)
    .bind(event_type)
    .bind(performed_by)
    .bind(source_module)
    .bind(payload_snapshot.as_deref())
    .bind(result)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_external_refs_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    handoff_id: i64,
    external_refs: &[shared::TmsExternalRefRequest],
) -> Result<(), sqlx::Error> {
    for external_ref in external_refs {
        sqlx::query(
            "INSERT INTO stloads_external_refs (
                handoff_id, ref_type, ref_value, ref_source, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(handoff_id)
        .bind(external_ref.ref_type.as_str())
        .bind(external_ref.ref_value.as_str())
        .bind(external_ref.ref_source.as_deref())
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn insert_reconciliation_log_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    handoff_id: i64,
    action: &str,
    tms_status_from: Option<&str>,
    tms_status_to: Option<&str>,
    stloads_status_from: Option<&str>,
    stloads_status_to: Option<&str>,
    detail: &str,
    triggered_by: &str,
    webhook_payload: Option<&serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO stloads_reconciliation_log (
            handoff_id, action, tms_status_from, tms_status_to, stloads_status_from,
            stloads_status_to, detail, triggered_by, webhook_payload, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(handoff_id)
    .bind(action)
    .bind(tms_status_from)
    .bind(tms_status_to)
    .bind(stloads_status_from)
    .bind(stloads_status_to)
    .bind(detail)
    .bind(triggered_by)
    .bind(webhook_payload.cloned())
    .execute(&mut **tx)
    .await?;

    Ok(())
}

fn parse_required_datetime(value: &str) -> Result<NaiveDateTime, sqlx::Error> {
    parse_optional_datetime(Some(value)).ok_or(sqlx::Error::Protocol(
        format!("invalid datetime value: {}", value).into(),
    ))
}

fn parse_optional_datetime(value: Option<&str>) -> Option<NaiveDateTime> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }

    chrono::DateTime::parse_from_rfc3339(value)
        .map(|value| value.naive_utc())
        .ok()
        .or_else(|| chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
        .or_else(|| chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").ok())
        .or_else(|| chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f").ok())
        .or_else(|| chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").ok())
}

fn sanitize_load_token(value: &str) -> String {
    let sanitized = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();

    if sanitized.is_empty() {
        "LEGACY".into()
    } else {
        sanitized.to_uppercase()
    }
}

async fn find_inbound_event_by_idempotency(
    pool: &DbPool,
    tenant_id: &str,
    idempotency_key: &str,
    action: &str,
) -> Result<Option<AtmpContractApplyResult>, sqlx::Error> {
    let existing = sqlx::query_as::<_, (i64, Option<String>)>(
        "SELECT id, atmp_load_id
         FROM atmp_inbound_events
         WHERE tenant_id = $1 AND idempotency_key = $2
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await?;

    let Some((inbound_event_id, atmp_load_id)) = existing else {
        return Ok(None);
    };

    let posting_id = if let Some(atmp_load_id) = atmp_load_id {
        find_posting_id_for_atmp_load_pool(pool, tenant_id, &atmp_load_id).await?
    } else {
        None
    };

    Ok(Some(AtmpContractApplyResult {
        inbound_event_id: Some(inbound_event_id),
        posting_id,
        handoff_id: None,
        action: action.into(),
        status_label: "Replay".into(),
        replayed: true,
        message: "Idempotent ATMP contract replay returned the existing result.".into(),
    }))
}

async fn ensure_contract_tenant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at, updated_at)
         VALUES ($1, $2, $3, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (id) DO UPDATE SET updated_at = CURRENT_TIMESTAMP",
    )
    .bind(tenant_id)
    .bind(tenant_id)
    .bind(tenant_id.to_ascii_lowercase().replace('_', "-"))
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_atmp_inbound_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    envelope: &AtmpContractEnvelope,
    raw_payload: &Value,
    normalized_payload: &Value,
    payload_hash: &str,
    validation_status: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO atmp_inbound_events (
            tenant_id, event_id, correlation_id, idempotency_key, contract_version, action,
            atmp_load_id, atmp_leg_id, payload_hash, raw_payload, normalized_payload,
            validation_status, validation_errors, created_at, updated_at
         ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11,
            $12, '[]'::jsonb, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(&envelope.tenant_id)
    .bind(&envelope.event_id)
    .bind(&envelope.correlation_id)
    .bind(&envelope.idempotency_key)
    .bind(&envelope.contract_version)
    .bind(envelope.action.as_str())
    .bind(&envelope.atmp_load_id)
    .bind(envelope.atmp_leg_id.as_deref())
    .bind(payload_hash)
    .bind(raw_payload)
    .bind(normalized_payload)
    .bind(validation_status)
    .fetch_one(&mut **tx)
    .await
}

async fn insert_contract_payload(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    inbound_event_id: i64,
    contract_version: &str,
    payload_hash: &str,
    payload: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO atmp_contract_payloads (
            tenant_id, inbound_event_id, contract_version, direction, payload_hash, payload, created_at
         ) VALUES ($1, $2, $3, 'inbound', $4, $5, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(inbound_event_id)
    .bind(contract_version)
    .bind(payload_hash)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_atmp_outbound_event_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    event_type: &str,
    posting_id: Option<i64>,
    booking_award_id: Option<i64>,
    payload: &Value,
    correlation_id: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let event_id = format!("stloads-{}", uuid::Uuid::new_v4());
    let payload_hash = stable_json_hash(payload);
    let outbound_event_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO atmp_outbound_events (
            tenant_id, event_id, correlation_id, event_type, posting_id, booking_award_id,
            payload, payload_hash, status, next_attempt_at, created_at, updated_at
         ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, 'queued', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(&event_id)
    .bind(correlation_id)
    .bind(event_type)
    .bind(posting_id)
    .bind(booking_award_id)
    .bind(payload)
    .bind(&payload_hash)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO atmp_contract_payloads (
            tenant_id, outbound_event_id, contract_version, direction, payload_hash, payload, created_at
         ) VALUES ($1, $2, '2026-05-01', 'outbound', $3, $4, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(outbound_event_id)
    .bind(&payload_hash)
    .bind(payload)
    .execute(&mut **tx)
    .await?;

    Ok(outbound_event_id)
}

async fn insert_listing_lifecycle_outbound_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    envelope: &AtmpContractEnvelope,
    event_type: &str,
    posting_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    insert_atmp_outbound_event_tx(
        tx,
        &envelope.tenant_id,
        event_type,
        posting_id,
        None,
        &serde_json::json!({
            "event_type": event_type,
            "tenant_id": envelope.tenant_id,
            "atmp_load_id": envelope.atmp_load_id,
            "atmp_leg_id": envelope.atmp_leg_id,
            "posting_id": posting_id,
            "correlation_id": envelope.correlation_id,
            "source_action": envelope.action.as_str()
        }),
        Some(&envelope.correlation_id),
    )
    .await
}

fn listing_lifecycle_event_type(action: AtmpContractAction) -> &'static str {
    match action {
        AtmpContractAction::Withdraw => "listing_withdrawn",
        AtmpContractAction::Close => "listing_closed",
        AtmpContractAction::Cancel => "listing_canceled",
        AtmpContractAction::Publish => "listing_published",
        AtmpContractAction::Update => "listing_updated",
        AtmpContractAction::Status | AtmpContractAction::Document | AtmpContractAction::Finance => {
            "sync_error"
        }
    }
}

async fn upsert_atmp_posting(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    envelope: &AtmpContractEnvelope,
    payload_hash: &str,
    publish: bool,
) -> Result<i64, sqlx::Error> {
    let source_leg_id = normalized_source_leg_id(envelope);
    let title = envelope
        .payload
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&envelope.atmp_load_id);
    let freight_mode = string_payload(&envelope.payload, "freight_mode", "road");
    let equipment_type = optional_string_payload(&envelope.payload, "equipment_type");
    let commodity_description = optional_string_payload(&envelope.payload, "commodity_description");
    let weight = envelope.payload.get("weight").and_then(Value::as_f64);
    let weight_unit = string_payload(&envelope.payload, "weight_unit", "lbs");
    let status = if publish { "published" } else { "updated" };
    let visibility = if publish { "public" } else { "private" };
    let readiness = string_payload(&envelope.payload, "readiness", "ready");
    let is_hazardous = envelope
        .payload
        .get("is_hazardous")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let is_temperature_controlled = envelope
        .payload
        .get("is_temperature_controlled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let posting_number = format!("ATMP-{}-{}", envelope.atmp_load_id, source_leg_id);

    let posting_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO stloads_postings (
            tenant_id, source_system, source_load_id, source_leg_id, posting_number, title,
            freight_mode, equipment_type, commodity_description, weight, weight_unit,
            status, visibility, release_gate, readiness, is_hazardous, is_temperature_controlled,
            published_at, metadata, created_at, updated_at
         ) VALUES (
            $1, 'atmp', $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16,
            CASE WHEN $17 THEN CURRENT_TIMESTAMP ELSE NULL END,
            $18, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (tenant_id, posting_number) DO UPDATE SET
            title = EXCLUDED.title,
            freight_mode = EXCLUDED.freight_mode,
            equipment_type = EXCLUDED.equipment_type,
            commodity_description = EXCLUDED.commodity_description,
            weight = EXCLUDED.weight,
            weight_unit = EXCLUDED.weight_unit,
            status = EXCLUDED.status,
            visibility = EXCLUDED.visibility,
            release_gate = EXCLUDED.release_gate,
            readiness = EXCLUDED.readiness,
            is_hazardous = EXCLUDED.is_hazardous,
            is_temperature_controlled = EXCLUDED.is_temperature_controlled,
            published_at = COALESCE(stloads_postings.published_at, EXCLUDED.published_at),
            metadata = EXCLUDED.metadata,
            updated_at = CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(&envelope.tenant_id)
    .bind(&envelope.atmp_load_id)
    .bind(&source_leg_id)
    .bind(&posting_number)
    .bind(title)
    .bind(freight_mode)
    .bind(equipment_type.as_deref())
    .bind(commodity_description.as_deref())
    .bind(weight)
    .bind(weight_unit)
    .bind(status)
    .bind(visibility)
    .bind(envelope.release_gate.as_deref())
    .bind(readiness)
    .bind(is_hazardous)
    .bind(is_temperature_controlled)
    .bind(publish)
    .bind(serde_json::json!({
        "atmp_event_id": envelope.event_id,
        "atmp_correlation_id": envelope.correlation_id,
        "payload_hash": payload_hash,
    }))
    .fetch_one(&mut **tx)
    .await?;

    append_posting_stops_and_rate(tx, envelope, posting_id).await?;
    append_posting_version(tx, envelope, posting_id, payload_hash).await?;
    Ok(posting_id)
}

async fn append_posting_stops_and_rate(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    envelope: &AtmpContractEnvelope,
    posting_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM stloads_posting_stops WHERE tenant_id = $1 AND posting_id = $2")
        .bind(&envelope.tenant_id)
        .bind(posting_id)
        .execute(&mut **tx)
        .await?;

    for (sequence, stop_type, prefix) in
        [(1_i32, "pickup", "pickup"), (2_i32, "dropoff", "dropoff")]
    {
        sqlx::query(
            "INSERT INTO stloads_posting_stops (
                tenant_id, posting_id, stop_sequence, stop_type, address_line1, city,
                state_region, postal_code, country_code, window_start, window_end,
                instructions, created_at, updated_at
             ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10::timestamp, $11::timestamp,
                $12, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
             )",
        )
        .bind(&envelope.tenant_id)
        .bind(posting_id)
        .bind(sequence)
        .bind(stop_type)
        .bind(string_payload(
            &envelope.payload,
            &format!("{prefix}_address"),
            "",
        ))
        .bind(string_payload(
            &envelope.payload,
            &format!("{prefix}_city"),
            "",
        ))
        .bind(optional_string_payload(&envelope.payload, &format!("{prefix}_state")).as_deref())
        .bind(optional_string_payload(&envelope.payload, &format!("{prefix}_zip")).as_deref())
        .bind(string_payload(
            &envelope.payload,
            &format!("{prefix}_country"),
            "US",
        ))
        .bind(
            optional_string_payload(&envelope.payload, &format!("{prefix}_window_start"))
                .as_deref(),
        )
        .bind(
            optional_string_payload(&envelope.payload, &format!("{prefix}_window_end")).as_deref(),
        )
        .bind(
            optional_string_payload(&envelope.payload, &format!("{prefix}_instructions"))
                .as_deref(),
        )
        .execute(&mut **tx)
        .await?;
    }

    let amount = envelope
        .payload
        .get("board_rate")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let currency = string_payload(&envelope.payload, "rate_currency", "USD");
    sqlx::query(
        "INSERT INTO stloads_posting_rates (
            tenant_id, posting_id, rate_type, currency, amount, accessorial_policy,
            is_book_now, effective_at, created_at, updated_at
         ) VALUES ($1, $2, 'flat', $3, $4, $5, FALSE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(&envelope.tenant_id)
    .bind(posting_id)
    .bind(currency)
    .bind(amount)
    .bind(envelope.payload.get("accessorial_flags").cloned().unwrap_or_else(|| serde_json::json!({})))
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn append_posting_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    envelope: &AtmpContractEnvelope,
    posting_id: i64,
    payload_hash: &str,
) -> Result<i64, sqlx::Error> {
    let next_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number), 0) + 1
         FROM stloads_posting_versions
         WHERE tenant_id = $1 AND posting_id = $2",
    )
    .bind(&envelope.tenant_id)
    .bind(posting_id)
    .fetch_one(&mut **tx)
    .await?;

    let version_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO stloads_posting_versions (
            tenant_id, posting_id, version_number, change_reason, normalized_payload,
            payload_hash, created_at
         ) VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(&envelope.tenant_id)
    .bind(posting_id)
    .bind(next_version)
    .bind(envelope.action.as_str())
    .bind(envelope.normalized_payload())
    .bind(payload_hash)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        "UPDATE stloads_postings
         SET active_version = $1,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(version_id)
    .bind(posting_id)
    .execute(&mut **tx)
    .await?;

    Ok(version_id)
}

async fn transition_atmp_posting(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    envelope: &AtmpContractEnvelope,
) -> Result<Option<i64>, sqlx::Error> {
    let Some(status) = envelope.action.terminal_status() else {
        return Ok(None);
    };
    let source_leg_id = normalized_source_leg_id(envelope);
    let timestamp_column = match envelope.action {
        AtmpContractAction::Close => "closed_at",
        AtmpContractAction::Cancel => "canceled_at",
        _ => "withdrawn_at",
    };
    let sql = format!(
        "UPDATE stloads_postings
         SET status = $1,
             visibility = 'private',
             {timestamp_column} = CURRENT_TIMESTAMP,
             metadata = COALESCE(metadata, '{{}}'::jsonb) || $2::jsonb,
             updated_at = CURRENT_TIMESTAMP
         WHERE tenant_id = $3 AND source_system = 'atmp' AND source_load_id = $4 AND source_leg_id = $5
         RETURNING id"
    );

    let posting_id = sqlx::query_scalar::<_, i64>(&sql)
        .bind(status)
        .bind(serde_json::json!({
            "terminal_event_id": envelope.event_id,
            "terminal_action": envelope.action.as_str(),
        }))
        .bind(&envelope.tenant_id)
        .bind(&envelope.atmp_load_id)
        .bind(&source_leg_id)
        .fetch_optional(&mut **tx)
        .await?;

    if let Some(posting_id) = posting_id {
        let hash = stable_json_hash(&envelope.normalized_payload());
        append_posting_version(tx, envelope, posting_id, &hash).await?;
    }

    Ok(posting_id)
}

async fn find_posting_id_for_atmp_load(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    atmp_load_id: &str,
    atmp_leg_id: Option<&str>,
) -> Result<Option<i64>, sqlx::Error> {
    let source_leg_id = atmp_leg_id.unwrap_or("primary");
    sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM stloads_postings
         WHERE tenant_id = $1 AND source_system = 'atmp' AND source_load_id = $2 AND source_leg_id = $3
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(atmp_load_id)
    .bind(source_leg_id)
    .fetch_optional(&mut **tx)
    .await
}

async fn find_posting_id_for_atmp_load_pool(
    pool: &DbPool,
    tenant_id: &str,
    atmp_load_id: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM stloads_postings
         WHERE tenant_id = $1 AND source_system = 'atmp' AND source_load_id = $2
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(atmp_load_id)
    .fetch_optional(pool)
    .await
}

fn normalized_source_leg_id(envelope: &AtmpContractEnvelope) -> String {
    envelope
        .atmp_leg_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("primary")
        .to_string()
}

fn string_payload(payload: &Value, key: &str, fallback: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn optional_string_payload(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
