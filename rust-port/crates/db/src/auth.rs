use chrono::{NaiveDate, NaiveDateTime};
use domain::auth::{AccountStatus, UserRole};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRecord {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role_id: Option<i16>,
    pub dob: Option<NaiveDate>,
    pub gender: Option<String>,
    pub phone_no: Option<String>,
    pub ucr_hcc_no: Option<String>,
    pub mc_cbsa_usdot_no: Option<String>,
    pub ssn_no: Option<String>,
    pub address: Option<String>,
    pub nationality: Option<String>,
    pub company_name: Option<String>,
    pub registration_number: Option<String>,
    pub tax_id: Option<String>,
    pub country_of_incorporation: Option<String>,
    pub consent_sanctions_screening: Option<bool>,
    pub politically_exposed_person: Option<bool>,
    pub source_of_funds: Option<String>,
    pub agree_aml_policies: Option<bool>,
    pub gov_id_number: Option<String>,
    pub cdl_number: Option<String>,
    pub cdl_expiry: Option<NaiveDate>,
    pub cdl_class: Option<String>,
    pub regulatory_country: Option<String>,
    pub usdot_number: Option<String>,
    pub mc_number: Option<String>,
    pub ntn: Option<String>,
    pub vat_number: Option<String>,
    pub insurance_expiry: Option<NaiveDate>,
    pub coverage_limits: Option<String>,
    pub insurer_name: Option<String>,
    pub vehicle_reg: Option<String>,
    pub vehicle_make_model: Option<String>,
    pub vehicle_year: Option<String>,
    pub vehicle_type: Option<String>,
    pub load_capacity: Option<f64>,
    pub company_address: Option<String>,
    pub bank_account: Option<String>,
    pub criminal_declaration: Option<bool>,
    pub terms_agreed: Option<bool>,
    pub trade_name: Option<String>,
    pub incorporation_date: Option<NaiveDate>,
    pub director_name: Option<String>,
    pub director_dob: Option<NaiveDate>,
    pub ubo_name: Option<String>,
    pub ubo_dob: Option<NaiveDate>,
    pub ubo_nationality: Option<String>,
    pub ubo_address: Option<String>,
    pub fmc_license: Option<String>,
    pub nvocc_reg: Option<String>,
    pub surety_bond: Option<String>,
    pub customs_broker_license: Option<String>,
    pub iata_accreditation: Option<String>,
    pub eori_number: Option<String>,
    pub secp_reg: Option<String>,
    pub chamber_reg: Option<String>,
    pub policy_number: Option<String>,
    pub insurer_contact: Option<String>,
    pub transport_modes: Option<String>,
    pub countries_served: Option<String>,
    pub customs_brokerage: Option<bool>,
    pub consolidation_services: Option<bool>,
    pub warehousing: Option<bool>,
    pub years_in_operation: Option<String>,
    pub annual_volume: Option<String>,
    pub monthly_transaction_volume: Option<String>,
    pub ofac_consent: Option<bool>,
    pub otp: Option<String>,
    pub otp_expires_at: Option<NaiveDateTime>,
    pub otp_resend_count: i32,
    pub last_otp_resend_at: Option<NaiveDateTime>,
    pub image: Option<String>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub status: i16,
    pub approved_at: Option<NaiveDateTime>,
    pub rejected_at: Option<NaiveDateTime>,
    pub kyc_pending_at: Option<NaiveDateTime>,
    pub stripe_connect_account_id: Option<String>,
    pub payouts_enabled: bool,
    pub kyc_status: Option<String>,
    pub remember_token: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserRecord {
    pub fn account_status(&self) -> Option<AccountStatus> {
        AccountStatus::from_legacy_code(self.status)
    }

    pub fn primary_role(&self) -> Option<UserRole> {
        self.role_id.and_then(UserRole::from_legacy_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserDetailRecord {
    pub id: i64,
    pub user_id: i64,
    pub company_name: Option<String>,
    pub company_address: Option<String>,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub equipment_types: Option<String>,
    pub business_entity_id: Option<String>,
    pub facility_address: Option<String>,
    pub fulfillment_contact_info: Option<String>,
    pub fmcsa_broker_license_no: Option<String>,
    pub mc_authority_number: Option<String>,
    pub freight_forwarder_license: Option<String>,
    pub customs_license: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KycDocumentRecord {
    pub id: i64,
    pub user_id: i64,
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
pub struct UserHistoryRecord {
    pub id: i64,
    pub user_id: i64,
    pub admin_id: Option<i64>,
    pub status: i16,
    pub remarks: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BrowserSessionRecord {
    pub id: String,
    pub user_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: String,
    pub last_activity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordResetTokenRecord {
    pub email: String,
    pub token: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PersonalAccessTokenRecord {
    pub id: i64,
    pub tokenable_type: String,
    pub tokenable_id: i64,
    pub name: String,
    pub token: String,
    pub abilities: Option<String>,
    pub last_used_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoleRecord {
    pub id: i64,
    pub name: String,
    pub guard_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PermissionRecord {
    pub id: i64,
    pub name: String,
    pub guard_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn find_user_by_email(
    pool: &DbPool,
    email: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>("SELECT * FROM users WHERE email = $1 LIMIT 1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn list_roles(pool: &DbPool) -> Result<Vec<RoleRecord>, sqlx::Error> {
    sqlx::query_as::<_, RoleRecord>(
        "SELECT id, name, guard_name, created_at, updated_at FROM roles ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_permissions(pool: &DbPool) -> Result<Vec<PermissionRecord>, sqlx::Error> {
    sqlx::query_as::<_, PermissionRecord>(
        "SELECT id, name, guard_name, created_at, updated_at FROM permissions ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_user_roles(pool: &DbPool, user_id: i64) -> Result<Vec<RoleRecord>, sqlx::Error> {
    sqlx::query_as::<_, RoleRecord>(
        "SELECT r.id, r.name, r.guard_name, r.created_at, r.updated_at
         FROM roles r
         INNER JOIN model_has_roles mhr ON mhr.role_id = r.id
         WHERE mhr.model_type = $1 AND mhr.model_id = $2
         ORDER BY r.id",
    )
    .bind("App\\Models\\User")
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn list_personal_access_tokens(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<PersonalAccessTokenRecord>, sqlx::Error> {
    sqlx::query_as::<_, PersonalAccessTokenRecord>(
        "SELECT id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
         FROM personal_access_tokens
         WHERE tokenable_type = $1 AND tokenable_id = $2
         ORDER BY id DESC",
    )
    .bind("App\\Models\\User")
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn find_user_by_id(
    pool: &DbPool,
    user_id: i64,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>("SELECT * FROM users WHERE id = $1 LIMIT 1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn find_personal_access_token_exact(
    pool: &DbPool,
    token: &str,
) -> Result<Option<PersonalAccessTokenRecord>, sqlx::Error> {
    sqlx::query_as::<_, PersonalAccessTokenRecord>(
        "SELECT id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
         FROM personal_access_tokens
         WHERE token = $1 AND tokenable_type = $2
         LIMIT 1",
    )
    .bind(token)
    .bind("App\\Models\\User")
    .fetch_optional(pool)
    .await
}

pub async fn insert_personal_access_token(
    pool: &DbPool,
    user_id: i64,
    name: &str,
    token: &str,
    abilities: Option<&str>,
    expires_at: Option<NaiveDateTime>,
) -> Result<PersonalAccessTokenRecord, sqlx::Error> {
    let token_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO personal_access_tokens
            (tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, NULL, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind("App\\Models\\User")
    .bind(user_id)
    .bind(name)
    .bind(token)
    .bind(abilities)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    sqlx::query_as::<_, PersonalAccessTokenRecord>(
        "SELECT id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
         FROM personal_access_tokens
         WHERE id = $1
         LIMIT 1",
    )
    .bind(token_id)
    .fetch_one(pool)
    .await
}

pub async fn touch_personal_access_token(pool: &DbPool, token_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE personal_access_tokens
         SET last_used_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(token_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_personal_access_token_by_token(
    pool: &DbPool,
    token: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM personal_access_tokens
         WHERE token = $1 AND tokenable_type = $2",
    )
    .bind(token)
    .bind("App\\Models\\User")
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
