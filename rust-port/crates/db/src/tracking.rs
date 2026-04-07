use chrono::NaiveDateTime;
use domain::tracking::{Coordinate, tracking_module_contract};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn list_tracking_points_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<LegLocationRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegLocationRecord>(
        "SELECT id, leg_id, lat, lng, recorded_at, created_at, updated_at
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
        "SELECT id, leg_id, lat, lng, recorded_at, created_at, updated_at
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

pub async fn tracking_contract_summary() -> domain::tracking::TrackingModuleContract {
    tracking_module_contract()
}
