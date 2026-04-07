use chrono::NaiveDateTime;
use domain::dispatch::{
    LegExecutionStatus, LegPostingStatus, LegacyLoadLegStatusCode, load_module_contract,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CountryRecord {
    pub id: i64,
    pub name: String,
    pub iso_code: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CityRecord {
    pub id: i64,
    pub country_id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocationRecord {
    pub id: i64,
    pub name: String,
    pub city_id: Option<i64>,
    pub country_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadTypeRecord {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EquipmentRecord {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommodityTypeRecord {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadRecord {
    pub id: i64,
    pub load_number: Option<String>,
    pub title: String,
    pub user_id: Option<i64>,
    pub load_type_id: Option<i64>,
    pub equipment_id: Option<i64>,
    pub commodity_type_id: Option<i64>,
    pub weight_unit: Option<String>,
    pub weight: Option<f64>,
    pub special_instructions: Option<String>,
    pub is_hazardous: bool,
    pub is_temperature_controlled: bool,
    pub status: i16,
    pub leg_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadDocumentRecord {
    pub id: i64,
    pub load_id: i64,
    pub document_name: String,
    pub document_type: String,
    pub file_path: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub hash: Option<String>,
    pub hash_algorithm: Option<String>,
    pub mock_blockchain_tx: Option<String>,
    pub mock_blockchain_timestamp: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadStatusMasterRecord {
    pub id: i16,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadLegRecord {
    pub id: i64,
    pub load_id: i64,
    pub leg_no: i32,
    pub leg_code: Option<String>,
    pub pickup_location_id: i64,
    pub delivery_location_id: i64,
    pub pickup_date: Option<NaiveDateTime>,
    pub delivery_date: Option<NaiveDateTime>,
    pub bid_status: Option<String>,
    pub price: Option<f64>,
    pub status_id: i16,
    pub booked_carrier_id: Option<i64>,
    pub booked_at: Option<NaiveDateTime>,
    pub booked_amount: Option<f64>,
    pub accepted_offer_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl LoadLegRecord {
    pub fn legacy_status(&self) -> Option<LegacyLoadLegStatusCode> {
        LegacyLoadLegStatusCode::from_legacy_code(self.status_id)
    }

    pub fn posting_status(&self) -> Option<LegPostingStatus> {
        match self.legacy_status()? {
            LegacyLoadLegStatusCode::Draft => Some(LegPostingStatus::Draft),
            LegacyLoadLegStatusCode::New => Some(LegPostingStatus::OpenForReview),
            LegacyLoadLegStatusCode::Reviewed | LegacyLoadLegStatusCode::OfferReady => {
                Some(LegPostingStatus::OpenForOffers)
            }
            LegacyLoadLegStatusCode::Booked
            | LegacyLoadLegStatusCode::EscrowFunded
            | LegacyLoadLegStatusCode::PickupStarted
            | LegacyLoadLegStatusCode::AtPickup
            | LegacyLoadLegStatusCode::InTransit
            | LegacyLoadLegStatusCode::AtDelivery
            | LegacyLoadLegStatusCode::Delivered
            | LegacyLoadLegStatusCode::PaidOut => Some(LegPostingStatus::Booked),
        }
    }

    pub fn execution_status(&self) -> Option<LegExecutionStatus> {
        match self.legacy_status()? {
            LegacyLoadLegStatusCode::EscrowFunded => Some(LegExecutionStatus::ReadyForPickup),
            LegacyLoadLegStatusCode::PickupStarted => Some(LegExecutionStatus::PickupStarted),
            LegacyLoadLegStatusCode::AtPickup => Some(LegExecutionStatus::AtPickup),
            LegacyLoadLegStatusCode::InTransit => Some(LegExecutionStatus::InTransit),
            LegacyLoadLegStatusCode::AtDelivery => Some(LegExecutionStatus::AtDelivery),
            LegacyLoadLegStatusCode::Delivered => Some(LegExecutionStatus::Delivered),
            LegacyLoadLegStatusCode::PaidOut => Some(LegExecutionStatus::PaidOut),
            LegacyLoadLegStatusCode::Booked => Some(LegExecutionStatus::AwaitingFunding),
            LegacyLoadLegStatusCode::Draft
            | LegacyLoadLegStatusCode::New
            | LegacyLoadLegStatusCode::Reviewed
            | LegacyLoadLegStatusCode::OfferReady => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadHistoryRecord {
    pub id: i64,
    pub load_id: i64,
    pub admin_id: Option<i64>,
    pub status: i16,
    pub remarks: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarrierPreferenceRecord {
    pub id: i64,
    pub user_id: i64,
    pub equipment_id: Option<String>,
    pub load_type_id: Option<String>,
    pub country_id: Option<String>,
    pub city_id: Option<String>,
    pub availability_days: Option<String>,
    pub max_weight_capacity: Option<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadBoardLegRecord {
    pub leg_id: i64,
    pub load_id: i64,
    pub leg_no: i32,
    pub leg_code: Option<String>,
    pub load_number: Option<String>,
    pub load_title: String,
    pub pickup_location_name: Option<String>,
    pub delivery_location_name: Option<String>,
    pub pickup_date: Option<NaiveDateTime>,
    pub delivery_date: Option<NaiveDateTime>,
    pub bid_status: Option<String>,
    pub price: Option<f64>,
    pub status_id: i16,
    pub booked_carrier_id: Option<i64>,
    pub booked_carrier_name: Option<String>,
    pub booked_amount: Option<f64>,
    pub escrow_status: Option<String>,
    pub stloads_status: Option<String>,
    pub stloads_retry_count: Option<i32>,
    pub stloads_alert_title: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadLegScopeRecord {
    pub leg_id: i64,
    pub load_id: i64,
    pub load_owner_user_id: Option<i64>,
    pub booked_carrier_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadBoardTabCountRecord {
    pub tab_key: String,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadBoardMetricsRecord {
    pub open_board_total: i64,
    pub recommended_total: i64,
    pub funding_watch_total: i64,
}

pub async fn list_recent_loads(pool: &DbPool, limit: i64) -> Result<Vec<LoadRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadRecord>(
        "SELECT id, load_number, title, user_id, load_type_id, equipment_id, commodity_type_id, weight_unit, weight,
                special_instructions, is_hazardous, is_temperature_controlled, status, leg_count, created_at, updated_at, deleted_at
         FROM loads
         WHERE deleted_at IS NULL
         ORDER BY id DESC
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_loads_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<LoadRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadRecord>(
        "SELECT id, load_number, title, user_id, load_type_id, equipment_id, commodity_type_id, weight_unit, weight,
                special_instructions, is_hazardous, is_temperature_controlled, status, leg_count, created_at, updated_at, deleted_at
         FROM loads
         WHERE deleted_at IS NULL AND user_id = $1
         ORDER BY id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn find_load_by_id(
    pool: &DbPool,
    load_id: i64,
) -> Result<Option<LoadRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadRecord>(
        "SELECT id, load_number, title, user_id, load_type_id, equipment_id, commodity_type_id, weight_unit, weight,
                special_instructions, is_hazardous, is_temperature_controlled, status, leg_count, created_at, updated_at, deleted_at
         FROM loads
         WHERE deleted_at IS NULL AND id = $1
         LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_load_legs_for_load(
    pool: &DbPool,
    load_id: i64,
) -> Result<Vec<LoadLegRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadLegRecord>(
        "SELECT id, load_id, leg_no, leg_code, pickup_location_id, delivery_location_id, pickup_date, delivery_date,
                bid_status, price, status_id, booked_carrier_id, booked_at, booked_amount, accepted_offer_id,
                created_at, updated_at, deleted_at
         FROM load_legs
         WHERE deleted_at IS NULL AND load_id = $1
         ORDER BY leg_no, id",
    )
    .bind(load_id)
    .fetch_all(pool)
    .await
}

pub async fn list_load_documents_for_load(
    pool: &DbPool,
    load_id: i64,
) -> Result<Vec<LoadDocumentRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadDocumentRecord>(
        "SELECT id, load_id, document_name, document_type, file_path, original_name, mime_type, file_size,
                hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp, created_at, updated_at
         FROM load_documents
         WHERE load_id = $1
         ORDER BY id DESC",
    )
    .bind(load_id)
    .fetch_all(pool)
    .await
}

pub async fn list_load_history_for_load(
    pool: &DbPool,
    load_id: i64,
) -> Result<Vec<LoadHistoryRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadHistoryRecord>(
        "SELECT id, load_id, admin_id, status, remarks, created_at, updated_at
         FROM load_history
         WHERE load_id = $1
         ORDER BY id DESC",
    )
    .bind(load_id)
    .fetch_all(pool)
    .await
}

pub async fn list_load_board_legs_filtered(
    pool: &DbPool,
    tab_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<LoadBoardLegRecord>, sqlx::Error> {
    let filter_clause = match tab_filter {
        Some("recommended") => "AND ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL",
        Some("booked") => "AND (ll.booked_carrier_id IS NOT NULL OR ll.status_id >= 4)",
        _ => "",
    };

    let query = format!(
        "{}\n        WHERE ll.deleted_at IS NULL\n        {}\n        ORDER BY COALESCE(ll.pickup_date, ll.created_at) ASC, ll.id DESC\n        LIMIT $1",
        load_board_select_sql(),
        filter_clause
    );

    sqlx::query_as::<_, LoadBoardLegRecord>(&query)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn list_load_board_legs_for_owner_filtered(
    pool: &DbPool,
    owner_user_id: i64,
    tab_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<LoadBoardLegRecord>, sqlx::Error> {
    let filter_clause = match tab_filter {
        Some("recommended") => {
            "AND l.user_id = $1 AND ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL"
        }
        Some("booked") => {
            "AND l.user_id = $1 AND (ll.booked_carrier_id IS NOT NULL OR ll.status_id >= 4)"
        }
        _ => "AND l.user_id = $1",
    };

    let query = format!(
        "{}\n        WHERE ll.deleted_at IS NULL\n        {}\n        ORDER BY COALESCE(ll.pickup_date, ll.created_at) ASC, ll.id DESC\n        LIMIT $2",
        load_board_select_sql(),
        filter_clause
    );

    sqlx::query_as::<_, LoadBoardLegRecord>(&query)
        .bind(owner_user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn list_load_board_legs_for_carrier_filtered(
    pool: &DbPool,
    carrier_user_id: i64,
    tab_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<LoadBoardLegRecord>, sqlx::Error> {
    let (filter_clause, limit_placeholder, needs_carrier_bind) = match tab_filter {
        Some("recommended") => (
            "AND ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL",
            1,
            false,
        ),
        Some("booked") => ("AND ll.booked_carrier_id = $1", 2, true),
        _ => (
            "AND ((ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL) OR ll.booked_carrier_id = $1)",
            2,
            true,
        ),
    };

    let query = format!(
        "{}\n        WHERE ll.deleted_at IS NULL\n        {}\n        ORDER BY COALESCE(ll.pickup_date, ll.created_at) ASC, ll.id DESC\n        LIMIT ${}",
        load_board_select_sql(),
        filter_clause,
        limit_placeholder
    );

    let query = sqlx::query_as::<_, LoadBoardLegRecord>(&query);
    let query = if needs_carrier_bind {
        query.bind(carrier_user_id).bind(limit)
    } else {
        query.bind(limit)
    };

    query.fetch_all(pool).await
}

pub async fn load_board_tab_counts(
    pool: &DbPool,
) -> Result<Vec<LoadBoardTabCountRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadBoardTabCountRecord>(
        r#"
        SELECT 'all' AS tab_key, COUNT(*) AS total
        FROM load_legs
        WHERE deleted_at IS NULL
        UNION ALL
        SELECT 'recommended' AS tab_key, COUNT(*) AS total
        FROM load_legs
        WHERE deleted_at IS NULL AND status_id IN (1, 2, 3) AND booked_carrier_id IS NULL
        UNION ALL
        SELECT 'booked' AS tab_key, COUNT(*) AS total
        FROM load_legs
        WHERE deleted_at IS NULL AND (booked_carrier_id IS NOT NULL OR status_id >= 4)
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn load_board_tab_counts_for_owner(
    pool: &DbPool,
    owner_user_id: i64,
) -> Result<Vec<LoadBoardTabCountRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadBoardTabCountRecord>(
        r#"
        SELECT 'all' AS tab_key, COUNT(*) AS total
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        WHERE ll.deleted_at IS NULL AND l.user_id = $1
        UNION ALL
        SELECT 'recommended' AS tab_key, COUNT(*) AS total
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        WHERE ll.deleted_at IS NULL AND l.user_id = $2 AND ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL
        UNION ALL
        SELECT 'booked' AS tab_key, COUNT(*) AS total
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        WHERE ll.deleted_at IS NULL AND l.user_id = $3 AND (ll.booked_carrier_id IS NOT NULL OR ll.status_id >= 4)
        "#,
    )
    .bind(owner_user_id)
    .bind(owner_user_id)
    .bind(owner_user_id)
    .fetch_all(pool)
    .await
}

pub async fn load_board_tab_counts_for_carrier(
    pool: &DbPool,
    carrier_user_id: i64,
) -> Result<Vec<LoadBoardTabCountRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadBoardTabCountRecord>(
        r#"
        SELECT 'all' AS tab_key, COUNT(*) AS total
        FROM load_legs
        WHERE deleted_at IS NULL
            AND ((status_id IN (1, 2, 3) AND booked_carrier_id IS NULL) OR booked_carrier_id = $1)
        UNION ALL
        SELECT 'recommended' AS tab_key, COUNT(*) AS total
        FROM load_legs
        WHERE deleted_at IS NULL AND status_id IN (1, 2, 3) AND booked_carrier_id IS NULL
        UNION ALL
        SELECT 'booked' AS tab_key, COUNT(*) AS total
        FROM load_legs
        WHERE deleted_at IS NULL AND booked_carrier_id = $2
        "#,
    )
    .bind(carrier_user_id)
    .bind(carrier_user_id)
    .fetch_all(pool)
    .await
}

pub async fn load_board_metrics(pool: &DbPool) -> Result<LoadBoardMetricsRecord, sqlx::Error> {
    sqlx::query_as::<_, LoadBoardMetricsRecord>(
        r#"
        SELECT
            COUNT(CASE WHEN ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL THEN 1 END) AS open_board_total,
            COUNT(CASE WHEN ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL AND ll.price IS NOT NULL THEN 1 END) AS recommended_total,
            COUNT(CASE
                WHEN (ll.booked_carrier_id IS NOT NULL OR ll.status_id >= 4)
                    AND (escrow.status IS NULL OR escrow.status NOT IN ('released', 'paid_out'))
                THEN 1
            END) AS funding_watch_total
        FROM load_legs ll
        LEFT JOIN escrows escrow ON escrow.leg_id = ll.id
        WHERE ll.deleted_at IS NULL
        "#,
    )
    .fetch_one(pool)
    .await
}

pub async fn load_board_metrics_for_owner(
    pool: &DbPool,
    owner_user_id: i64,
) -> Result<LoadBoardMetricsRecord, sqlx::Error> {
    sqlx::query_as::<_, LoadBoardMetricsRecord>(
        r#"
        SELECT
            COUNT(CASE WHEN ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL THEN 1 END) AS open_board_total,
            COUNT(CASE WHEN ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL AND ll.price IS NOT NULL THEN 1 END) AS recommended_total,
            COUNT(CASE
                WHEN (ll.booked_carrier_id IS NOT NULL OR ll.status_id >= 4)
                    AND (escrow.status IS NULL OR escrow.status NOT IN ('released', 'paid_out'))
                THEN 1
            END) AS funding_watch_total
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        LEFT JOIN escrows escrow ON escrow.leg_id = ll.id
        WHERE ll.deleted_at IS NULL AND l.user_id = $1
        "#,
    )
    .bind(owner_user_id)
    .fetch_one(pool)
    .await
}

pub async fn load_board_metrics_for_carrier(
    pool: &DbPool,
    carrier_user_id: i64,
) -> Result<LoadBoardMetricsRecord, sqlx::Error> {
    sqlx::query_as::<_, LoadBoardMetricsRecord>(
        r#"
        SELECT
            COUNT(CASE WHEN ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL THEN 1 END) AS open_board_total,
            COUNT(CASE WHEN ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL AND ll.price IS NOT NULL THEN 1 END) AS recommended_total,
            COUNT(CASE
                WHEN ll.booked_carrier_id = $1
                    AND (escrow.status IS NULL OR escrow.status NOT IN ('released', 'paid_out'))
                THEN 1
            END) AS funding_watch_total
        FROM load_legs ll
        LEFT JOIN escrows escrow ON escrow.leg_id = ll.id
        WHERE ll.deleted_at IS NULL
        "#,
    )
    .bind(carrier_user_id)
    .fetch_one(pool)
    .await
}

pub async fn find_load_leg_by_id(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<LoadLegRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadLegRecord>(
        "SELECT id, load_id, leg_no, leg_code, pickup_location_id, delivery_location_id, pickup_date, delivery_date,
                bid_status, price, status_id, booked_carrier_id, booked_at, booked_amount, accepted_offer_id,
                created_at, updated_at, deleted_at
         FROM load_legs
         WHERE deleted_at IS NULL AND id = $1
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_load_leg_scope(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<LoadLegScopeRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadLegScopeRecord>(
        r#"
        SELECT
            ll.id AS leg_id,
            ll.load_id,
            l.user_id AS load_owner_user_id,
            ll.booked_carrier_id
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        WHERE ll.deleted_at IS NULL AND ll.id = $1
        LIMIT 1
        "#,
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn book_load_leg(
    pool: &DbPool,
    leg_id: i64,
    carrier_id: i64,
    booked_amount: Option<f64>,
    actor_user_id: Option<i64>,
) -> Result<Option<LoadLegRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(leg) = sqlx::query_as::<_, LoadLegRecord>(
        "SELECT id, load_id, leg_no, leg_code, pickup_location_id, delivery_location_id, pickup_date, delivery_date,
                bid_status, price, status_id, booked_carrier_id, booked_at, booked_amount, accepted_offer_id,
                created_at, updated_at, deleted_at
         FROM load_legs
         WHERE deleted_at IS NULL AND id = $1
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let resolved_amount = booked_amount.or(leg.booked_amount).or(leg.price);

    sqlx::query(
        "UPDATE load_legs
         SET booked_carrier_id = $1,
             booked_amount = $2,
             booked_at = COALESCE(booked_at, CURRENT_TIMESTAMP),
             status_id = CASE WHEN status_id < 4 THEN 4 ELSE status_id END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3",
    )
    .bind(carrier_id)
    .bind(resolved_amount)
    .bind(leg_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, 4, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg.load_id)
    .bind(actor_user_id)
    .bind("Rust dispatch booking action")
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, LoadLegRecord>(
        "SELECT id, load_id, leg_no, leg_code, pickup_location_id, delivery_location_id, pickup_date, delivery_date,
                bid_status, price, status_id, booked_carrier_id, booked_at, booked_amount, accepted_offer_id,
                created_at, updated_at, deleted_at
         FROM load_legs
         WHERE deleted_at IS NULL AND id = $1
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}

pub async fn load_contract_summary() -> domain::dispatch::LoadModuleContract {
    load_module_contract()
}

fn load_board_select_sql() -> &'static str {
    r#"
        SELECT
            ll.id AS leg_id,
            ll.load_id,
            ll.leg_no,
            ll.leg_code,
            l.load_number,
            l.title AS load_title,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
            ll.pickup_date,
            ll.delivery_date,
            ll.bid_status,
            ll.price,
            ll.status_id,
            ll.booked_carrier_id,
            carrier.name AS booked_carrier_name,
            ll.booked_amount,
            escrow.status AS escrow_status,
            handoff.status AS stloads_status,
            handoff.retry_count AS stloads_retry_count,
            sync_issue.title AS stloads_alert_title,
            ll.created_at
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
        LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id
        LEFT JOIN users carrier ON carrier.id = ll.booked_carrier_id
        LEFT JOIN escrows escrow ON escrow.leg_id = ll.id
        LEFT JOIN stloads_handoffs handoff ON handoff.id = (
            SELECT handoff_inner.id
            FROM stloads_handoffs handoff_inner
            WHERE handoff_inner.load_id = l.id
            ORDER BY handoff_inner.id DESC
            LIMIT 1
        )
        LEFT JOIN stloads_sync_errors sync_issue ON sync_issue.id = (
            SELECT sync_issue_inner.id
            FROM stloads_sync_errors sync_issue_inner
            WHERE sync_issue_inner.handoff_id = handoff.id AND sync_issue_inner.resolved = FALSE
            ORDER BY sync_issue_inner.id DESC
            LIMIT 1
        )
    "#
}
