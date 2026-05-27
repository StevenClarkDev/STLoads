use chrono::{NaiveDateTime, Utc};
use domain::dispatch::LegacyLoadLegStatusCode;
use domain::tracking::{Coordinate, tracking_module_contract};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LegLocationRecord {
    pub id: i64,
    pub leg_id: i64,
    pub lat: f64,
    pub lng: f64,
    pub recorded_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl LegLocationRecord {
    pub fn coordinate(&self) -> Coordinate {
        Coordinate {
            lat: self.lat,
            lng: self.lng,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LegEventRecord {
    pub id: i64,
    pub leg_id: i64,
    pub r#type: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LegDocumentRecord {
    pub id: i64,
    pub leg_id: i64,
    pub r#type: String,
    pub path: String,
    pub meta: Option<Value>,
    pub current_version: i32,
    pub version_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LegDocumentScopeRecord {
    pub document_id: i64,
    pub leg_id: i64,
    pub load_id: i64,
    pub load_owner_user_id: Option<i64>,
    pub booked_carrier_id: Option<i64>,
    pub uploaded_by_user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionLegRecord {
    pub leg_id: i64,
    pub load_id: i64,
    pub load_number: Option<String>,
    pub load_title: String,
    pub leg_code: Option<String>,
    pub pickup_location_name: Option<String>,
    pub delivery_location_name: Option<String>,
    pub pickup_latitude: Option<f64>,
    pub pickup_longitude: Option<f64>,
    pub delivery_latitude: Option<f64>,
    pub delivery_longitude: Option<f64>,
    pub pickup_date: Option<NaiveDateTime>,
    pub delivery_date: Option<NaiveDateTime>,
    pub status_id: i16,
    pub booked_carrier_id: Option<i64>,
    pub booked_carrier_name: Option<String>,
    pub load_owner_user_id: Option<i64>,
    pub load_owner_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionTrackingConsentRecord {
    pub id: i64,
    pub leg_id: i64,
    pub user_id: i64,
    pub consent_text: String,
    pub consented_at: NaiveDateTime,
    pub retention_days: i32,
    pub customer_visible_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionOfflineSubmissionRecord {
    pub id: i64,
    pub leg_id: i64,
    pub user_id: Option<i64>,
    pub submission_type: String,
    pub client_submission_id: Option<String>,
    pub processing_status: String,
    pub reconciliation_note: Option<String>,
    pub received_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionCloseoutReadinessRecord {
    pub leg_id: i64,
    pub closeout_status: Option<String>,
    pub pod_review_status: Option<String>,
    pub export_path: Option<String>,
    pub delivery_pod_count: i64,
    pub open_exception_count: i64,
    pub pending_offline_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionFinanceExceptionSummaryRecord {
    pub exception_type: String,
    pub status: String,
    pub count: i64,
    pub amount_cents: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionCustomerTrackingLinkRecord {
    pub leg_id: i64,
    pub share_token: String,
    pub expires_at: NaiveDateTime,
    pub visibility_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionRoutePlanRecord {
    pub provider_key: String,
    pub distance_miles: Option<f64>,
    pub duration_minutes: Option<i32>,
    pub truck_safe: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionTelematicsSummaryRecord {
    pub provider_key: String,
    pub status: String,
    pub last_ping_at: Option<NaiveDateTime>,
    pub fallback_behavior: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionNoteRecord {
    pub id: i64,
    pub load_id: i64,
    pub admin_id: Option<i64>,
    pub actor_name: Option<String>,
    pub status: i16,
    pub remarks: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLegDocumentParams {
    pub document_name: String,
    pub document_type: String,
    pub file_path: String,
    pub storage_provider: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

pub async fn list_tracking_points_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<LegLocationRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegLocationRecord>(
        "SELECT id, leg_id, lat::double precision AS lat, lng::double precision AS lng, recorded_at, created_at, updated_at
         FROM leg_locations
         WHERE leg_id = $1
         ORDER BY recorded_at DESC, id DESC",
    )
    .bind(leg_id)
    .fetch_all(pool)
    .await
}

pub async fn latest_tracking_point_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<LegLocationRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegLocationRecord>(
        "SELECT id, leg_id, lat::double precision AS lat, lng::double precision AS lng, recorded_at, created_at, updated_at
         FROM leg_locations
         WHERE leg_id = $1
         ORDER BY recorded_at DESC, id DESC
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_leg_events(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<LegEventRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegEventRecord>(
        "SELECT id, leg_id, type, created_at, updated_at
         FROM leg_events
         WHERE leg_id = $1
         ORDER BY created_at ASC, id ASC",
    )
    .bind(leg_id)
    .fetch_all(pool)
    .await
}

pub async fn list_leg_documents(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<LegDocumentRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegDocumentRecord>(
        "SELECT id, leg_id, type, path, meta,
                COALESCE((SELECT MAX(version_number) FROM leg_document_versions WHERE document_id = leg_documents.id), 1)::int AS current_version,
                COALESCE((SELECT COUNT(*) FROM leg_document_versions WHERE document_id = leg_documents.id), 1)::bigint AS version_count,
                created_at, updated_at
         FROM leg_documents
         WHERE leg_id = $1
         ORDER BY id DESC",
    )
    .bind(leg_id)
    .fetch_all(pool)
    .await
}

pub async fn find_leg_document_by_id(
    pool: &DbPool,
    document_id: i64,
) -> Result<Option<LegDocumentRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegDocumentRecord>(
        "SELECT id, leg_id, type, path, meta,
                COALESCE((SELECT MAX(version_number) FROM leg_document_versions WHERE document_id = leg_documents.id), 1)::int AS current_version,
                COALESCE((SELECT COUNT(*) FROM leg_document_versions WHERE document_id = leg_documents.id), 1)::bigint AS version_count,
                created_at, updated_at
         FROM leg_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_leg_document_scope(
    pool: &DbPool,
    document_id: i64,
) -> Result<Option<LegDocumentScopeRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegDocumentScopeRecord>(
        r#"
        SELECT
            document.id AS document_id,
            document.leg_id,
            leg.load_id,
            loads_record.user_id AS load_owner_user_id,
            leg.booked_carrier_id,
            NULLIF(document.meta ->> 'uploaded_by', '')::bigint AS uploaded_by_user_id
        FROM leg_documents document
        INNER JOIN load_legs leg ON leg.id = document.leg_id AND leg.deleted_at IS NULL
        INNER JOIN loads loads_record ON loads_record.id = leg.load_id AND loads_record.deleted_at IS NULL
        WHERE document.id = $1
        LIMIT 1
        "#,
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_execution_leg_by_id(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<ExecutionLegRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionLegRecord>(
        "SELECT
            ll.id AS leg_id,
            ll.load_id,
            l.load_number,
            l.title AS load_title,
            ll.leg_code,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
            pickup.latitude::double precision AS pickup_latitude,
            pickup.longitude::double precision AS pickup_longitude,
            delivery.latitude::double precision AS delivery_latitude,
            delivery.longitude::double precision AS delivery_longitude,
            ll.pickup_date,
            ll.delivery_date,
            ll.status_id,
            ll.booked_carrier_id,
            carrier.name AS booked_carrier_name,
            l.user_id AS load_owner_user_id,
            owner.name AS load_owner_name
         FROM load_legs ll
         INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
         LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
         LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id
         LEFT JOIN users carrier ON carrier.id = ll.booked_carrier_id
         LEFT JOIN users owner ON owner.id = l.user_id
         WHERE ll.id = $1 AND ll.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn active_tracking_consent_for_leg_user(
    pool: &DbPool,
    leg_id: i64,
    user_id: i64,
) -> Result<Option<ExecutionTrackingConsentRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionTrackingConsentRecord>(
        "SELECT id, leg_id, user_id, consent_text, consented_at, retention_days, customer_visible_scope
         FROM execution_tracking_consents
         WHERE leg_id = $1 AND user_id = $2 AND revoked_at IS NULL
         ORDER BY consented_at DESC, id DESC
         LIMIT 1",
    )
    .bind(leg_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn record_tracking_consent(
    pool: &DbPool,
    leg_id: i64,
    user_id: i64,
    consent_text: &str,
) -> Result<ExecutionTrackingConsentRecord, sqlx::Error> {
    sqlx::query_as::<_, ExecutionTrackingConsentRecord>(
        "INSERT INTO execution_tracking_consents (
             leg_id, user_id, consent_text, retention_days, customer_visible_scope,
             consented_at, created_at, updated_at
         )
         VALUES ($1, $2, $3, 90, 'latest_location_and_status', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (leg_id, user_id) WHERE revoked_at IS NULL
         DO UPDATE SET
             consent_text = EXCLUDED.consent_text,
             consented_at = CURRENT_TIMESTAMP,
             retention_days = EXCLUDED.retention_days,
             customer_visible_scope = EXCLUDED.customer_visible_scope,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, leg_id, user_id, consent_text, consented_at, retention_days, customer_visible_scope",
    )
    .bind(leg_id)
    .bind(user_id)
    .bind(consent_text.trim())
    .fetch_one(pool)
    .await
}

pub async fn record_execution_offline_submission(
    pool: &DbPool,
    leg_id: i64,
    user_id: Option<i64>,
    submission_type: &str,
    client_submission_id: &str,
    payload: &Value,
    captured_at: Option<NaiveDateTime>,
) -> Result<ExecutionOfflineSubmissionRecord, sqlx::Error> {
    sqlx::query_as::<_, ExecutionOfflineSubmissionRecord>(
        "INSERT INTO execution_offline_submissions (
             leg_id, user_id, submission_type, client_submission_id, payload, captured_at,
             received_at, processed_at, processing_status, reconciliation_note, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 'processed', 'Replayed through Rust execution offline sync.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (leg_id, user_id, client_submission_id) WHERE client_submission_id IS NOT NULL
         DO UPDATE SET
             payload = EXCLUDED.payload,
             processing_status = 'duplicate',
             reconciliation_note = 'Duplicate offline submission received; original replay was preserved.',
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, leg_id, user_id, submission_type, client_submission_id, processing_status, reconciliation_note, received_at",
    )
    .bind(leg_id)
    .bind(user_id)
    .bind(submission_type.trim())
    .bind(client_submission_id.trim())
    .bind(payload)
    .bind(captured_at)
    .fetch_one(pool)
    .await
}

pub async fn list_execution_offline_submissions(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<ExecutionOfflineSubmissionRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionOfflineSubmissionRecord>(
        "SELECT id, leg_id, user_id, submission_type, client_submission_id, processing_status, reconciliation_note, received_at
         FROM execution_offline_submissions
         WHERE leg_id = $1
         ORDER BY received_at DESC, id DESC
         LIMIT 20",
    )
    .bind(leg_id)
    .fetch_all(pool)
    .await
}

pub async fn mark_execution_offline_submission_failed(
    pool: &DbPool,
    submission_id: i64,
    note: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE execution_offline_submissions
         SET processing_status = 'failed',
             reconciliation_note = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(submission_id)
    .bind(note.trim())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_execution_offline_history_note(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    status_id: i16,
    note: &str,
) -> Result<(), sqlx::Error> {
    let Some(load_id) = sqlx::query_scalar::<_, i64>(
        "SELECT load_id FROM load_legs WHERE id = $1 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(actor_user_id)
    .bind(status_id)
    .bind(format!(
        "Rust offline replay note for leg #{}: {}",
        leg_id,
        note.trim()
    ))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn execution_closeout_readiness(
    pool: &DbPool,
    leg_id: i64,
) -> Result<ExecutionCloseoutReadinessRecord, sqlx::Error> {
    sqlx::query_as::<_, ExecutionCloseoutReadinessRecord>(
        "SELECT
            $1::bigint AS leg_id,
            package.status AS closeout_status,
            package.pod_review_status,
            package.export_path,
            COALESCE((SELECT COUNT(*) FROM leg_documents WHERE leg_id = $1 AND type = 'delivery_pod'), 0)::bigint AS delivery_pod_count,
            COALESCE((SELECT COUNT(*) FROM execution_finance_exceptions WHERE leg_id = $1 AND status IN ('pending', 'disputed', 'review')), 0)::bigint AS open_exception_count,
            COALESCE((SELECT COUNT(*) FROM execution_offline_submissions WHERE leg_id = $1 AND processing_status IN ('received', 'pending', 'failed')), 0)::bigint AS pending_offline_count
         FROM (SELECT $1::bigint AS leg_id) seed
         LEFT JOIN execution_closeout_packages package ON package.leg_id = seed.leg_id
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_one(pool)
    .await
}

pub async fn list_execution_finance_exception_summaries(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<ExecutionFinanceExceptionSummaryRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionFinanceExceptionSummaryRecord>(
        "SELECT exception_type, status, COUNT(*)::bigint AS count, SUM(amount_cents)::bigint AS amount_cents
         FROM execution_finance_exceptions
         WHERE leg_id = $1
         GROUP BY exception_type, status
         ORDER BY status ASC, exception_type ASC",
    )
    .bind(leg_id)
    .fetch_all(pool)
    .await
}

pub async fn active_customer_tracking_link(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<ExecutionCustomerTrackingLinkRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionCustomerTrackingLinkRecord>(
        "SELECT leg_id, share_token, expires_at, visibility_scope
         FROM execution_customer_tracking_links
         WHERE leg_id = $1 AND revoked_at IS NULL AND expires_at > CURRENT_TIMESTAMP
         ORDER BY expires_at DESC, id DESC
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn active_customer_tracking_link_by_token(
    pool: &DbPool,
    share_token: &str,
) -> Result<Option<ExecutionCustomerTrackingLinkRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionCustomerTrackingLinkRecord>(
        "SELECT leg_id, share_token, expires_at, visibility_scope
         FROM execution_customer_tracking_links
         WHERE share_token = $1 AND revoked_at IS NULL AND expires_at > CURRENT_TIMESTAMP
         LIMIT 1",
    )
    .bind(share_token.trim())
    .fetch_optional(pool)
    .await
}

pub async fn create_customer_tracking_link(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    share_token: &str,
    visibility_scope: &str,
    expires_at: NaiveDateTime,
    rotate_existing: bool,
) -> Result<Option<ExecutionCustomerTrackingLinkRecord>, sqlx::Error> {
    #[derive(Debug, FromRow)]
    struct LegLoadRow {
        load_id: i64,
        status_id: i16,
    }

    let mut tx = pool.begin().await?;
    let Some(leg_row) = sqlx::query_as::<_, LegLoadRow>(
        "SELECT load_id, status_id
         FROM load_legs
         WHERE id = $1 AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    if rotate_existing {
        sqlx::query(
            "UPDATE execution_customer_tracking_links
             SET revoked_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE leg_id = $1
               AND revoked_at IS NULL
               AND expires_at > CURRENT_TIMESTAMP",
        )
        .bind(leg_id)
        .execute(&mut *tx)
        .await?;
    }

    let link = sqlx::query_as::<_, ExecutionCustomerTrackingLinkRecord>(
        "INSERT INTO execution_customer_tracking_links (
             leg_id, share_token, visibility_scope, expires_at, created_by_user_id, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING leg_id, share_token, expires_at, visibility_scope",
    )
    .bind(leg_id)
    .bind(share_token.trim())
    .bind(visibility_scope.trim())
    .bind(expires_at)
    .bind(actor_user_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_row.load_id)
    .bind(actor_user_id)
    .bind(leg_row.status_id)
    .bind(format!(
        "Rust customer tracking link created for leg #{} with scope {}.",
        leg_id,
        visibility_scope.trim()
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(link))
}

pub async fn revoke_customer_tracking_links(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    reason: Option<&str>,
) -> Result<u64, sqlx::Error> {
    #[derive(Debug, FromRow)]
    struct LegLoadRow {
        load_id: i64,
        status_id: i16,
    }

    let mut tx = pool.begin().await?;
    let Some(leg_row) = sqlx::query_as::<_, LegLoadRow>(
        "SELECT load_id, status_id
         FROM load_legs
         WHERE id = $1 AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(0);
    };

    let result = sqlx::query(
        "UPDATE execution_customer_tracking_links
         SET revoked_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE leg_id = $1
           AND revoked_at IS NULL
           AND expires_at > CURRENT_TIMESTAMP",
    )
    .bind(leg_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() > 0 {
        let clean_reason = reason
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("No reason supplied.");
        sqlx::query(
            "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
             VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(leg_row.load_id)
        .bind(actor_user_id)
        .bind(leg_row.status_id)
        .bind(format!(
            "Rust customer tracking link revoked for leg #{}: {}",
            leg_id, clean_reason
        ))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(result.rows_affected())
}

pub async fn current_route_plan(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<ExecutionRoutePlanRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionRoutePlanRecord>(
        "SELECT provider_key, distance_miles::double precision AS distance_miles, duration_minutes, truck_safe, status
         FROM execution_route_plans
         WHERE leg_id = $1
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn current_telematics_summary(
    pool: &DbPool,
    carrier_user_id: Option<i64>,
) -> Result<Option<ExecutionTelematicsSummaryRecord>, sqlx::Error> {
    let Some(carrier_user_id) = carrier_user_id else {
        return Ok(None);
    };

    sqlx::query_as::<_, ExecutionTelematicsSummaryRecord>(
        "SELECT provider_key, status, last_ping_at, fallback_behavior
         FROM telematics_connections
         WHERE carrier_user_id = $1
         ORDER BY updated_at DESC, id DESC
         LIMIT 1",
    )
    .bind(carrier_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn approve_execution_closeout_package(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    pod_review_status: &str,
    export_path: Option<&str>,
    note: Option<&str>,
) -> Result<ExecutionCloseoutReadinessRecord, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let status = if matches!(pod_review_status, "approved" | "accepted") {
        "approved"
    } else {
        "review"
    };
    let export_path = export_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("/execution/legs/{}/closeout-package", leg_id));

    sqlx::query(
        "INSERT INTO execution_closeout_packages (
             leg_id, status, pod_review_status, required_documents, approved_by_user_id,
             approved_at, export_path, created_at, updated_at
         )
         VALUES ($1, $2, $3, '[\"delivery_pod\"]'::jsonb, $4,
             CASE WHEN $2 = 'approved' THEN CURRENT_TIMESTAMP ELSE NULL END,
             $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (leg_id)
         DO UPDATE SET
             status = EXCLUDED.status,
             pod_review_status = EXCLUDED.pod_review_status,
             approved_by_user_id = EXCLUDED.approved_by_user_id,
             approved_at = EXCLUDED.approved_at,
             export_path = EXCLUDED.export_path,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(leg_id)
    .bind(status)
    .bind(pod_review_status)
    .bind(actor_user_id)
    .bind(&export_path)
    .execute(&mut *tx)
    .await?;

    if let Some(note) = note.map(str::trim).filter(|value| !value.is_empty())
        && let Some((load_id, status_id)) = sqlx::query_as::<_, (i64, i16)>(
            "SELECT load_id, status_id FROM load_legs WHERE id = $1 AND deleted_at IS NULL LIMIT 1",
        )
        .bind(leg_id)
        .fetch_optional(&mut *tx)
        .await?
    {
        sqlx::query(
            "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(load_id)
        .bind(actor_user_id)
        .bind(status_id)
        .bind(format!(
            "Rust closeout review for leg #{}: {}",
            leg_id, note
        ))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    execution_closeout_readiness(pool, leg_id).await
}

// Finance exceptions need a complete operational context at creation time so the
// closeout desk can distinguish amount, visibility, evidence, and actor.
#[allow(clippy::too_many_arguments)]
pub async fn create_execution_finance_exception(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    exception_type: &str,
    status: &str,
    amount_cents: Option<i64>,
    visibility: &str,
    description: &str,
    evidence_document_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO execution_finance_exceptions (
             leg_id, exception_type, amount_cents, status, visibility, description,
             evidence_document_id, created_by_user_id,
             resolved_by_user_id, resolved_at, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
             CASE WHEN $4 IN ('approved', 'rejected', 'resolved') THEN $8 ELSE NULL END,
             CASE WHEN $4 IN ('approved', 'rejected', 'resolved') THEN CURRENT_TIMESTAMP ELSE NULL END,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(leg_id)
    .bind(exception_type.trim())
    .bind(amount_cents)
    .bind(status.trim())
    .bind(visibility.trim())
    .bind(description.trim())
    .bind(evidence_document_id)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await
}

pub async fn decide_execution_finance_exceptions(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    exception_type: &str,
    status: &str,
    resolution_note: Option<&str>,
) -> Result<u64, sqlx::Error> {
    #[derive(Debug, FromRow)]
    struct LegLoadRow {
        load_id: i64,
        status_id: i16,
    }

    let mut tx = pool.begin().await?;
    let Some(leg_row) = sqlx::query_as::<_, LegLoadRow>(
        "SELECT load_id, status_id
         FROM load_legs
         WHERE id = $1 AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(0);
    };

    let result = sqlx::query(
        "UPDATE execution_finance_exceptions
         SET status = $3,
             resolved_by_user_id = CASE WHEN $3 IN ('approved', 'rejected', 'resolved') THEN $4 ELSE resolved_by_user_id END,
             resolved_at = CASE WHEN $3 IN ('approved', 'rejected', 'resolved') THEN CURRENT_TIMESTAMP ELSE resolved_at END,
             updated_at = CURRENT_TIMESTAMP
         WHERE leg_id = $1
           AND exception_type = $2
           AND status IN ('pending', 'disputed', 'review')",
    )
    .bind(leg_id)
    .bind(exception_type.trim())
    .bind(status.trim())
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() > 0 {
        let note = resolution_note
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("No resolution note supplied.");
        sqlx::query(
            "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
             VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(leg_row.load_id)
        .bind(actor_user_id)
        .bind(leg_row.status_id)
        .bind(format!(
            "Rust finance exception decision for leg #{} type {} -> {}. Invoice/settlement blocker reviewed; support timeline note: {}",
            leg_id,
            exception_type.trim(),
            status.trim(),
            note
        ))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(result.rows_affected())
}

pub async fn upsert_telematics_connection(
    pool: &DbPool,
    carrier_user_id: Option<i64>,
    provider_key: &str,
    status: &str,
    fallback_behavior: &str,
) -> Result<(), sqlx::Error> {
    let Some(carrier_user_id) = carrier_user_id else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO telematics_connections (
             carrier_user_id, provider_key, status, fallback_behavior, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (carrier_user_id, provider_key) WHERE carrier_user_id IS NOT NULL
         DO UPDATE SET
             status = EXCLUDED.status,
             fallback_behavior = EXCLUDED.fallback_behavior,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(carrier_user_id)
    .bind(provider_key.trim())
    .bind(status.trim())
    .bind(fallback_behavior.trim())
    .execute(pool)
    .await?;

    Ok(())
}

// Telematics pings mirror provider payload fields; the explicit signature keeps
// optional vehicle, trailer, HOS, and actor fields visible to callers.
#[allow(clippy::too_many_arguments)]
pub async fn record_telematics_execution_ping(
    pool: &DbPool,
    leg_id: i64,
    provider_key: &str,
    lat: Option<f64>,
    lng: Option<f64>,
    recorded_at: Option<NaiveDateTime>,
    event_type: Option<&str>,
    hos_status: Option<&str>,
    truck_id: Option<&str>,
    trailer_id: Option<&str>,
    actor_user_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    #[derive(Debug, FromRow)]
    struct LegLoadRow {
        load_id: i64,
        status_id: i16,
    }

    let mut tx = pool.begin().await?;
    let Some(leg_row) = sqlx::query_as::<_, LegLoadRow>(
        "SELECT load_id, status_id
         FROM load_legs
         WHERE id = $1 AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(());
    };

    if let (Some(lat), Some(lng)) = (lat, lng) {
        sqlx::query(
            "INSERT INTO leg_locations (leg_id, lat, lng, recorded_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(leg_id)
        .bind(lat)
        .bind(lng)
        .bind(recorded_at.unwrap_or_else(|| Utc::now().naive_utc()))
        .execute(&mut *tx)
        .await?;
    }

    let clean_event = event_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("telematics_ping");
    sqlx::query(
        "INSERT INTO leg_events (leg_id, type, created_at, updated_at)
         VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .bind(clean_event)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_row.load_id)
    .bind(actor_user_id)
    .bind(leg_row.status_id)
    .bind(format!(
        "Rust telematics ping for leg #{} from {}: event={}, HOS={}, truck={}, trailer={}.",
        leg_id,
        provider_key.trim(),
        clean_event,
        hos_status.unwrap_or("unknown"),
        truck_id.unwrap_or("unknown"),
        trailer_id.unwrap_or("unknown")
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

// Route plans are written as one atomic provider result, so the route metrics
// and constraints stay together at the call site.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_execution_route_plan(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    provider_key: &str,
    distance_miles: Option<f64>,
    duration_minutes: Option<i32>,
    truck_safe: bool,
    status: &str,
    constraints: &Value,
) -> Result<(), sqlx::Error> {
    #[derive(Debug, FromRow)]
    struct LegLoadRow {
        load_id: i64,
        status_id: i16,
    }

    let mut tx = pool.begin().await?;
    let leg_row = sqlx::query_as::<_, LegLoadRow>(
        "SELECT load_id, status_id
         FROM load_legs
         WHERE id = $1 AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO execution_route_plans (
             leg_id, provider_key, distance_miles, duration_minutes, truck_safe,
             constraints, status, calculated_at, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (leg_id)
         DO UPDATE SET
             provider_key = EXCLUDED.provider_key,
             distance_miles = EXCLUDED.distance_miles,
             duration_minutes = EXCLUDED.duration_minutes,
             truck_safe = EXCLUDED.truck_safe,
             constraints = EXCLUDED.constraints,
             status = EXCLUDED.status,
             calculated_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(leg_id)
    .bind(provider_key.trim())
    .bind(distance_miles)
    .bind(duration_minutes)
    .bind(truck_safe)
    .bind(constraints)
    .bind(status.trim())
    .execute(&mut *tx)
    .await?;

    if let Some(leg_row) = leg_row {
        sqlx::query(
            "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
             VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(leg_row.load_id)
        .bind(actor_user_id)
        .bind(leg_row.status_id)
        .bind(format!(
            "Rust route plan for leg #{}: provider={}, miles={}, minutes={}, truck_safe={}, constraints={}. Pricing, ETA, exception, and settlement readiness should use this source.",
            leg_id,
            provider_key.trim(),
            distance_miles
                .map(|value| format!("{:.1}", value))
                .unwrap_or_else(|| "unknown".into()),
            duration_minutes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".into()),
            truck_safe,
            constraints
        ))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list_execution_note_history_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<ExecutionNoteRecord>, sqlx::Error> {
    let leg_marker = format!("leg #{}", leg_id);
    sqlx::query_as::<_, ExecutionNoteRecord>(
        "SELECT
            history.id,
            history.load_id,
            history.admin_id,
            users_record.name AS actor_name,
            history.status,
            history.remarks,
            history.created_at
         FROM load_history history
         INNER JOIN load_legs leg ON leg.load_id = history.load_id AND leg.deleted_at IS NULL
         LEFT JOIN users users_record ON users_record.id = history.admin_id
         WHERE leg.id = $1
           AND history.remarks IS NOT NULL
           AND history.remarks ILIKE $2
         ORDER BY history.id DESC
         LIMIT 12",
    )
    .bind(leg_id)
    .bind(format!("%{}%", leg_marker))
    .fetch_all(pool)
    .await
}

pub async fn advance_leg_execution(
    pool: &DbPool,
    leg_id: i64,
    next_status: LegacyLoadLegStatusCode,
    event_type: &str,
    actor_user_id: Option<i64>,
    remarks: Option<&str>,
) -> Result<Option<ExecutionLegRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(existing) = sqlx::query_as::<_, ExecutionLegRecord>(
        "SELECT
            ll.id AS leg_id,
            ll.load_id,
            l.load_number,
            l.title AS load_title,
            ll.leg_code,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
            pickup.latitude::double precision AS pickup_latitude,
            pickup.longitude::double precision AS pickup_longitude,
            delivery.latitude::double precision AS delivery_latitude,
            delivery.longitude::double precision AS delivery_longitude,
            ll.pickup_date,
            ll.delivery_date,
            ll.status_id,
            ll.booked_carrier_id,
            carrier.name AS booked_carrier_name,
            l.user_id AS load_owner_user_id,
            owner.name AS load_owner_name
         FROM load_legs ll
         INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
         LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
         LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id
         LEFT JOIN users carrier ON carrier.id = ll.booked_carrier_id
         LEFT JOIN users owner ON owner.id = l.user_id
         WHERE ll.id = $1 AND ll.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "UPDATE load_legs
         SET status_id = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(leg_id)
    .bind(next_status.legacy_code())
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO leg_events (leg_id, type, created_at, updated_at)
         VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .bind(event_type)
    .execute(&mut *tx)
    .await?;

    let history_remarks = remarks
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("Rust execution note for leg #{}: {}", leg_id, value))
        .unwrap_or_else(|| format!("Rust execution action on leg #{}: {}", leg_id, event_type));

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(existing.load_id)
    .bind(actor_user_id)
    .bind(next_status.legacy_code())
    .bind(history_remarks)
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, ExecutionLegRecord>(
        "SELECT
            ll.id AS leg_id,
            ll.load_id,
            l.load_number,
            l.title AS load_title,
            ll.leg_code,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
            pickup.latitude::double precision AS pickup_latitude,
            pickup.longitude::double precision AS pickup_longitude,
            delivery.latitude::double precision AS delivery_latitude,
            delivery.longitude::double precision AS delivery_longitude,
            ll.pickup_date,
            ll.delivery_date,
            ll.status_id,
            ll.booked_carrier_id,
            carrier.name AS booked_carrier_name,
            l.user_id AS load_owner_user_id,
            owner.name AS load_owner_name
         FROM load_legs ll
         INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
         LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
         LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id
         LEFT JOIN users carrier ON carrier.id = ll.booked_carrier_id
         LEFT JOIN users owner ON owner.id = l.user_id
         WHERE ll.id = $1 AND ll.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}

pub async fn create_leg_document(
    pool: &DbPool,
    leg_id: i64,
    params: &CreateLegDocumentParams,
    actor_user_id: Option<i64>,
) -> Result<Option<LegDocumentRecord>, sqlx::Error> {
    #[derive(Debug, FromRow)]
    struct LegLoadRow {
        load_id: i64,
        status_id: i16,
    }

    let mut tx = pool.begin().await?;
    let Some(leg_row) = sqlx::query_as::<_, LegLoadRow>(
        "SELECT load_id, status_id
         FROM load_legs
         WHERE id = $1 AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let metadata = leg_document_meta(params, actor_user_id);

    let created = sqlx::query_as::<_, LegDocumentRecord>(
        "INSERT INTO leg_documents (leg_id, type, path, meta, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, leg_id, type, path, meta,
             1::int AS current_version,
             1::bigint AS version_count,
             created_at, updated_at",
    )
    .bind(leg_id)
    .bind(&params.document_type)
    .bind(&params.file_path)
    .bind(metadata)
    .fetch_one(&mut *tx)
    .await?;

    insert_leg_document_version(&mut tx, &created, 1, Some("initial upload")).await?;

    sqlx::query(
        "INSERT INTO leg_events (leg_id, type, created_at, updated_at)
         VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .bind("document_uploaded")
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_row.load_id)
    .bind(actor_user_id)
    .bind(leg_row.status_id)
    .bind(format!(
        "Rust execution added {} to leg #{}",
        params.document_name, leg_id
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(created))
}

async fn insert_leg_document_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document: &LegDocumentRecord,
    version_number: i32,
    replacement_reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO leg_document_versions (
            document_id, version_number, document_type, file_path, meta, uploaded_by_user_id,
            original_name, mime_type, file_size, replacement_reason, created_at
         )
         VALUES (
            $1, $2, $3, $4, $5,
            NULLIF($5 ->> 'uploaded_by', '')::bigint,
            NULLIF($5 ->> 'original_name', ''),
            NULLIF($5 ->> 'mime_type', ''),
            NULLIF($5 ->> 'file_size', '')::bigint,
            $6,
            CURRENT_TIMESTAMP
         )
         ON CONFLICT (document_id, version_number) DO NOTHING",
    )
    .bind(document.id)
    .bind(version_number)
    .bind(&document.r#type)
    .bind(&document.path)
    .bind(document.meta.as_ref())
    .bind(replacement_reason)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn create_tracking_point(
    pool: &DbPool,
    leg_id: i64,
    lat: f64,
    lng: f64,
    recorded_at: Option<NaiveDateTime>,
) -> Result<LegLocationRecord, sqlx::Error> {
    let recorded_at = recorded_at.unwrap_or_else(|| Utc::now().naive_utc());
    let point_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO leg_locations (leg_id, lat, lng, recorded_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(leg_id)
    .bind(lat)
    .bind(lng)
    .bind(recorded_at)
    .fetch_one(pool)
    .await?;

    sqlx::query_as::<_, LegLocationRecord>(
        "SELECT id, leg_id, lat::double precision AS lat, lng::double precision AS lng, recorded_at, created_at, updated_at
         FROM leg_locations
         WHERE id = $1
         LIMIT 1",
    )
    .bind(point_id)
    .fetch_one(pool)
    .await
}

pub async fn tracking_contract_summary() -> domain::tracking::TrackingModuleContract {
    tracking_module_contract()
}

fn leg_document_meta(params: &CreateLegDocumentParams, actor_user_id: Option<i64>) -> Value {
    let mut meta = Map::new();
    meta.insert("document_name".into(), json!(params.document_name));
    meta.insert("storage_provider".into(), json!(params.storage_provider));

    if let Some(original_name) = params
        .original_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        meta.insert("original_name".into(), json!(original_name));
    }

    if let Some(mime_type) = params
        .mime_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        meta.insert("mime_type".into(), json!(mime_type));
    }

    if let Some(file_size) = params.file_size {
        meta.insert("file_size".into(), json!(file_size));
    }

    if let Some(actor_user_id) = actor_user_id {
        meta.insert("uploaded_by".into(), json!(actor_user_id));
    }

    Value::Object(meta)
}
