use chrono::NaiveDateTime;
use domain::dispatch::{
    LegExecutionStatus, LegPostingStatus, LegacyLoadLegStatusCode, load_module_contract,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Postgres, QueryBuilder};

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
    pub organization_id: i64,
    pub load_number: Option<String>,
    pub title: String,
    pub user_id: Option<i64>,
    pub load_type_id: Option<i64>,
    pub equipment_id: Option<i64>,
    pub commodity_type_id: Option<i64>,
    pub customer_contract_id: Option<i64>,
    pub customer_contract_lane_id: Option<i64>,
    pub contract_rate: Option<f64>,
    pub contract_rate_currency: Option<String>,
    pub contract_posting_behavior: Option<String>,
    pub contract_service_rules: Option<Value>,
    pub freight_mode: String,
    pub visibility: String,
    pub service_level: Option<String>,
    pub customer_reference: Option<String>,
    pub po_number: Option<String>,
    pub pickup_appointment_ref: Option<String>,
    pub delivery_appointment_ref: Option<String>,
    pub facility_contact_name: Option<String>,
    pub facility_contact_phone: Option<String>,
    pub facility_contact_email: Option<String>,
    pub appointment_window_start: Option<NaiveDateTime>,
    pub appointment_window_end: Option<NaiveDateTime>,
    pub accessorial_flags: Option<Value>,
    pub weight_unit: Option<String>,
    pub weight: Option<f64>,
    pub temperature_data: Option<Value>,
    pub container_data: Option<Value>,
    pub securement_data: Option<Value>,
    pub special_instructions: Option<String>,
    pub is_hazardous: bool,
    pub is_temperature_controlled: bool,
    pub lifecycle_status: String,
    pub revision_number: i32,
    pub cloned_from_load_id: Option<i64>,
    pub is_template: bool,
    pub template_name: Option<String>,
    pub published_at: Option<NaiveDateTime>,
    pub revised_at: Option<NaiveDateTime>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub archived_at: Option<NaiveDateTime>,
    pub lifecycle_reason: Option<String>,
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
    pub storage_provider: String,
    pub uploaded_by_user_id: Option<i64>,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub hash: Option<String>,
    pub hash_algorithm: Option<String>,
    pub mock_blockchain_tx: Option<String>,
    pub mock_blockchain_timestamp: Option<NaiveDateTime>,
    pub current_version: i32,
    pub version_count: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadProfileLegRecord {
    pub id: i64,
    pub load_id: i64,
    pub leg_no: i32,
    pub leg_code: Option<String>,
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
    pub escrow_id: Option<i64>,
    pub escrow_status: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadBuilderLegRecord {
    pub id: i64,
    pub load_id: i64,
    pub leg_no: i32,
    pub leg_code: Option<String>,
    pub pickup_location_name: Option<String>,
    pub pickup_city_name: Option<String>,
    pub pickup_country_name: Option<String>,
    pub delivery_location_name: Option<String>,
    pub delivery_city_name: Option<String>,
    pub delivery_country_name: Option<String>,
    pub pickup_date: Option<NaiveDateTime>,
    pub delivery_date: Option<NaiveDateTime>,
    pub bid_status: Option<String>,
    pub price: Option<f64>,
    pub status_id: i16,
    pub booked_carrier_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub equipment_name: Option<String>,
    pub commodity_type_name: Option<String>,
    pub freight_mode: Option<String>,
    pub service_level: Option<String>,
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
    pub organization_id: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadBoardSavedFilterRecord {
    pub id: i64,
    pub name: String,
    pub role_key: Option<String>,
    pub is_default: bool,
    pub filter_payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBoardSearchResult {
    pub rows: Vec<LoadBoardLegRecord>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DispatchDeskLegRecord {
    pub leg_id: i64,
    pub load_id: i64,
    pub handoff_id: Option<i64>,
    pub desk_key: Option<String>,
    pub assigned_owner_name: Option<String>,
    pub priority: Option<String>,
    pub sla_due_at: Option<NaiveDateTime>,
    pub escalation_reason: Option<String>,
    pub exception_type: Option<String>,
    pub exception_status: Option<String>,
    pub exception_severity: Option<String>,
    pub latest_internal_note: Option<String>,
    pub latest_customer_update: Option<String>,
    pub load_number: Option<String>,
    pub load_title: String,
    pub equipment_name: Option<String>,
    pub weight: Option<f64>,
    pub status_id: i16,
    pub booked_carrier_id: Option<i64>,
    pub booked_carrier_name: Option<String>,
    pub booked_amount: Option<f64>,
    pub escrow_status: Option<String>,
    pub handoff_status: Option<String>,
    pub latest_activity_note: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminLoadLegRecord {
    pub leg_id: i64,
    pub load_id: i64,
    pub organization_id: i64,
    pub leg_code: Option<String>,
    pub load_number: Option<String>,
    pub owner_name: Option<String>,
    pub carrier_name: Option<String>,
    pub pickup_location_name: Option<String>,
    pub delivery_location_name: Option<String>,
    pub pickup_date: Option<NaiveDateTime>,
    pub delivery_date: Option<NaiveDateTime>,
    pub bid_status: Option<String>,
    pub price: Option<f64>,
    pub status_id: i16,
    pub booked_amount: Option<f64>,
    pub escrow_id: Option<i64>,
    pub escrow_status: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadIdAndStatusRecord {
    pub load_id: i64,
    pub status_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadStatusScopeRecord {
    pub load_id: i64,
    pub owner_user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoadParams {
    pub title: String,
    pub owner_user_id: i64,
    pub load_type_id: i64,
    pub equipment_id: i64,
    pub commodity_type_id: i64,
    pub customer_contract_id: Option<i64>,
    pub customer_contract_lane_id: Option<i64>,
    pub contract_rate: Option<f64>,
    pub contract_rate_currency: Option<String>,
    pub contract_posting_behavior: Option<String>,
    pub contract_service_rules: Option<Value>,
    pub freight_mode: String,
    pub visibility: String,
    pub service_level: Option<String>,
    pub customer_reference: Option<String>,
    pub po_number: Option<String>,
    pub pickup_appointment_ref: Option<String>,
    pub delivery_appointment_ref: Option<String>,
    pub facility_contact_name: Option<String>,
    pub facility_contact_phone: Option<String>,
    pub facility_contact_email: Option<String>,
    pub appointment_window_start: Option<NaiveDateTime>,
    pub appointment_window_end: Option<NaiveDateTime>,
    pub accessorial_flags: Option<Value>,
    pub weight_unit: String,
    pub weight: f64,
    pub temperature_data: Option<Value>,
    pub container_data: Option<Value>,
    pub securement_data: Option<Value>,
    pub special_instructions: Option<String>,
    pub is_hazardous: bool,
    pub is_temperature_controlled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoadLegParams {
    pub pickup_location_id: i64,
    pub delivery_location_id: i64,
    pub pickup_date: NaiveDateTime,
    pub delivery_date: NaiveDateTime,
    pub bid_status: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedLoadRecord {
    pub load_id: i64,
    pub load_number: String,
    pub leg_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadDocumentScopeRecord {
    pub document_id: i64,
    pub load_id: i64,
    pub organization_id: i64,
    pub load_owner_user_id: Option<i64>,
    pub uploaded_by_user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertLoadDocumentParams {
    pub document_name: String,
    pub document_type: String,
    pub file_path: String,
    pub storage_provider: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerContractRecord {
    pub id: i64,
    pub organization_id: i64,
    pub customer_user_id: Option<i64>,
    pub contract_number: String,
    pub contract_name: String,
    pub status: String,
    pub effective_start: chrono::NaiveDate,
    pub effective_end: Option<chrono::NaiveDate>,
    pub default_currency: String,
    pub service_rules: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerContractLaneRecord {
    pub id: i64,
    pub organization_id: i64,
    pub contract_id: i64,
    pub lane_name: String,
    pub origin_location_id: Option<i64>,
    pub destination_location_id: Option<i64>,
    pub equipment_id: Option<i64>,
    pub commodity_type_id: Option<i64>,
    pub freight_mode: String,
    pub contracted_rate: f64,
    pub rate_currency: String,
    pub accessorial_rules: Option<Value>,
    pub service_level: Option<String>,
    pub posting_behavior: String,
    pub tender_priority: i32,
    pub service_rules: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerContractParams {
    pub organization_id: i64,
    pub customer_user_id: Option<i64>,
    pub contract_number: String,
    pub contract_name: String,
    pub effective_start: chrono::NaiveDate,
    pub effective_end: Option<chrono::NaiveDate>,
    pub default_currency: String,
    pub service_rules: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerContractLaneParams {
    pub organization_id: i64,
    pub contract_id: i64,
    pub lane_name: String,
    pub origin_location_id: Option<i64>,
    pub destination_location_id: Option<i64>,
    pub equipment_id: Option<i64>,
    pub commodity_type_id: Option<i64>,
    pub freight_mode: String,
    pub contracted_rate: f64,
    pub rate_currency: String,
    pub accessorial_rules: Option<Value>,
    pub service_level: Option<String>,
    pub posting_behavior: String,
    pub tender_priority: i32,
    pub effective_start: chrono::NaiveDate,
    pub effective_end: Option<chrono::NaiveDate>,
    pub service_rules: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FreightDocumentTemplateRecord {
    pub id: i64,
    pub template_key: String,
    pub version: String,
    pub title: String,
    pub document_type_key: String,
    pub body_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FreightDocumentGenerationContext {
    pub load_id: i64,
    pub load_number: Option<String>,
    pub load_title: String,
    pub special_instructions: Option<String>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub pickup_location_name: Option<String>,
    pub delivery_location_name: Option<String>,
    pub booked_amount: Option<f64>,
    pub carrier_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GeneratedFreightDocumentRecord {
    pub id: i64,
    pub load_id: i64,
    pub document_id: i64,
    pub template_key: String,
    pub template_version: String,
    pub generated_by_user_id: Option<i64>,
    pub generation_status: String,
    pub created_at: NaiveDateTime,
}

pub async fn list_active_freight_document_templates(
    pool: &DbPool,
) -> Result<Vec<FreightDocumentTemplateRecord>, sqlx::Error> {
    sqlx::query_as::<_, FreightDocumentTemplateRecord>(
        "SELECT DISTINCT ON (template_key)
            id, template_key, version, title, document_type_key, body_template
         FROM freight_document_templates
         WHERE is_active = TRUE
         ORDER BY template_key, created_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn load_freight_document_context(
    pool: &DbPool,
    load_id: i64,
) -> Result<Option<FreightDocumentGenerationContext>, sqlx::Error> {
    sqlx::query_as::<_, FreightDocumentGenerationContext>(
        "SELECT
            load.id AS load_id,
            load.load_number,
            load.title AS load_title,
            load.special_instructions,
            load.weight::double precision AS weight,
            load.weight_unit,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
            leg.booked_amount::double precision AS booked_amount,
            carrier.name AS carrier_name
         FROM loads load
         LEFT JOIN load_legs leg ON leg.load_id = load.id AND leg.deleted_at IS NULL
         LEFT JOIN locations pickup ON pickup.id = leg.pickup_location_id
         LEFT JOIN locations delivery ON delivery.id = leg.delivery_location_id
         LEFT JOIN users carrier ON carrier.id = leg.booked_carrier_id
         WHERE load.id = $1 AND load.deleted_at IS NULL
         ORDER BY leg.leg_no ASC NULLS LAST, leg.id ASC NULLS LAST
         LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(pool)
    .await
}

pub async fn record_generated_freight_document(
    pool: &DbPool,
    load_id: i64,
    document_id: i64,
    template_key: &str,
    template_version: &str,
    generated_by_user_id: Option<i64>,
) -> Result<GeneratedFreightDocumentRecord, sqlx::Error> {
    sqlx::query_as::<_, GeneratedFreightDocumentRecord>(
        "INSERT INTO generated_freight_documents (
            load_id, document_id, template_key, template_version, generated_by_user_id,
            generation_status, created_at
         ) VALUES ($1, $2, $3, $4, $5, 'generated', CURRENT_TIMESTAMP)
         ON CONFLICT (load_id, template_key, template_version)
         DO UPDATE SET
            document_id = EXCLUDED.document_id,
            generated_by_user_id = EXCLUDED.generated_by_user_id,
            generation_status = 'regenerated',
            created_at = CURRENT_TIMESTAMP
         RETURNING id, load_id, document_id, template_key, template_version,
            generated_by_user_id, generation_status, created_at",
    )
    .bind(load_id)
    .bind(document_id)
    .bind(template_key)
    .bind(template_version)
    .bind(generated_by_user_id)
    .fetch_one(pool)
    .await
}

pub async fn create_customer_contract(
    pool: &DbPool,
    params: &CreateCustomerContractParams,
) -> Result<CustomerContractRecord, sqlx::Error> {
    sqlx::query_as::<_, CustomerContractRecord>(
        "INSERT INTO customer_contracts (
            organization_id, customer_user_id, contract_number, contract_name, status,
            effective_start, effective_end, default_currency, service_rules, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, 'active', $5, $6, $7, $8, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, organization_id, customer_user_id, contract_number, contract_name, status,
            effective_start, effective_end, default_currency, service_rules",
    )
    .bind(params.organization_id)
    .bind(params.customer_user_id)
    .bind(&params.contract_number)
    .bind(&params.contract_name)
    .bind(params.effective_start)
    .bind(params.effective_end)
    .bind(&params.default_currency)
    .bind(&params.service_rules)
    .fetch_one(pool)
    .await
}

pub async fn create_customer_contract_lane(
    pool: &DbPool,
    params: &CreateCustomerContractLaneParams,
) -> Result<CustomerContractLaneRecord, sqlx::Error> {
    sqlx::query_as::<_, CustomerContractLaneRecord>(
        "INSERT INTO customer_contract_lanes (
            organization_id, contract_id, lane_name, origin_location_id, destination_location_id,
            equipment_id, commodity_type_id, freight_mode, contracted_rate, rate_currency,
            accessorial_rules, service_level, posting_behavior, tender_priority,
            effective_start, effective_end, service_rules, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, organization_id, contract_id, lane_name, origin_location_id,
            destination_location_id, equipment_id, commodity_type_id, freight_mode,
            contracted_rate::double precision AS contracted_rate, rate_currency,
            accessorial_rules, service_level, posting_behavior, tender_priority, service_rules",
    )
    .bind(params.organization_id)
    .bind(params.contract_id)
    .bind(&params.lane_name)
    .bind(params.origin_location_id)
    .bind(params.destination_location_id)
    .bind(params.equipment_id)
    .bind(params.commodity_type_id)
    .bind(&params.freight_mode)
    .bind(params.contracted_rate)
    .bind(&params.rate_currency)
    .bind(&params.accessorial_rules)
    .bind(&params.service_level)
    .bind(&params.posting_behavior)
    .bind(params.tender_priority)
    .bind(params.effective_start)
    .bind(params.effective_end)
    .bind(&params.service_rules)
    .fetch_one(pool)
    .await
}

pub async fn find_active_customer_contract_lane(
    pool: &DbPool,
    organization_id: i64,
    lane_id: i64,
) -> Result<Option<CustomerContractLaneRecord>, sqlx::Error> {
    sqlx::query_as::<_, CustomerContractLaneRecord>(
        "SELECT lane.id, lane.organization_id, lane.contract_id, lane.lane_name,
            lane.origin_location_id, lane.destination_location_id, lane.equipment_id,
            lane.commodity_type_id, lane.freight_mode,
            lane.contracted_rate::double precision AS contracted_rate, lane.rate_currency,
            lane.accessorial_rules, lane.service_level, lane.posting_behavior,
            lane.tender_priority, COALESCE(lane.service_rules, contract.service_rules) AS service_rules
         FROM customer_contract_lanes lane
         INNER JOIN customer_contracts contract ON contract.id = lane.contract_id
            AND contract.deleted_at IS NULL
            AND contract.status = 'active'
            AND contract.effective_start <= CURRENT_DATE
            AND (contract.effective_end IS NULL OR contract.effective_end >= CURRENT_DATE)
         WHERE lane.deleted_at IS NULL
            AND lane.organization_id = $1
            AND lane.id = $2
            AND lane.effective_start <= CURRENT_DATE
            AND (lane.effective_end IS NULL OR lane.effective_end >= CURRENT_DATE)
         LIMIT 1",
    )
    .bind(organization_id)
    .bind(lane_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_recent_loads(pool: &DbPool, limit: i64) -> Result<Vec<LoadRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadRecord>(
        "SELECT id, organization_id, load_number, title, user_id, load_type_id, equipment_id, commodity_type_id,
                customer_contract_id, customer_contract_lane_id, contract_rate::double precision AS contract_rate,
                contract_rate_currency, contract_posting_behavior, contract_service_rules,
                freight_mode, visibility, service_level, customer_reference, po_number, pickup_appointment_ref, delivery_appointment_ref,
                facility_contact_name, facility_contact_phone, facility_contact_email, appointment_window_start, appointment_window_end,
                accessorial_flags, weight_unit, weight::double precision AS weight, temperature_data, container_data, securement_data,
                special_instructions, is_hazardous, is_temperature_controlled, lifecycle_status, revision_number, cloned_from_load_id,
                is_template, template_name, published_at, revised_at, cancelled_at, archived_at, lifecycle_reason,
                status, leg_count, created_at, updated_at, deleted_at
         FROM loads
         WHERE deleted_at IS NULL
         ORDER BY id DESC
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_admin_load_legs_filtered(
    pool: &DbPool,
    status_ids: Option<&[i16]>,
    organization_id: Option<i64>,
    limit: i64,
) -> Result<Vec<AdminLoadLegRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminLoadLegRecord>(
        r#"
        SELECT
            ll.id AS leg_id,
            ll.load_id,
            l.organization_id,
            ll.leg_code,
            l.load_number,
            owner.name AS owner_name,
            carrier.name AS carrier_name,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
            ll.pickup_date,
            ll.delivery_date,
            ll.bid_status,
            ll.price::double precision AS price,
            ll.status_id,
            ll.booked_amount::double precision AS booked_amount,
            escrow.id AS escrow_id,
            escrow.status AS escrow_status,
            ll.created_at
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        LEFT JOIN users owner ON owner.id = l.user_id
        LEFT JOIN users carrier ON carrier.id = ll.booked_carrier_id
        LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
        LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id
        LEFT JOIN escrows escrow ON escrow.leg_id = ll.id
        WHERE ll.deleted_at IS NULL
          AND ($1::smallint[] IS NULL OR ll.status_id = ANY($1))
          AND ($2::bigint IS NULL OR l.organization_id = $2)
        ORDER BY ll.created_at DESC, ll.id DESC
        LIMIT $3
        "#,
    )
    .bind(status_ids)
    .bind(organization_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn count_admin_load_legs_filtered(
    pool: &DbPool,
    status_ids: Option<&[i16]>,
    organization_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let (total,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        WHERE ll.deleted_at IS NULL
          AND ($1::smallint[] IS NULL OR ll.status_id = ANY($1))
          AND ($2::bigint IS NULL OR l.organization_id = $2)
        "#,
    )
    .bind(status_ids)
    .bind(organization_id)
    .fetch_one(pool)
    .await?;

    Ok(total)
}

pub async fn list_loads_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<LoadRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadRecord>(
        "SELECT id, organization_id, load_number, title, user_id, load_type_id, equipment_id, commodity_type_id,
                customer_contract_id, customer_contract_lane_id, contract_rate::double precision AS contract_rate,
                contract_rate_currency, contract_posting_behavior, contract_service_rules,
                freight_mode, visibility, service_level, customer_reference, po_number, pickup_appointment_ref, delivery_appointment_ref,
                facility_contact_name, facility_contact_phone, facility_contact_email, appointment_window_start, appointment_window_end,
                accessorial_flags, weight_unit, weight::double precision AS weight, temperature_data, container_data, securement_data,
                special_instructions, is_hazardous, is_temperature_controlled, lifecycle_status, revision_number, cloned_from_load_id,
                is_template, template_name, published_at, revised_at, cancelled_at, archived_at, lifecycle_reason,
                status, leg_count, created_at, updated_at, deleted_at
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
        "SELECT id, organization_id, load_number, title, user_id, load_type_id, equipment_id, commodity_type_id,
                customer_contract_id, customer_contract_lane_id, contract_rate::double precision AS contract_rate,
                contract_rate_currency, contract_posting_behavior, contract_service_rules,
                freight_mode, visibility, service_level, customer_reference, po_number, pickup_appointment_ref, delivery_appointment_ref,
                facility_contact_name, facility_contact_phone, facility_contact_email, appointment_window_start, appointment_window_end,
                accessorial_flags, weight_unit, weight::double precision AS weight, temperature_data, container_data, securement_data,
                special_instructions, is_hazardous, is_temperature_controlled, lifecycle_status, revision_number, cloned_from_load_id,
                is_template, template_name, published_at, revised_at, cancelled_at, archived_at, lifecycle_reason,
                status, leg_count, created_at, updated_at, deleted_at
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
                bid_status, price::double precision AS price, status_id, booked_carrier_id, booked_at, booked_amount::double precision AS booked_amount, accepted_offer_id,
                created_at, updated_at, deleted_at
         FROM load_legs
         WHERE deleted_at IS NULL AND load_id = $1
         ORDER BY leg_no, id",
    )
    .bind(load_id)
    .fetch_all(pool)
    .await
}

pub async fn list_load_profile_legs_for_load(
    pool: &DbPool,
    load_id: i64,
) -> Result<Vec<LoadProfileLegRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadProfileLegRecord>(
        "SELECT ll.id, ll.load_id, ll.leg_no, ll.leg_code,
                pickup.name AS pickup_location_name,
                delivery.name AS delivery_location_name,
                ll.pickup_date, ll.delivery_date, ll.bid_status, ll.price::double precision AS price,
                ll.status_id, ll.booked_carrier_id,
                carrier.name AS booked_carrier_name,
                ll.booked_amount::double precision AS booked_amount,
                escrow.id AS escrow_id,
                escrow.status AS escrow_status,
                ll.created_at, ll.updated_at
         FROM load_legs ll
         LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
         LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id
         LEFT JOIN users carrier ON carrier.id = ll.booked_carrier_id
         LEFT JOIN escrows escrow ON escrow.load_leg_id = ll.id
         WHERE ll.deleted_at IS NULL AND ll.load_id = $1
         ORDER BY ll.leg_no, ll.id",
    )
    .bind(load_id)
    .fetch_all(pool)
    .await
}
pub async fn list_load_builder_legs_for_load(
    pool: &DbPool,
    load_id: i64,
) -> Result<Vec<LoadBuilderLegRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadBuilderLegRecord>(
        "SELECT ll.id, ll.load_id, ll.leg_no, ll.leg_code,
                pickup.name AS pickup_location_name,
                pickup_city.name AS pickup_city_name,
                pickup_country.name AS pickup_country_name,
                delivery.name AS delivery_location_name,
                delivery_city.name AS delivery_city_name,
                delivery_country.name AS delivery_country_name,
                ll.pickup_date, ll.delivery_date, ll.bid_status, ll.price::double precision AS price,
                ll.status_id, ll.booked_carrier_id, ll.created_at, ll.updated_at
         FROM load_legs ll
         LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
         LEFT JOIN cities pickup_city ON pickup_city.id = pickup.city_id
         LEFT JOIN countries pickup_country ON pickup_country.id = pickup.country_id
         LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id
         LEFT JOIN cities delivery_city ON delivery_city.id = delivery.city_id
         LEFT JOIN countries delivery_country ON delivery_country.id = delivery.country_id
         WHERE ll.deleted_at IS NULL AND ll.load_id = $1
         ORDER BY ll.leg_no, ll.id",
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
        "SELECT id, load_id, document_name, document_type, file_path, storage_provider, uploaded_by_user_id,
                original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx,
                mock_blockchain_timestamp,
                COALESCE((SELECT MAX(version_number) FROM load_document_versions WHERE document_id = load_documents.id), 1)::int AS current_version,
                COALESCE((SELECT COUNT(*) FROM load_document_versions WHERE document_id = load_documents.id), 1)::bigint AS version_count,
                created_at, updated_at
         FROM load_documents
         WHERE load_id = $1
         ORDER BY id DESC",
    )
    .bind(load_id)
    .fetch_all(pool)
    .await
}

pub async fn find_load_document_by_id(
    pool: &DbPool,
    document_id: i64,
) -> Result<Option<LoadDocumentRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadDocumentRecord>(
        "SELECT id, load_id, document_name, document_type, file_path, storage_provider, uploaded_by_user_id,
                original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx,
                mock_blockchain_timestamp,
                COALESCE((SELECT MAX(version_number) FROM load_document_versions WHERE document_id = load_documents.id), 1)::int AS current_version,
                COALESCE((SELECT COUNT(*) FROM load_document_versions WHERE document_id = load_documents.id), 1)::bigint AS version_count,
                created_at, updated_at
         FROM load_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_load_document_scope(
    pool: &DbPool,
    document_id: i64,
) -> Result<Option<LoadDocumentScopeRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadDocumentScopeRecord>(
        r#"
        SELECT
            document.id AS document_id,
            document.load_id,
            load.organization_id,
            load.user_id AS load_owner_user_id,
            document.uploaded_by_user_id
        FROM load_documents document
        INNER JOIN loads load ON load.id = document.load_id AND load.deleted_at IS NULL
        WHERE document.id = $1
        LIMIT 1
        "#,
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_load_document(
    pool: &DbPool,
    load_id: i64,
    params: &UpsertLoadDocumentParams,
    actor_user_id: Option<i64>,
) -> Result<Option<LoadDocumentRecord>, sqlx::Error> {
    #[derive(FromRow)]
    struct LoadStatusRow {
        id: i64,
        status: i16,
    }

    let mut tx = pool.begin().await?;
    let Some(load_row) = sqlx::query_as::<_, LoadStatusRow>(
        "SELECT id, status FROM loads WHERE deleted_at IS NULL AND id = $1 LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let created = sqlx::query_as::<_, LoadDocumentRecord>(
        "INSERT INTO load_documents (
            load_id, document_name, document_type, file_path, storage_provider, uploaded_by_user_id,
            original_name, mime_type, file_size, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, load_id, document_name, document_type, file_path, storage_provider, uploaded_by_user_id,
             original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx,
             mock_blockchain_timestamp,
             1::int AS current_version,
             1::bigint AS version_count,
             created_at, updated_at",
    )
    .bind(load_id)
    .bind(&params.document_name)
    .bind(&params.document_type)
    .bind(&params.file_path)
    .bind(&params.storage_provider)
    .bind(actor_user_id)
    .bind(params.original_name.as_deref())
    .bind(params.mime_type.as_deref())
    .bind(params.file_size)
    .fetch_one(&mut *tx)
    .await?;

    insert_load_document_version(
        &mut tx,
        created.id,
        1,
        &created.document_name,
        &created.document_type,
        &created.file_path,
        &created.storage_provider,
        created.uploaded_by_user_id,
        created.original_name.as_deref(),
        created.mime_type.as_deref(),
        created.file_size,
        created.hash.as_deref(),
        created.hash_algorithm.as_deref(),
        created.mock_blockchain_tx.as_deref(),
        created.mock_blockchain_timestamp,
        Some("initial upload"),
    )
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_row.id)
    .bind(actor_user_id)
    .bind(load_row.status)
    .bind(format!(
        "Rust load profile added document {}",
        params.document_name
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(created))
}

pub async fn update_load_document(
    pool: &DbPool,
    document_id: i64,
    params: &UpsertLoadDocumentParams,
    actor_user_id: Option<i64>,
) -> Result<Option<LoadDocumentRecord>, sqlx::Error> {
    #[derive(FromRow)]
    struct DocumentLoadRow {
        load_id: i64,
        status: i16,
    }

    let mut tx = pool.begin().await?;
    let Some(load_row) = sqlx::query_as::<_, DocumentLoadRow>(
        "SELECT document.load_id, load.status
         FROM load_documents document
         INNER JOIN loads load ON load.id = document.load_id AND load.deleted_at IS NULL
         WHERE document.id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let next_version = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(version_number), 0)::int + 1
         FROM load_document_versions
         WHERE document_id = $1",
    )
    .bind(document_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE load_documents
         SET document_name = $1,
             document_type = $2,
             file_path = $3,
             storage_provider = $4,
             original_name = $5,
             mime_type = $6,
             file_size = $7,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $8",
    )
    .bind(&params.document_name)
    .bind(&params.document_type)
    .bind(&params.file_path)
    .bind(&params.storage_provider)
    .bind(params.original_name.as_deref())
    .bind(params.mime_type.as_deref())
    .bind(params.file_size)
    .bind(document_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_row.load_id)
    .bind(actor_user_id)
    .bind(load_row.status)
    .bind(format!(
        "Rust load profile updated document {}",
        params.document_name
    ))
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, LoadDocumentRecord>(
        "SELECT id, load_id, document_name, document_type, file_path, storage_provider, uploaded_by_user_id,
                original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx,
                mock_blockchain_timestamp,
                COALESCE((SELECT MAX(version_number) FROM load_document_versions WHERE document_id = load_documents.id), 1)::int AS current_version,
                COALESCE((SELECT COUNT(*) FROM load_document_versions WHERE document_id = load_documents.id), 1)::bigint AS version_count,
                created_at, updated_at
         FROM load_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(&mut *tx)
    .await?;

    let mut updated = updated;
    if let Some(updated_document) = updated.as_mut() {
        insert_load_document_version(
            &mut tx,
            updated_document.id,
            next_version.max(1),
            &updated_document.document_name,
            &updated_document.document_type,
            &updated_document.file_path,
            &updated_document.storage_provider,
            updated_document.uploaded_by_user_id,
            updated_document.original_name.as_deref(),
            updated_document.mime_type.as_deref(),
            updated_document.file_size,
            updated_document.hash.as_deref(),
            updated_document.hash_algorithm.as_deref(),
            updated_document.mock_blockchain_tx.as_deref(),
            updated_document.mock_blockchain_timestamp,
            Some("document replacement or metadata update"),
        )
        .await?;
        updated_document.current_version = next_version.max(1);
        updated_document.version_count = i64::from(next_version.max(1));
    }

    tx.commit().await?;
    Ok(updated)
}

#[allow(clippy::too_many_arguments)]
async fn insert_load_document_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_id: i64,
    version_number: i32,
    document_name: &str,
    document_type: &str,
    file_path: &str,
    storage_provider: &str,
    uploaded_by_user_id: Option<i64>,
    original_name: Option<&str>,
    mime_type: Option<&str>,
    file_size: Option<i64>,
    hash: Option<&str>,
    hash_algorithm: Option<&str>,
    mock_blockchain_tx: Option<&str>,
    mock_blockchain_timestamp: Option<NaiveDateTime>,
    replacement_reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO load_document_versions (
            document_id, version_number, document_name, document_type, file_path, storage_provider,
            uploaded_by_user_id, original_name, mime_type, file_size, hash, hash_algorithm,
            mock_blockchain_tx, mock_blockchain_timestamp, replacement_reason, created_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, CURRENT_TIMESTAMP)
         ON CONFLICT (document_id, version_number) DO NOTHING",
    )
    .bind(document_id)
    .bind(version_number)
    .bind(document_name)
    .bind(document_type)
    .bind(file_path)
    .bind(storage_provider)
    .bind(uploaded_by_user_id)
    .bind(original_name)
    .bind(mime_type)
    .bind(file_size)
    .bind(hash)
    .bind(hash_algorithm)
    .bind(mock_blockchain_tx)
    .bind(mock_blockchain_timestamp)
    .bind(replacement_reason)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn verify_load_document_blockchain(
    pool: &DbPool,
    document_id: i64,
    content_sha256: &str,
    actor_user_id: Option<i64>,
    note: Option<&str>,
) -> Result<Option<LoadDocumentRecord>, sqlx::Error> {
    #[derive(FromRow)]
    struct DocumentLoadRow {
        load_id: i64,
        status: i16,
        document_name: String,
    }

    let mut tx = pool.begin().await?;
    let Some(load_row) = sqlx::query_as::<_, DocumentLoadRow>(
        "SELECT document.load_id, load.status, document.document_name
         FROM load_documents document
         INNER JOIN loads load ON load.id = document.load_id AND load.deleted_at IS NULL
         WHERE document.id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let next_version = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(version_number), 0)::int + 1
         FROM load_document_versions
         WHERE document_id = $1",
    )
    .bind(document_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE load_documents
         SET document_type = 'blockchain',
             hash = $1,
             hash_algorithm = 'sha256',
             mock_blockchain_tx = NULL,
             mock_blockchain_timestamp = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(content_sha256)
    .bind(document_id)
    .execute(&mut *tx)
    .await?;

    let remark = note
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            format!(
                "Rust content hash verification for {}: {}",
                load_row.document_name, value
            )
        })
        .unwrap_or_else(|| {
            format!(
                "Rust content hash verification completed for {}",
                load_row.document_name
            )
        });

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_row.load_id)
    .bind(actor_user_id)
    .bind(load_row.status)
    .bind(remark)
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, LoadDocumentRecord>(
        "SELECT id, load_id, document_name, document_type, file_path, storage_provider, uploaded_by_user_id,
                original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx,
                mock_blockchain_timestamp,
                COALESCE((SELECT MAX(version_number) FROM load_document_versions WHERE document_id = load_documents.id), 1)::int AS current_version,
                COALESCE((SELECT COUNT(*) FROM load_document_versions WHERE document_id = load_documents.id), 1)::bigint AS version_count,
                created_at, updated_at
         FROM load_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(&mut *tx)
    .await?;

    let mut updated = updated;
    if let Some(updated_document) = updated.as_mut() {
        insert_load_document_version(
            &mut tx,
            updated_document.id,
            next_version.max(1),
            &updated_document.document_name,
            &updated_document.document_type,
            &updated_document.file_path,
            &updated_document.storage_provider,
            updated_document.uploaded_by_user_id,
            updated_document.original_name.as_deref(),
            updated_document.mime_type.as_deref(),
            updated_document.file_size,
            updated_document.hash.as_deref(),
            updated_document.hash_algorithm.as_deref(),
            updated_document.mock_blockchain_tx.as_deref(),
            updated_document.mock_blockchain_timestamp,
            Some("blockchain verification metadata update"),
        )
        .await?;
        updated_document.current_version = next_version.max(1);
        updated_document.version_count = i64::from(next_version.max(1));
    }

    tx.commit().await?;
    Ok(updated)
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

pub async fn load_board_search(
    pool: &DbPool,
    viewer_role: Option<domain::auth::UserRole>,
    viewer_user_id: i64,
    filters: &shared::LoadBoardFilters,
    tab_filter: Option<&str>,
) -> Result<LoadBoardSearchResult, sqlx::Error> {
    let page = filters.page.max(1) as i64;
    let per_page = filters.per_page.clamp(1, 100) as i64;
    let offset = (page - 1) * per_page;

    let mut data_query = QueryBuilder::<Postgres>::new(load_board_select_sql());
    push_load_board_search_where(
        &mut data_query,
        viewer_role,
        viewer_user_id,
        filters,
        tab_filter,
    );
    data_query.push(" ORDER BY COALESCE(ll.pickup_date, ll.created_at) ASC, ll.id DESC LIMIT ");
    data_query.push_bind(per_page);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);
    let rows = data_query
        .build_query_as::<LoadBoardLegRecord>()
        .fetch_all(pool)
        .await?;

    let mut count_query = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) FROM load_legs ll
         INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
         LEFT JOIN locations pickup ON pickup.id = ll.pickup_location_id
         LEFT JOIN locations delivery ON delivery.id = ll.delivery_location_id",
    );
    push_load_board_search_where(
        &mut count_query,
        viewer_role,
        viewer_user_id,
        filters,
        tab_filter,
    );
    let (total,): (i64,) = count_query.build_query_as().fetch_one(pool).await?;

    Ok(LoadBoardSearchResult { rows, total })
}

fn push_load_board_search_where(
    query: &mut QueryBuilder<'_, Postgres>,
    viewer_role: Option<domain::auth::UserRole>,
    viewer_user_id: i64,
    filters: &shared::LoadBoardFilters,
    tab_filter: Option<&str>,
) {
    query.push(" WHERE ll.deleted_at IS NULL");

    match tab_filter {
        Some("recommended") => {
            query.push(" AND ll.status_id IN (1, 2, 3) AND ll.booked_carrier_id IS NULL");
        }
        Some("booked") => {
            query.push(" AND (ll.booked_carrier_id IS NOT NULL OR ll.status_id >= 4)");
        }
        _ => {}
    }

    match viewer_role {
        Some(domain::auth::UserRole::Carrier) => {
            query.push(
                " AND (
                    ll.booked_carrier_id = ",
            );
            query.push_bind(viewer_user_id);
            query.push(
                " OR (
                    ll.status_id IN (1, 2, 3)
                    AND ll.booked_carrier_id IS NULL
                    AND NOT EXISTS (
                        SELECT 1
                        FROM carrier_network_memberships blocked_network
                        WHERE blocked_network.owner_user_id = l.user_id
                          AND blocked_network.carrier_user_id = ",
            );
            query.push_bind(viewer_user_id);
            query.push(
                "
                          AND blocked_network.relationship_status = 'blocked'
                          AND blocked_network.effective_from <= CURRENT_DATE
                          AND (blocked_network.effective_to IS NULL OR blocked_network.effective_to >= CURRENT_DATE)
                    )
                    AND (
                        l.visibility = 'public'
                        OR EXISTS (
                            SELECT 1
                            FROM carrier_network_memberships allowed_network
                            WHERE allowed_network.owner_user_id = l.user_id
                              AND allowed_network.carrier_user_id = ",
            );
            query.push_bind(viewer_user_id);
            query.push(
                "
                              AND allowed_network.relationship_status IN ('approved', 'preferred', 'backup')
                              AND allowed_network.effective_from <= CURRENT_DATE
                              AND (allowed_network.effective_to IS NULL OR allowed_network.effective_to >= CURRENT_DATE)
                        )
                    )
                ))",
            );
        }
        Some(domain::auth::UserRole::Shipper)
        | Some(domain::auth::UserRole::Broker)
        | Some(domain::auth::UserRole::FreightForwarder) => {
            query.push(" AND l.user_id = ");
            query.push_bind(viewer_user_id);
        }
        _ => {}
    }

    if let Some(origin) = filters
        .origin
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND pickup.name ILIKE ");
        query.push_bind(format!("%{}%", origin));
    }
    if let Some(destination) = filters
        .destination
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND delivery.name ILIKE ");
        query.push_bind(format!("%{}%", destination));
    }
    if let Some(pickup_date) = filters
        .pickup_date
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        query.push(" AND ll.pickup_date::date = ");
        query.push_bind(pickup_date.to_string());
        query.push("::date");
    }
    if let Some(delivery_date) = filters
        .delivery_date
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        query.push(" AND ll.delivery_date::date = ");
        query.push_bind(delivery_date.to_string());
        query.push("::date");
    }
    if let Some(equipment_id) = filters.equipment_id {
        query.push(" AND l.equipment_id = ");
        query.push_bind(equipment_id as i64);
    }
    if let Some(commodity_type_id) = filters.commodity_type_id {
        query.push(" AND l.commodity_type_id = ");
        query.push_bind(commodity_type_id as i64);
    }
    if let Some(min_rate) = filters.min_rate {
        query.push(" AND COALESCE(ll.booked_amount, ll.price) >= ");
        query.push_bind(min_rate);
    }
    if let Some(max_rate) = filters.max_rate {
        query.push(" AND COALESCE(ll.booked_amount, ll.price) <= ");
        query.push_bind(max_rate);
    }
    if let Some(customer) = filters
        .customer
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND (l.customer_reference ILIKE ");
        query.push_bind(format!("%{}%", customer));
        query.push(" OR l.po_number ILIKE ");
        query.push_bind(format!("%{}%", customer));
        query.push(" OR l.title ILIKE ");
        query.push_bind(format!("%{}%", customer));
        query.push(")");
    }
    if let Some(status) = filters
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && let Ok(status_id) = status.parse::<i16>()
    {
        query.push(" AND ll.status_id = ");
        query.push_bind(status_id);
    }
    if let Some(compliance) = filters.compliance.as_deref() {
        match compliance {
            "hazmat" => {
                query.push(" AND l.is_hazardous = TRUE");
            }
            "temperature_controlled" => {
                query.push(" AND l.is_temperature_controlled = TRUE");
            }
            "documents_required" => {
                query.push(" AND EXISTS (SELECT 1 FROM required_document_rules rule WHERE rule.is_active = TRUE)");
            }
            _ => {}
        };
    }
    if let Some(visibility) = filters
        .visibility
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND l.visibility = ");
        query.push_bind(visibility.to_string());
    }
}

pub async fn list_load_board_saved_filters(
    pool: &DbPool,
    organization_id: i64,
    user_id: i64,
    role_key: Option<&str>,
) -> Result<Vec<LoadBoardSavedFilterRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadBoardSavedFilterRecord>(
        "SELECT id, name, role_key, is_default, filter_payload
         FROM load_board_saved_filters
         WHERE deleted_at IS NULL
            AND organization_id = $1
            AND (user_id = $2 OR (user_id IS NULL AND ($3::text IS NULL OR role_key = $3)))
         ORDER BY is_default DESC, name ASC",
    )
    .bind(organization_id)
    .bind(user_id)
    .bind(role_key)
    .fetch_all(pool)
    .await
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
                bid_status, price::double precision AS price, status_id, booked_carrier_id, booked_at, booked_amount::double precision AS booked_amount, accepted_offer_id,
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
            l.organization_id,
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
    idempotency_key: Option<&str>,
) -> Result<Option<LoadLegRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let normalized_idempotency_key = idempotency_key
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(key) = normalized_idempotency_key {
        let inserted = sqlx::query_scalar::<_, i64>(
            "INSERT INTO booking_idempotency_keys (carrier_user_id, load_leg_id, idempotency_key, created_at)
             VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
             ON CONFLICT (carrier_user_id, idempotency_key) DO NOTHING
             RETURNING load_leg_id",
        )
        .bind(carrier_id)
        .bind(leg_id)
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;

        if inserted.is_none() {
            let existing_leg_id = sqlx::query_scalar::<_, i64>(
                "SELECT load_leg_id
                 FROM booking_idempotency_keys
                 WHERE carrier_user_id = $1 AND idempotency_key = $2
                 LIMIT 1",
            )
            .bind(carrier_id)
            .bind(key)
            .fetch_one(&mut *tx)
            .await?;

            if existing_leg_id != leg_id {
                return Err(sqlx::Error::Protocol(
                    "Booking idempotency key was already used for a different load leg.".into(),
                ));
            }

            let existing = sqlx::query_as::<_, LoadLegRecord>(
                "SELECT id, load_id, leg_no, leg_code, pickup_location_id, delivery_location_id, pickup_date, delivery_date,
                        bid_status, price::double precision AS price, status_id, booked_carrier_id, booked_at, booked_amount::double precision AS booked_amount, accepted_offer_id,
                        created_at, updated_at, deleted_at
                 FROM load_legs
                 WHERE deleted_at IS NULL AND id = $1
                 LIMIT 1",
            )
            .bind(leg_id)
            .fetch_optional(&mut *tx)
            .await?;

            tx.commit().await?;
            return Ok(existing);
        }
    }

    let Some(leg) = sqlx::query_as::<_, LoadLegRecord>(
        "SELECT id, load_id, leg_no, leg_code, pickup_location_id, delivery_location_id, pickup_date, delivery_date,
                bid_status, price::double precision AS price, status_id, booked_carrier_id, booked_at, booked_amount::double precision AS booked_amount, accepted_offer_id,
                created_at, updated_at, deleted_at
         FROM load_legs
         WHERE deleted_at IS NULL AND id = $1
         LIMIT 1
         FOR UPDATE",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let resolved_amount = booked_amount.or(leg.booked_amount).or(leg.price);

    if leg.booked_carrier_id == Some(carrier_id) {
        tx.commit().await?;
        return Ok(Some(leg));
    }

    if leg.booked_carrier_id.is_some() || leg.status_id >= 4 {
        return Err(sqlx::Error::Protocol(
            "Load leg is already booked or locked for booking.".into(),
        ));
    }

    let compliance_blocker = sqlx::query_scalar::<_, Option<String>>(
        "WITH verification AS (
             SELECT authority_status, insurance_status, insurance_expires_at
             FROM carrier_authority_verifications
             WHERE carrier_user_id = $1
             LIMIT 1
         )
         SELECT CASE
             WHEN NOT EXISTS (SELECT 1 FROM verification)
                 THEN 'Carrier FMCSA/DOT/MC and insurance verification is missing.'
             WHEN EXISTS (SELECT 1 FROM verification WHERE authority_status <> 'active')
                 THEN 'Carrier operating authority is not active.'
             WHEN EXISTS (SELECT 1 FROM verification WHERE insurance_status <> 'verified')
                 THEN 'Carrier insurance is not verified.'
             WHEN EXISTS (SELECT 1 FROM verification WHERE insurance_expires_at IS NULL OR insurance_expires_at < CURRENT_DATE)
                 THEN 'Carrier insurance is expired or missing an expiration date.'
             ELSE NULL
         END",
    )
    .bind(carrier_id)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(reason) = compliance_blocker {
        return Err(sqlx::Error::Protocol(reason));
    }

    let driver_equipment_blocker = sqlx::query_scalar::<_, Option<String>>(
        "WITH safety AS (
             SELECT *
             FROM driver_equipment_safety_compliance
             WHERE carrier_user_id = $1
             LIMIT 1
         )
         SELECT CASE
             WHEN NOT EXISTS (SELECT 1 FROM safety) THEN NULL
             WHEN EXISTS (SELECT 1 FROM safety WHERE restricted_freight_blocking = TRUE)
                 THEN 'Carrier driver/equipment safety review blocks restricted freight.'
             WHEN EXISTS (SELECT 1 FROM safety WHERE driver_compliance_status IN ('expired', 'blocked'))
                 THEN 'Carrier driver qualification is expired or blocked.'
             WHEN EXISTS (SELECT 1 FROM safety WHERE equipment_compliance_status IN ('expired', 'blocked'))
                 THEN 'Carrier equipment compliance is expired or blocked.'
             WHEN EXISTS (SELECT 1 FROM safety WHERE cdl_expires_at IS NOT NULL AND cdl_expires_at < CURRENT_DATE)
                 THEN 'Carrier CDL evidence is expired.'
             WHEN EXISTS (SELECT 1 FROM safety WHERE medical_card_expires_at IS NOT NULL AND medical_card_expires_at < CURRENT_DATE)
                 THEN 'Carrier medical card evidence is expired.'
             WHEN EXISTS (SELECT 1 FROM safety WHERE inspection_expires_at IS NOT NULL AND inspection_expires_at < CURRENT_DATE)
                 THEN 'Carrier equipment inspection is expired.'
             WHEN EXISTS (SELECT 1 FROM safety WHERE maintenance_status IN ('overdue', 'blocked'))
                 THEN 'Carrier equipment maintenance is overdue or blocked.'
             WHEN EXISTS (SELECT 1 FROM safety WHERE equipment_insurance_status IN ('expired', 'rejected', 'missing'))
                 THEN 'Carrier equipment insurance is expired, rejected, or missing.'
             ELSE NULL
         END",
    )
    .bind(carrier_id)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(reason) = driver_equipment_blocker {
        return Err(sqlx::Error::Protocol(reason));
    }

    let sanctions_tax_blocker = sqlx::query_scalar::<_, Option<String>>(
        "WITH profile AS (
             SELECT sanctions_status, beneficial_owner_status
             FROM sanctions_tax_profiles
             WHERE user_id = $1
             LIMIT 1
         )
         SELECT CASE
             WHEN NOT EXISTS (SELECT 1 FROM profile) THEN NULL
             WHEN EXISTS (SELECT 1 FROM profile WHERE sanctions_status IN ('possible_match', 'blocked'))
                 THEN 'Carrier has an unresolved sanctions screening result.'
             WHEN EXISTS (SELECT 1 FROM profile WHERE beneficial_owner_status = 'blocked')
                 THEN 'Carrier beneficial owner check is blocked.'
             ELSE NULL
         END",
    )
    .bind(carrier_id)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(reason) = sanctions_tax_blocker {
        return Err(sqlx::Error::Protocol(reason));
    }

    let risk_review_blocker = sqlx::query_scalar::<_, Option<String>>(
        "SELECT CONCAT('Booking blocked by ', review_type, ' review: ', array_to_string(reasons, '; '))
         FROM risk_review_items
         WHERE hold_booking = TRUE
           AND status IN ('open', 'in_review', 'blocked')
           AND (subject_user_id = $1 OR leg_id = $2)
         ORDER BY
           CASE severity WHEN 'critical' THEN 4 WHEN 'high' THEN 3 WHEN 'medium' THEN 2 ELSE 1 END DESC,
           score DESC,
           created_at DESC
         LIMIT 1",
    )
    .bind(carrier_id)
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    .flatten();

    if let Some(reason) = risk_review_blocker {
        return Err(sqlx::Error::Protocol(reason));
    }

    let update_result = sqlx::query(
        "UPDATE load_legs
         SET booked_carrier_id = $1,
             booked_amount = $2,
             booked_at = COALESCE(booked_at, CURRENT_TIMESTAMP),
             status_id = CASE WHEN status_id < 4 THEN 4 ELSE status_id END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3 AND booked_carrier_id IS NULL AND status_id < 4",
    )
    .bind(carrier_id)
    .bind(resolved_amount)
    .bind(leg_id)
    .execute(&mut *tx)
    .await?;

    if update_result.rows_affected() != 1 {
        return Err(sqlx::Error::Protocol(
            "Load leg booking lost the race to another carrier.".into(),
        ));
    }

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
                bid_status, price::double precision AS price, status_id, booked_carrier_id, booked_at, booked_amount::double precision AS booked_amount, accepted_offer_id,
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
            equipment.name AS equipment_name,
            commodity.name AS commodity_type_name,
            l.freight_mode,
            l.service_level,
            pickup.name AS pickup_location_name,
            delivery.name AS delivery_location_name,
            ll.pickup_date,
            ll.delivery_date,
            ll.bid_status,
            ll.price::double precision AS price,
            ll.status_id,
            ll.booked_carrier_id,
            carrier.name AS booked_carrier_name,
            ll.booked_amount::double precision AS booked_amount,
            escrow.status AS escrow_status,
            handoff.status AS stloads_status,
            handoff.retry_count AS stloads_retry_count,
            sync_issue.title AS stloads_alert_title,
            ll.created_at
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        LEFT JOIN equipments equipment ON equipment.id = l.equipment_id
        LEFT JOIN commodity_types commodity ON commodity.id = l.commodity_type_id
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

pub async fn list_dispatch_desk_legs_filtered(
    pool: &DbPool,
    owner_user_id: Option<i64>,
    status_ids: &[i16],
    desk_key: &str,
    limit: i64,
) -> Result<Vec<DispatchDeskLegRecord>, sqlx::Error> {
    sqlx::query_as::<_, DispatchDeskLegRecord>(
        r#"
        SELECT
            ll.id AS leg_id,
            ll.load_id,
            handoff.id AS handoff_id,
            work.desk_key,
            assigned.name AS assigned_owner_name,
            work.priority,
            work.sla_due_at,
            work.escalation_reason,
            exception.exception_type,
            exception.status AS exception_status,
            exception.severity AS exception_severity,
            (
                SELECT note.note
                FROM dispatch_notes note
                WHERE note.leg_id = ll.id
                  AND note.desk_key = $4
                  AND note.visibility = 'internal'
                ORDER BY note.created_at DESC, note.id DESC
                LIMIT 1
            ) AS latest_internal_note,
            (
                SELECT note.note
                FROM dispatch_notes note
                WHERE note.leg_id = ll.id
                  AND note.desk_key = $4
                  AND note.visibility = 'customer_visible'
                ORDER BY note.created_at DESC, note.id DESC
                LIMIT 1
            ) AS latest_customer_update,
            l.load_number,
            l.title AS load_title,
            equipment.name AS equipment_name,
            l.weight::double precision AS weight,
            ll.status_id,
            ll.booked_carrier_id,
            carrier.name AS booked_carrier_name,
            ll.booked_amount::double precision AS booked_amount,
            escrow.status AS escrow_status,
            handoff.status AS handoff_status,
            (
                SELECT history.remarks
                FROM load_history history
                WHERE history.load_id = l.id
                  AND history.remarks IS NOT NULL
                ORDER BY history.created_at DESC, history.id DESC
                LIMIT 1
            ) AS latest_activity_note,
            ll.created_at
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        LEFT JOIN equipments equipment ON equipment.id = l.equipment_id
        LEFT JOIN users carrier ON carrier.id = ll.booked_carrier_id
        LEFT JOIN escrows escrow ON escrow.leg_id = ll.id
        LEFT JOIN dispatch_work_items work ON work.leg_id = ll.id
            AND work.desk_key = $4
            AND work.status IN ('open', 'in_progress', 'escalated')
        LEFT JOIN users assigned ON assigned.id = work.assigned_user_id
        LEFT JOIN dispatch_exceptions exception ON exception.id = (
            SELECT exception_inner.id
            FROM dispatch_exceptions exception_inner
            WHERE exception_inner.leg_id = ll.id
              AND exception_inner.desk_key = $4
              AND exception_inner.status IN ('open', 'in_progress')
            ORDER BY exception_inner.severity DESC, exception_inner.detected_at DESC, exception_inner.id DESC
            LIMIT 1
        )
        LEFT JOIN stloads_handoffs handoff ON handoff.id = (
            SELECT handoff_inner.id
            FROM stloads_handoffs handoff_inner
            WHERE handoff_inner.load_id = l.id
            ORDER BY handoff_inner.id DESC
            LIMIT 1
        )
        WHERE ll.deleted_at IS NULL
            AND ll.status_id = ANY($1)
            AND ($2::bigint IS NULL OR l.user_id = $2)
        ORDER BY ll.created_at DESC
        LIMIT $3
        "#,
    )
    .bind(status_ids)
    .bind(owner_user_id)
    .bind(limit)
    .bind(desk_key)
    .fetch_all(pool)
    .await
}

pub async fn find_load_id_and_status_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<LoadIdAndStatusRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadIdAndStatusRecord>(
        "SELECT ll.load_id, ll.status_id
         FROM load_legs ll
         INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
         WHERE ll.id = $1
           AND ll.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_load_status_scope(
    pool: &DbPool,
    load_id: i64,
) -> Result<Option<LoadStatusScopeRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadStatusScopeRecord>(
        "SELECT id AS load_id, user_id AS owner_user_id
         FROM loads
         WHERE id = $1
           AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(pool)
    .await
}

pub async fn review_load_status(
    pool: &DbPool,
    load_id: i64,
    status_id: i16,
    remarks: Option<&str>,
    actor_user_id: Option<i64>,
) -> Result<Option<LoadStatusScopeRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(scope) = sqlx::query_as::<_, LoadStatusScopeRecord>(
        "SELECT id AS load_id, user_id AS owner_user_id
         FROM loads
         WHERE id = $1
           AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "UPDATE load_legs
         SET status_id = $1,
             updated_at = CURRENT_TIMESTAMP
         WHERE load_id = $2
           AND deleted_at IS NULL",
    )
    .bind(status_id)
    .bind(load_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE loads
         SET status = $1,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(status_id)
    .bind(load_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(actor_user_id)
    .bind(status_id)
    .bind(remarks.map(str::to_string))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(scope))
}

pub async fn append_dispatch_desk_follow_up(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    desk_key: &str,
    note: &str,
    visibility: &str,
) -> Result<Option<LoadIdAndStatusRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(load_row) = sqlx::query_as::<_, LoadIdAndStatusRecord>(
        "SELECT ll.load_id, ll.status_id
         FROM load_legs ll
         INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
         WHERE ll.id = $1
           AND ll.deleted_at IS NULL
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
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_row.load_id)
    .bind(actor_user_id)
    .bind(load_row.status_id)
    .bind(format!(
        "Rust {} desk follow-up on leg {}: {}",
        desk_key.trim(),
        leg_id,
        note.trim()
    ))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO dispatch_notes (leg_id, load_id, desk_key, visibility, note, created_by_user_id, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .bind(load_row.load_id)
    .bind(desk_key.trim())
    .bind(visibility)
    .bind(note.trim())
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(load_row))
}

pub async fn resolve_dispatch_exception(
    pool: &DbPool,
    leg_id: i64,
    actor_user_id: Option<i64>,
    desk_key: &str,
    exception_type: Option<&str>,
    resolution_note: &str,
) -> Result<Option<LoadIdAndStatusRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(load_row) = sqlx::query_as::<_, LoadIdAndStatusRecord>(
        "SELECT ll.load_id, ll.status_id
         FROM load_legs ll
         INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
         WHERE ll.id = $1
           AND ll.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let result = sqlx::query(
        "UPDATE dispatch_exceptions
         SET status = 'resolved',
             resolved_at = CURRENT_TIMESTAMP,
             resolved_by_user_id = $3,
             resolution_note = $4,
             updated_at = CURRENT_TIMESTAMP
         WHERE leg_id = $1
           AND desk_key = $2
           AND status IN ('open', 'in_progress')
           AND ($5::text IS NULL OR exception_type = $5)",
    )
    .bind(leg_id)
    .bind(desk_key.trim())
    .bind(actor_user_id)
    .bind(resolution_note.trim())
    .bind(
        exception_type
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO dispatch_notes (leg_id, load_id, desk_key, visibility, note, created_by_user_id, created_at)
         VALUES ($1, $2, $3, 'internal', $4, $5, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .bind(load_row.load_id)
    .bind(desk_key.trim())
    .bind(format!(
        "Resolved {} exception: {}",
        exception_type.unwrap_or("active"),
        resolution_note.trim()
    ))
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO dispatch_exceptions (
                leg_id, load_id, desk_key, exception_type, status, severity,
                resolved_at, resolution_note, resolved_by_user_id, created_at, updated_at
             )
             VALUES ($1, $2, $3, $4, 'resolved', 'low', CURRENT_TIMESTAMP, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(leg_id)
        .bind(load_row.load_id)
        .bind(desk_key.trim())
        .bind(exception_type.unwrap_or("manual_resolution"))
        .bind(resolution_note.trim())
        .bind(actor_user_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(Some(load_row))
}

pub async fn count_dispatch_desk_legs_filtered(
    pool: &DbPool,
    owner_user_id: Option<i64>,
    status_ids: &[i16],
) -> Result<i64, sqlx::Error> {
    let (total,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM load_legs ll
        INNER JOIN loads l ON l.id = ll.load_id AND l.deleted_at IS NULL
        WHERE ll.deleted_at IS NULL
            AND ll.status_id = ANY($1)
            AND ($2::bigint IS NULL OR l.user_id = $2)
        "#,
    )
    .bind(status_ids)
    .bind(owner_user_id)
    .fetch_one(pool)
    .await?;

    Ok(total)
}

pub async fn create_load_with_legs(
    pool: &DbPool,
    params: &CreateLoadParams,
    legs: &[CreateLoadLegParams],
    actor_user_id: Option<i64>,
) -> Result<CreatedLoadRecord, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let created_load: (i64,) = sqlx::query_as(
        "INSERT INTO loads (
            title,
            user_id,
            load_type_id,
            equipment_id,
            commodity_type_id,
            customer_contract_id,
            customer_contract_lane_id,
            contract_rate,
            contract_rate_currency,
            contract_posting_behavior,
            contract_service_rules,
            freight_mode,
            visibility,
            service_level,
            customer_reference,
            po_number,
            pickup_appointment_ref,
            delivery_appointment_ref,
            facility_contact_name,
            facility_contact_phone,
            facility_contact_email,
            appointment_window_start,
            appointment_window_end,
            accessorial_flags,
            weight_unit,
            weight,
            temperature_data,
            container_data,
            securement_data,
            special_instructions,
            is_hazardous,
            is_temperature_controlled,
            status,
            leg_count,
            created_at,
            updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                 $11, $12, $13, $14, $15, $16, $17, $18,
                 $19, $20, $21, $22, $23, $24, $25, $26,
                 $27, $28, $29, $30, $31, $32, 1, $33, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(&params.title)
    .bind(params.owner_user_id)
    .bind(params.load_type_id)
    .bind(params.equipment_id)
    .bind(params.commodity_type_id)
    .bind(params.customer_contract_id)
    .bind(params.customer_contract_lane_id)
    .bind(params.contract_rate)
    .bind(&params.contract_rate_currency)
    .bind(&params.contract_posting_behavior)
    .bind(&params.contract_service_rules)
    .bind(&params.freight_mode)
    .bind(&params.visibility)
    .bind(&params.service_level)
    .bind(&params.customer_reference)
    .bind(&params.po_number)
    .bind(&params.pickup_appointment_ref)
    .bind(&params.delivery_appointment_ref)
    .bind(&params.facility_contact_name)
    .bind(&params.facility_contact_phone)
    .bind(&params.facility_contact_email)
    .bind(params.appointment_window_start)
    .bind(params.appointment_window_end)
    .bind(&params.accessorial_flags)
    .bind(&params.weight_unit)
    .bind(params.weight)
    .bind(&params.temperature_data)
    .bind(&params.container_data)
    .bind(&params.securement_data)
    .bind(&params.special_instructions)
    .bind(params.is_hazardous)
    .bind(params.is_temperature_controlled)
    .bind(legs.len() as i32)
    .fetch_one(&mut *tx)
    .await?;

    let load_id = created_load.0;
    let load_number = format!("RUST-LD-{:06}", load_id.max(0));

    sqlx::query(
        "UPDATE loads
         SET load_number = $1,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(&load_number)
    .bind(load_id)
    .execute(&mut *tx)
    .await?;

    for (index, leg) in legs.iter().enumerate() {
        let leg_no = (index + 1) as i32;
        let leg_code = format!("{}-{}", load_number, leg_no);

        sqlx::query(
            "INSERT INTO load_legs (
                load_id,
                leg_no,
                leg_code,
                pickup_location_id,
                delivery_location_id,
                pickup_date,
                delivery_date,
                bid_status,
                price,
                status_id,
                created_at,
                updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(load_id)
        .bind(leg_no)
        .bind(&leg_code)
        .bind(leg.pickup_location_id)
        .bind(leg.delivery_location_id)
        .bind(leg.pickup_date)
        .bind(leg.delivery_date)
        .bind(&leg.bid_status)
        .bind(leg.price)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, 1, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(actor_user_id)
    .bind("Rust load builder created load")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(CreatedLoadRecord {
        load_id,
        load_number,
        leg_count: legs.len() as u64,
    })
}
pub async fn update_load_with_legs(
    pool: &DbPool,
    load_id: i64,
    params: &CreateLoadParams,
    legs: &[CreateLoadLegParams],
    actor_user_id: Option<i64>,
) -> Result<Option<CreatedLoadRecord>, sqlx::Error> {
    #[derive(Debug, FromRow)]
    struct ExistingLoadRow {
        load_number: Option<String>,
        status: i16,
    }

    let mut tx = pool.begin().await?;

    let Some(existing_load) = sqlx::query_as::<_, ExistingLoadRow>(
        "SELECT load_number, status
         FROM loads
         WHERE deleted_at IS NULL AND id = $1
         LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "UPDATE loads
         SET title = $1,
             user_id = $2,
             load_type_id = $3,
             equipment_id = $4,
             commodity_type_id = $5,
             customer_contract_id = $6,
             customer_contract_lane_id = $7,
             contract_rate = $8,
             contract_rate_currency = $9,
             contract_posting_behavior = $10,
             contract_service_rules = $11,
             freight_mode = $12,
             visibility = $13,
             service_level = $14,
             customer_reference = $15,
             po_number = $16,
             pickup_appointment_ref = $17,
             delivery_appointment_ref = $18,
             facility_contact_name = $19,
             facility_contact_phone = $20,
             facility_contact_email = $21,
             appointment_window_start = $22,
             appointment_window_end = $23,
             accessorial_flags = $24,
             weight_unit = $25,
             weight = $26,
             temperature_data = $27,
             container_data = $28,
             securement_data = $29,
             special_instructions = $30,
             is_hazardous = $31,
             is_temperature_controlled = $32,
             leg_count = $33,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $34",
    )
    .bind(&params.title)
    .bind(params.owner_user_id)
    .bind(params.load_type_id)
    .bind(params.equipment_id)
    .bind(params.commodity_type_id)
    .bind(params.customer_contract_id)
    .bind(params.customer_contract_lane_id)
    .bind(params.contract_rate)
    .bind(&params.contract_rate_currency)
    .bind(&params.contract_posting_behavior)
    .bind(&params.contract_service_rules)
    .bind(&params.freight_mode)
    .bind(&params.visibility)
    .bind(&params.service_level)
    .bind(&params.customer_reference)
    .bind(&params.po_number)
    .bind(&params.pickup_appointment_ref)
    .bind(&params.delivery_appointment_ref)
    .bind(&params.facility_contact_name)
    .bind(&params.facility_contact_phone)
    .bind(&params.facility_contact_email)
    .bind(params.appointment_window_start)
    .bind(params.appointment_window_end)
    .bind(&params.accessorial_flags)
    .bind(&params.weight_unit)
    .bind(params.weight)
    .bind(&params.temperature_data)
    .bind(&params.container_data)
    .bind(&params.securement_data)
    .bind(&params.special_instructions)
    .bind(params.is_hazardous)
    .bind(params.is_temperature_controlled)
    .bind(legs.len() as i32)
    .bind(load_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE load_legs
         SET deleted_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE load_id = $1 AND deleted_at IS NULL",
    )
    .bind(load_id)
    .execute(&mut *tx)
    .await?;

    let load_number = existing_load
        .load_number
        .unwrap_or_else(|| format!("RUST-LD-{:06}", load_id.max(0)));

    for (index, leg) in legs.iter().enumerate() {
        let leg_no = (index + 1) as i32;
        let leg_code = format!("{}-{}", load_number, leg_no);

        sqlx::query(
            "INSERT INTO load_legs (
                load_id,
                leg_no,
                leg_code,
                pickup_location_id,
                delivery_location_id,
                pickup_date,
                delivery_date,
                bid_status,
                price,
                status_id,
                created_at,
                updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(load_id)
        .bind(leg_no)
        .bind(&leg_code)
        .bind(leg.pickup_location_id)
        .bind(leg.delivery_location_id)
        .bind(leg.pickup_date)
        .bind(leg.delivery_date)
        .bind(&leg.bid_status)
        .bind(leg.price)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(actor_user_id)
    .bind(existing_load.status)
    .bind("Rust load builder updated load")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Some(CreatedLoadRecord {
        load_id,
        load_number,
        leg_count: legs.len() as u64,
    }))
}

pub async fn update_load_lifecycle(
    pool: &DbPool,
    load_id: i64,
    lifecycle_status: &str,
    reason: Option<&str>,
    template_name: Option<&str>,
    actor_user_id: Option<i64>,
) -> Result<Option<LoadRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let timestamp_column = match lifecycle_status {
        "published" => "published_at = CURRENT_TIMESTAMP,",
        "revised" => "revised_at = CURRENT_TIMESTAMP, revision_number = revision_number + 1,",
        "cancelled" => "cancelled_at = CURRENT_TIMESTAMP,",
        "archived" => "archived_at = CURRENT_TIMESTAMP,",
        _ => "",
    };
    let sql = format!(
        "UPDATE loads
         SET lifecycle_status = $1,
             {timestamp_column}
             lifecycle_reason = $2,
             is_template = CASE WHEN $3::text IS NULL THEN is_template ELSE TRUE END,
             template_name = COALESCE($3, template_name),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4 AND deleted_at IS NULL
         RETURNING id"
    );

    let updated: Option<(i64,)> = sqlx::query_as(&sql)
        .bind(lifecycle_status)
        .bind(reason)
        .bind(template_name)
        .bind(load_id)
        .fetch_optional(&mut *tx)
        .await?;

    let Some(_) = updated else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, 1, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(actor_user_id)
    .bind(format!(
        "Load lifecycle moved to {}{}",
        lifecycle_status,
        reason
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!(": {}", value.trim()))
            .unwrap_or_default()
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_load_by_id(pool, load_id).await
}

pub async fn clone_load_as_draft(
    pool: &DbPool,
    source_load_id: i64,
    actor_user_id: Option<i64>,
) -> Result<Option<CreatedLoadRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let cloned_id: Option<(i64,)> = sqlx::query_as(
        "INSERT INTO loads (
            organization_id, title, user_id, load_type_id, equipment_id, commodity_type_id,
            customer_contract_id, customer_contract_lane_id, contract_rate,
            contract_rate_currency, contract_posting_behavior, contract_service_rules,
            freight_mode, visibility, service_level, customer_reference, po_number,
            pickup_appointment_ref, delivery_appointment_ref, facility_contact_name,
            facility_contact_phone, facility_contact_email, appointment_window_start,
            appointment_window_end, accessorial_flags, weight_unit, weight, temperature_data,
            container_data, securement_data, special_instructions, is_hazardous,
            is_temperature_controlled, lifecycle_status, revision_number, cloned_from_load_id,
            is_template, status, leg_count, created_at, updated_at
         )
         SELECT organization_id, title || ' (Copy)', user_id, load_type_id, equipment_id,
            commodity_type_id, customer_contract_id, customer_contract_lane_id, contract_rate,
            contract_rate_currency, contract_posting_behavior, contract_service_rules,
            freight_mode, 'private', service_level, customer_reference,
            po_number, pickup_appointment_ref, delivery_appointment_ref, facility_contact_name,
            facility_contact_phone, facility_contact_email, appointment_window_start,
            appointment_window_end, accessorial_flags, weight_unit, weight, temperature_data,
            container_data, securement_data, special_instructions, is_hazardous,
            is_temperature_controlled, 'draft', 1, id, FALSE, 1, leg_count,
            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         FROM loads
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id",
    )
    .bind(source_load_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((load_id,)) = cloned_id else {
        tx.rollback().await?;
        return Ok(None);
    };

    let load_number = format!("RUST-LD-{:06}", load_id.max(0));
    sqlx::query("UPDATE loads SET load_number = $1 WHERE id = $2")
        .bind(&load_number)
        .bind(load_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO load_legs (
            load_id, leg_no, leg_code, pickup_location_id, delivery_location_id,
            pickup_date, delivery_date, bid_status, price, status_id, created_at, updated_at
         )
         SELECT $1, leg_no, $2 || '-' || leg_no::text, pickup_location_id, delivery_location_id,
            pickup_date, delivery_date, bid_status, price, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         FROM load_legs
         WHERE load_id = $3 AND deleted_at IS NULL
         ORDER BY leg_no, id",
    )
    .bind(load_id)
    .bind(&load_number)
    .bind(source_load_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, 1, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(actor_user_id)
    .bind(format!("Cloned from load #{}", source_load_id))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(CreatedLoadRecord {
        load_id,
        load_number,
        leg_count: 0,
    }))
}
