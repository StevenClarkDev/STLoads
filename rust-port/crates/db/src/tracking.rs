use chrono::{NaiveDateTime, Utc};
use domain::dispatch::LegacyLoadLegStatusCode;
use domain::tracking::{Coordinate, tracking_module_contract};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::FromRow;

use crate::{
    DbPool,
    tms::{EnqueueAtmpOutboundEvent, enqueue_atmp_outbound_event},
};

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
    pub pickup_date: Option<NaiveDateTime>,
    pub delivery_date: Option<NaiveDateTime>,
    pub status_id: i16,
    pub booked_carrier_id: Option<i64>,
    pub booked_carrier_name: Option<String>,
    pub load_owner_user_id: Option<i64>,
    pub load_owner_name: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionPostingScopeRecord {
    pub tenant_id: String,
    pub posting_id: i64,
    pub booking_award_id: Option<i64>,
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
        "SELECT id, leg_id, type, path, meta, created_at, updated_at
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
        "SELECT id, leg_id, type, path, meta, created_at, updated_at
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

pub async fn find_execution_posting_scope(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<ExecutionPostingScopeRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExecutionPostingScopeRecord>(
        r#"
        SELECT
            posting.tenant_id,
            posting.id AS posting_id,
            award.id AS booking_award_id
        FROM stloads_postings posting
        LEFT JOIN LATERAL (
            SELECT id
            FROM booking_awards award
            WHERE award.tenant_id = posting.tenant_id
              AND award.posting_id = posting.id
              AND award.status IN ('awarded', 'accepted', 'in_transit')
            ORDER BY award.id DESC
            LIMIT 1
        ) award ON TRUE
        WHERE posting.source_leg_id = $1::TEXT
          AND posting.deleted_at IS NULL
        ORDER BY posting.id DESC
        LIMIT 1
        "#,
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
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

pub async fn record_leg_exception(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    remarks: &str,
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
        "INSERT INTO leg_events (leg_id, type, created_at, updated_at)
         VALUES ($1, 'exception_reported', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .execute(&mut *tx)
    .await?;

    let detail = remarks.trim();
    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(existing.load_id)
    .bind(actor_user_id)
    .bind(existing.status_id)
    .bind(format!(
        "Rust execution exception for leg #{}: {}",
        leg_id, detail
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_execution_leg_by_id(pool, leg_id).await
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
         RETURNING id, leg_id, type, path, meta, created_at, updated_at",
    )
    .bind(leg_id)
    .bind(&params.document_type)
    .bind(&params.file_path)
    .bind(metadata)
    .fetch_one(&mut *tx)
    .await?;

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

pub async fn create_tracking_point(
    pool: &DbPool,
    leg_id: i64,
    lat: f64,
    lng: f64,
    recorded_at: Option<NaiveDateTime>,
    actor_user_id: Option<i64>,
) -> Result<LegLocationRecord, sqlx::Error> {
    let recorded_at = recorded_at.unwrap_or_else(|| Utc::now().naive_utc());
    let mut tx = pool.begin().await?;
    let Some(leg) = sqlx::query_as::<_, ExecutionLegRecord>(
        "SELECT
            ll.id AS leg_id,
            ll.load_id,
            l.load_number,
            l.title AS load_title,
            ll.leg_code,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
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
        return Err(sqlx::Error::RowNotFound);
    };

    let point_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO leg_locations (leg_id, lat, lng, recorded_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(leg_id)
    .bind(lat)
    .bind(lng)
    .bind(recorded_at)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO leg_events (leg_id, type, created_at, updated_at)
         VALUES ($1, 'location_ping', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg.load_id)
    .bind(actor_user_id)
    .bind(leg.status_id)
    .bind(format!(
        "Rust tracking location ping for leg #{}: {:.5}, {:.5}",
        leg_id, lat, lng
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

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

pub async fn scan_stale_tracking_alerts(
    pool: &DbPool,
    stale_after_minutes: i64,
    limit: i64,
) -> Result<u64, sqlx::Error> {
    let threshold = Utc::now().naive_utc() - chrono::Duration::minutes(stale_after_minutes.max(1));
    let rows = sqlx::query_as::<_, ExecutionLegRecord>(
        r#"
        SELECT
            ll.id AS leg_id,
            ll.load_id,
            l.load_number,
            l.title AS load_title,
            ll.leg_code,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
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
        LEFT JOIN LATERAL (
            SELECT recorded_at
            FROM leg_locations loc
            WHERE loc.leg_id = ll.id
            ORDER BY loc.recorded_at DESC, loc.id DESC
            LIMIT 1
        ) latest ON TRUE
        WHERE ll.deleted_at IS NULL
          AND ll.status_id IN (5, 6, 7, 9)
          AND (latest.recorded_at IS NULL OR latest.recorded_at <= $1)
        ORDER BY COALESCE(latest.recorded_at, ll.updated_at) ASC
        LIMIT $2
        "#,
    )
    .bind(threshold)
    .bind(limit.max(1))
    .fetch_all(pool)
    .await?;

    let mut inserted = 0_u64;
    for leg in rows {
        let recent_alert_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM leg_events
             WHERE leg_id = $1
               AND type = 'tracking_stale'
               AND created_at >= $2",
        )
        .bind(leg.leg_id)
        .bind(threshold)
        .fetch_one(pool)
        .await?
            > 0;
        if recent_alert_exists {
            continue;
        }

        sqlx::query(
            "INSERT INTO leg_events (leg_id, type, created_at, updated_at)
             VALUES ($1, 'tracking_stale', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(leg.leg_id)
        .execute(pool)
        .await?;

        if let Some(scope) = find_execution_posting_scope(pool, leg.leg_id).await? {
            enqueue_atmp_outbound_event(
                pool,
                EnqueueAtmpOutboundEvent {
                    tenant_id: &scope.tenant_id,
                    event_type: "exception_event",
                    posting_id: Some(scope.posting_id),
                    booking_award_id: scope.booking_award_id,
                    target_url: None,
                    payload: json!({
                        "event_type": "tracking_stale",
                        "leg_id": leg.leg_id,
                        "load_id": leg.load_id,
                        "status_id": leg.status_id,
                        "stale_after_minutes": stale_after_minutes.max(1),
                    }),
                    correlation_id: None,
                },
            )
            .await?;
        }

        inserted += 1;
    }

    Ok(inserted)
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
