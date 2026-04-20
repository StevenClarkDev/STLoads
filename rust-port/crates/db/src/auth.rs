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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermissionRecord {
    pub role_id: i64,
    pub permission_id: i64,
    pub permission_name: String,
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

pub async fn list_role_permissions(
    pool: &DbPool,
) -> Result<Vec<RolePermissionRecord>, sqlx::Error> {
    sqlx::query_as::<_, RolePermissionRecord>(
        "SELECT
            rhp.role_id,
            rhp.permission_id,
            p.name AS permission_name
         FROM role_has_permissions rhp
         INNER JOIN permissions p ON p.id = rhp.permission_id
         ORDER BY rhp.role_id, p.name",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_permission_names_for_role(
    pool: &DbPool,
    role_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT p.name
         FROM role_has_permissions rhp
         INNER JOIN permissions p ON p.id = rhp.permission_id
         WHERE rhp.role_id = $1
         ORDER BY p.name",
    )
    .bind(role_id)
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

pub async fn list_user_ids_for_role(pool: &DbPool, role_id: i64) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM users
         WHERE role_id = $1
         ORDER BY id",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await
}

pub async fn replace_role_permissions(
    pool: &DbPool,
    role_id: i64,
    permission_keys: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM role_has_permissions WHERE role_id = $1")
        .bind(role_id)
        .execute(&mut *tx)
        .await?;

    if !permission_keys.is_empty() {
        let permission_ids = sqlx::query_scalar::<_, i64>(
            "SELECT id
             FROM permissions
             WHERE name = ANY($1)
             ORDER BY id",
        )
        .bind(permission_keys)
        .fetch_all(&mut *tx)
        .await?;

        for permission_id in permission_ids {
            sqlx::query(
                "INSERT INTO role_has_permissions (permission_id, role_id)
                 VALUES ($1, $2)
                 ON CONFLICT DO NOTHING",
            )
            .bind(permission_id)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    list_permission_names_for_role(pool, role_id).await
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role_id: i16,
    pub phone_no: Option<String>,
    pub address: Option<String>,
    pub otp: String,
    pub otp_expires_at: NaiveDateTime,
}

pub async fn create_registered_user(
    pool: &DbPool,
    input: &CreateUserInput,
) -> Result<UserRecord, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let user_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users
            (name, email, password, role_id, phone_no, address, otp, otp_expires_at, otp_resend_count, last_otp_resend_at, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, CURRENT_TIMESTAMP, 4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(&input.name)
    .bind(&input.email)
    .bind(&input.password_hash)
    .bind(input.role_id)
    .bind(&input.phone_no)
    .bind(&input.address)
    .bind(&input.otp)
    .bind(input.otp_expires_at)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO model_has_roles (role_id, model_type, model_id)
         VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(i64::from(input.role_id))
    .bind("App\\Models\\User")
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    find_user_by_id(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn refresh_user_otp(
    pool: &DbPool,
    user_id: i64,
    otp: &str,
    otp_expires_at: NaiveDateTime,
    resend_count: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users
         SET otp = $1,
             otp_expires_at = $2,
             otp_resend_count = $3,
             last_otp_resend_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4",
    )
    .bind(otp)
    .bind(otp_expires_at)
    .bind(resend_count)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn consume_registration_otp(
    pool: &DbPool,
    email: &str,
    otp: &str,
    verified_status: i16,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let user_id = sqlx::query_scalar::<_, i64>(
        "UPDATE users
         SET email_verified_at = CURRENT_TIMESTAMP,
             otp = NULL,
             otp_expires_at = NULL,
             updated_at = CURRENT_TIMESTAMP,
             status = $3
         WHERE email = $1
           AND otp = $2
           AND otp_expires_at > CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(email)
    .bind(otp)
    .bind(verified_status)
    .fetch_optional(pool)
    .await?;

    match user_id {
        Some(user_id) => find_user_by_id(pool, user_id).await,
        None => Ok(None),
    }
}

pub async fn consume_password_reset_otp(
    pool: &DbPool,
    email: &str,
    otp: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let user_id = sqlx::query_scalar::<_, i64>(
        "UPDATE users
         SET otp = NULL,
             otp_expires_at = NULL,
             updated_at = CURRENT_TIMESTAMP
         WHERE email = $1
           AND otp = $2
           AND otp_expires_at > CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(email)
    .bind(otp)
    .fetch_optional(pool)
    .await?;

    match user_id {
        Some(user_id) => find_user_by_id(pool, user_id).await,
        None => Ok(None),
    }
}

pub async fn store_password_reset_token(
    pool: &DbPool,
    email: &str,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO password_reset_tokens (email, token, created_at)
         VALUES ($1, $2, CURRENT_TIMESTAMP)
         ON CONFLICT (email) DO UPDATE
         SET token = EXCLUDED.token,
             created_at = EXCLUDED.created_at",
    )
    .bind(email)
    .bind(token)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn consume_password_reset_token(
    pool: &DbPool,
    email: &str,
    token: &str,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let matched_email = sqlx::query_scalar::<_, String>(
        "SELECT email
         FROM password_reset_tokens
         WHERE email = $1
           AND token = $2
           AND created_at > CURRENT_TIMESTAMP - INTERVAL '30 minutes'
         LIMIT 1",
    )
    .bind(email)
    .bind(token)
    .fetch_optional(&mut *tx)
    .await?;

    if matched_email.is_none() {
        tx.rollback().await?;
        return Ok(false);
    }

    sqlx::query(
        "UPDATE users
         SET password = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE email = $1",
    )
    .bind(email)
    .bind(password_hash)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM password_reset_tokens WHERE email = $1")
        .bind(email)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertUserOnboardingInput {
    pub user_id: i64,
    pub company_name: String,
    pub company_address: String,
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
    pub next_status: i16,
}

pub async fn find_user_detail_by_user_id(
    pool: &DbPool,
    user_id: i64,
) -> Result<Option<UserDetailRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserDetailRecord>(
        "SELECT id, user_id, company_name, company_address, dot_number, mc_number, equipment_types, business_entity_id, facility_address, fulfillment_contact_info, fmcsa_broker_license_no, mc_authority_number, freight_forwarder_license, customs_license, created_at, updated_at
         FROM user_details
         WHERE user_id = $1
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_user_onboarding_details(
    pool: &DbPool,
    input: &UpsertUserOnboardingInput,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO user_details
            (user_id, company_name, company_address, dot_number, mc_number, equipment_types, business_entity_id, facility_address, fulfillment_contact_info, fmcsa_broker_license_no, mc_authority_number, freight_forwarder_license, customs_license, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (user_id) DO UPDATE SET
            company_name = EXCLUDED.company_name,
            company_address = EXCLUDED.company_address,
            dot_number = EXCLUDED.dot_number,
            mc_number = EXCLUDED.mc_number,
            equipment_types = EXCLUDED.equipment_types,
            business_entity_id = EXCLUDED.business_entity_id,
            facility_address = EXCLUDED.facility_address,
            fulfillment_contact_info = EXCLUDED.fulfillment_contact_info,
            fmcsa_broker_license_no = EXCLUDED.fmcsa_broker_license_no,
            mc_authority_number = EXCLUDED.mc_authority_number,
            freight_forwarder_license = EXCLUDED.freight_forwarder_license,
            customs_license = EXCLUDED.customs_license,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(input.user_id)
    .bind(&input.company_name)
    .bind(&input.company_address)
    .bind(&input.dot_number)
    .bind(&input.mc_number)
    .bind(&input.equipment_types)
    .bind(&input.business_entity_id)
    .bind(&input.facility_address)
    .bind(&input.fulfillment_contact_info)
    .bind(&input.fmcsa_broker_license_no)
    .bind(&input.mc_authority_number)
    .bind(&input.freight_forwarder_license)
    .bind(&input.customs_license)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE users
         SET company_name = $2,
             company_address = $3,
             mc_number = COALESCE($4, mc_number),
             status = $5,
             kyc_pending_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(input.user_id)
    .bind(&input.company_name)
    .bind(&input.company_address)
    .bind(&input.mc_number)
    .bind(input.next_status)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, NULL, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(input.user_id)
    .bind(input.next_status)
    .bind("Onboarding submitted through the Rust onboarding flow.")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKycDocumentInput {
    pub user_id: i64,
    pub document_name: String,
    pub document_type: String,
    pub file_path: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKycDocumentInput {
    pub document_name: String,
    pub document_type: String,
    pub file_path: Option<String>,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub next_status: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PendingOnboardingUserRecord {
    pub user_id: i64,
    pub name: String,
    pub email: String,
    pub role_id: Option<i16>,
    pub status: i16,
    pub company_name: Option<String>,
    pub company_address: Option<String>,
    pub submitted_at: Option<NaiveDateTime>,
    pub document_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminUserDirectoryRecord {
    pub user_id: i64,
    pub name: String,
    pub email: String,
    pub role_id: Option<i16>,
    pub status: i16,
    pub company_name: Option<String>,
    pub phone_no: Option<String>,
    pub joined_at: NaiveDateTime,
    pub document_count: i64,
    pub latest_review_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminUserHistoryEntryRecord {
    pub id: i64,
    pub status: i16,
    pub remarks: Option<String>,
    pub created_at: NaiveDateTime,
    pub admin_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdminUserInput {
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role_id: i16,
    pub status: i16,
    pub phone_no: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdminUserProfileInput {
    pub user_id: i64,
    pub admin_id: i64,
    pub name: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub phone_no: Option<String>,
    pub address: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSelfProfileInput {
    pub user_id: i64,
    pub name: String,
    pub email: String,
    pub phone_no: Option<String>,
    pub address: Option<String>,
    pub company_name: Option<String>,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub mc_cbsa_usdot_no: Option<String>,
    pub ucr_hcc_no: Option<String>,
    pub password_hash: Option<String>,
    pub status: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSelfPasswordInput {
    pub user_id: i64,
    pub password_hash: String,
    pub status: i16,
    pub remarks: Option<String>,
}

pub async fn list_kyc_documents_by_user_id(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<KycDocumentRecord>, sqlx::Error> {
    sqlx::query_as::<_, KycDocumentRecord>(
        "SELECT id, user_id, document_name, document_type, file_path, original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp, created_at, updated_at
         FROM kyc_documents
         WHERE user_id = $1
         ORDER BY created_at DESC, id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn find_kyc_document_by_id(
    pool: &DbPool,
    document_id: i64,
) -> Result<Option<KycDocumentRecord>, sqlx::Error> {
    sqlx::query_as::<_, KycDocumentRecord>(
        "SELECT id, user_id, document_name, document_type, file_path, original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp, created_at, updated_at
         FROM kyc_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_kyc_document(
    pool: &DbPool,
    input: &CreateKycDocumentInput,
) -> Result<KycDocumentRecord, sqlx::Error> {
    let document_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO kyc_documents
            (user_id, document_name, document_type, file_path, original_name, mime_type, file_size, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(input.user_id)
    .bind(&input.document_name)
    .bind(&input.document_type)
    .bind(&input.file_path)
    .bind(&input.original_name)
    .bind(&input.mime_type)
    .bind(input.file_size)
    .fetch_one(pool)
    .await?;

    sqlx::query_as::<_, KycDocumentRecord>(
        "SELECT id, user_id, document_name, document_type, file_path, original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp, created_at, updated_at
         FROM kyc_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(document_id)
    .fetch_one(pool)
    .await
}

pub async fn update_kyc_document(
    pool: &DbPool,
    document_id: i64,
    user_id: i64,
    input: &UpdateKycDocumentInput,
) -> Result<Option<KycDocumentRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let owned_document_id = sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM kyc_documents
         WHERE id = $1 AND user_id = $2
         LIMIT 1",
    )
    .bind(document_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(owned_document_id) = owned_document_id else {
        tx.rollback().await?;
        return Ok(None);
    };

    if let Some(file_path) = input.file_path.as_deref() {
        sqlx::query(
            "UPDATE kyc_documents
             SET document_name = $1,
                 document_type = $2,
                 file_path = $3,
                 original_name = $4,
                 mime_type = $5,
                 file_size = $6,
                 hash = CASE WHEN $2 = 'standard' THEN NULL ELSE hash END,
                 hash_algorithm = CASE WHEN $2 = 'standard' THEN NULL ELSE hash_algorithm END,
                 mock_blockchain_tx = CASE WHEN $2 = 'standard' THEN NULL ELSE mock_blockchain_tx END,
                 mock_blockchain_timestamp = CASE WHEN $2 = 'standard' THEN NULL ELSE mock_blockchain_timestamp END,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $7 AND user_id = $8",
        )
        .bind(&input.document_name)
        .bind(&input.document_type)
        .bind(file_path)
        .bind(&input.original_name)
        .bind(&input.mime_type)
        .bind(input.file_size)
        .bind(owned_document_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            "UPDATE kyc_documents
             SET document_name = $1,
                 document_type = $2,
                 hash = CASE WHEN $2 = 'standard' THEN NULL ELSE hash END,
                 hash_algorithm = CASE WHEN $2 = 'standard' THEN NULL ELSE hash_algorithm END,
                 mock_blockchain_tx = CASE WHEN $2 = 'standard' THEN NULL ELSE mock_blockchain_tx END,
                 mock_blockchain_timestamp = CASE WHEN $2 = 'standard' THEN NULL ELSE mock_blockchain_timestamp END,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $3 AND user_id = $4",
        )
        .bind(&input.document_name)
        .bind(&input.document_type)
        .bind(owned_document_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }

    mark_user_profile_submission_for_review(
        &mut tx,
        user_id,
        input.next_status,
        &format!(
            "Rust self-service profile updated KYC document {}.",
            input.document_name
        ),
    )
    .await?;

    let updated = sqlx::query_as::<_, KycDocumentRecord>(
        "SELECT id, user_id, document_name, document_type, file_path, original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp, created_at, updated_at
         FROM kyc_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(owned_document_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}

pub async fn verify_kyc_document_blockchain(
    pool: &DbPool,
    document_id: i64,
    user_id: i64,
    note: Option<&str>,
    next_status: i16,
) -> Result<Option<KycDocumentRecord>, sqlx::Error> {
    #[derive(FromRow)]
    struct DocumentRow {
        id: i64,
        user_id: i64,
        document_name: String,
    }

    let mut tx = pool.begin().await?;
    let Some(document) = sqlx::query_as::<_, DocumentRow>(
        "SELECT id, user_id, document_name
         FROM kyc_documents
         WHERE id = $1 AND user_id = $2
         LIMIT 1",
    )
    .bind(document_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let timestamp_token = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let hash = format!("mocksha256-{}-{}", document.id, timestamp_token);
    let tx_id = format!("mocktx-{}-{}", document.user_id, timestamp_token);

    sqlx::query(
        "UPDATE kyc_documents
         SET document_type = 'blockchain',
             hash = $1,
             hash_algorithm = 'mock_sha256',
             mock_blockchain_tx = $2,
             mock_blockchain_timestamp = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3 AND user_id = $4",
    )
    .bind(&hash)
    .bind(&tx_id)
    .bind(document.id)
    .bind(document.user_id)
    .execute(&mut *tx)
    .await?;

    let remark = note
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            format!(
                "Rust self-service profile anchored KYC document {} to blockchain: {}",
                document.document_name, value
            )
        })
        .unwrap_or_else(|| {
            format!(
                "Rust self-service profile anchored KYC document {} to blockchain.",
                document.document_name
            )
        });

    mark_user_profile_submission_for_review(&mut tx, user_id, next_status, &remark).await?;

    let updated = sqlx::query_as::<_, KycDocumentRecord>(
        "SELECT id, user_id, document_name, document_type, file_path, original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp, created_at, updated_at
         FROM kyc_documents
         WHERE id = $1
         LIMIT 1",
    )
    .bind(document.id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}

pub async fn delete_kyc_document(
    pool: &DbPool,
    document_id: i64,
    user_id: i64,
    next_status: i16,
) -> Result<Option<KycDocumentRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, KycDocumentRecord>(
        "SELECT id, user_id, document_name, document_type, file_path, original_name, mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp, created_at, updated_at
         FROM kyc_documents
         WHERE id = $1 AND user_id = $2
         LIMIT 1",
    )
    .bind(document_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(existing) = existing else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query("DELETE FROM kyc_documents WHERE id = $1 AND user_id = $2")
        .bind(document_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    mark_user_profile_submission_for_review(
        &mut tx,
        user_id,
        next_status,
        &format!(
            "Rust self-service profile removed KYC document {}.",
            existing.document_name
        ),
    )
    .await?;

    tx.commit().await?;
    Ok(Some(existing))
}

async fn mark_user_profile_submission_for_review(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: i64,
    next_status: i16,
    remarks: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users
         SET status = $2,
             kyc_pending_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(user_id)
    .bind(next_status)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, NULL, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(user_id)
    .bind(next_status)
    .bind(remarks)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn list_pending_onboarding_users(
    pool: &DbPool,
) -> Result<Vec<PendingOnboardingUserRecord>, sqlx::Error> {
    sqlx::query_as::<_, PendingOnboardingUserRecord>(
        "SELECT
            u.id AS user_id,
            u.name,
            u.email,
            u.role_id,
            u.status,
            COALESCE(ud.company_name, u.company_name) AS company_name,
            COALESCE(ud.company_address, u.company_address) AS company_address,
            COALESCE(u.kyc_pending_at, u.updated_at) AS submitted_at,
            COUNT(kd.id) AS document_count
         FROM users u
         LEFT JOIN user_details ud ON ud.user_id = u.id
         LEFT JOIN kyc_documents kd ON kd.user_id = u.id
         WHERE u.status IN (3, 5)
         GROUP BY u.id, u.name, u.email, u.role_id, u.status, ud.company_name, u.company_name, ud.company_address, u.company_address, u.kyc_pending_at, u.updated_at
         ORDER BY COALESCE(u.kyc_pending_at, u.updated_at) DESC, u.id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn review_onboarding_user(
    pool: &DbPool,
    user_id: i64,
    admin_id: i64,
    next_status: i16,
    remarks: Option<&str>,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let updated_user_id = sqlx::query_scalar::<_, i64>(
        "UPDATE users
         SET status = $2,
             approved_at = CASE WHEN $2 = 1 THEN CURRENT_TIMESTAMP ELSE approved_at END,
             rejected_at = CASE WHEN $2 = 2 THEN CURRENT_TIMESTAMP ELSE rejected_at END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
         RETURNING id",
    )
    .bind(user_id)
    .bind(next_status)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(updated_user_id) = updated_user_id else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(updated_user_id)
    .bind(admin_id)
    .bind(next_status)
    .bind(remarks)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_user_by_id(pool, updated_user_id).await
}

pub async fn list_admin_users(pool: &DbPool) -> Result<Vec<AdminUserDirectoryRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminUserDirectoryRecord>(
        "SELECT
            u.id AS user_id,
            u.name,
            u.email,
            u.role_id,
            u.status,
            COALESCE(ud.company_name, u.company_name) AS company_name,
            u.phone_no,
            u.created_at AS joined_at,
            COUNT(kd.id) AS document_count,
            (
                SELECT history.remarks
                FROM user_history history
                WHERE history.user_id = u.id
                  AND history.remarks IS NOT NULL
                ORDER BY history.id DESC
                LIMIT 1
            ) AS latest_review_note
         FROM users u
         LEFT JOIN user_details ud ON ud.user_id = u.id
         LEFT JOIN kyc_documents kd ON kd.user_id = u.id
         GROUP BY
            u.id,
            u.name,
            u.email,
            u.role_id,
            u.status,
            ud.company_name,
            u.company_name,
            u.phone_no,
            u.created_at
         ORDER BY u.created_at DESC, u.id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_user_history_entries(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<AdminUserHistoryEntryRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminUserHistoryEntryRecord>(
        "SELECT
            history.id,
            history.status,
            history.remarks,
            history.created_at,
            admin_user.name AS admin_name
         FROM user_history history
         LEFT JOIN users admin_user ON admin_user.id = history.admin_id
         WHERE history.user_id = $1
         ORDER BY history.created_at DESC, history.id DESC
         LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn create_admin_user_account(
    pool: &DbPool,
    admin_id: i64,
    input: &CreateAdminUserInput,
) -> Result<UserRecord, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let user_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users
            (name, email, password, role_id, phone_no, address, email_verified_at, status, created_at, updated_at)
         VALUES (
            $1, $2, $3, $4, $5, $6,
            CASE WHEN $7 = 4 THEN NULL ELSE CURRENT_TIMESTAMP END,
            $7,
            CURRENT_TIMESTAMP,
            CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(&input.name)
    .bind(&input.email)
    .bind(&input.password_hash)
    .bind(input.role_id)
    .bind(&input.phone_no)
    .bind(&input.address)
    .bind(input.status)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO model_has_roles (role_id, model_type, model_id)
         VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(i64::from(input.role_id))
    .bind("App\\Models\\User")
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(user_id)
    .bind(admin_id)
    .bind(input.status)
    .bind("Rust admin created the account.")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_user_by_id(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_admin_user_profile(
    pool: &DbPool,
    input: &UpdateAdminUserProfileInput,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let updated_user_id = if let Some(password_hash) = input.password_hash.as_deref() {
        sqlx::query_scalar::<_, i64>(
            "UPDATE users
             SET name = $2,
                 email = $3,
                 password = $4,
                 phone_no = $5,
                 address = $6,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id",
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(password_hash)
        .bind(&input.phone_no)
        .bind(&input.address)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        sqlx::query_scalar::<_, i64>(
            "UPDATE users
             SET name = $2,
                 email = $3,
                 phone_no = $4,
                 address = $5,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id",
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(&input.phone_no)
        .bind(&input.address)
        .fetch_optional(&mut *tx)
        .await?
    };

    let Some(updated_user_id) = updated_user_id else {
        tx.rollback().await?;
        return Ok(None);
    };

    let history_remarks = input
        .remarks
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Rust admin updated the account profile.".into());

    let current_status = sqlx::query_scalar::<_, i16>("SELECT status FROM users WHERE id = $1")
        .bind(updated_user_id)
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(updated_user_id)
    .bind(input.admin_id)
    .bind(current_status)
    .bind(history_remarks)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_user_by_id(pool, updated_user_id).await
}

pub async fn update_self_profile(
    pool: &DbPool,
    input: &UpdateSelfProfileInput,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let updated_user_id = if let Some(password_hash) = input.password_hash.as_deref() {
        sqlx::query_scalar::<_, i64>(
            "UPDATE users
             SET name = $2,
                 email = $3,
                 password = $4,
                 phone_no = $5,
                 address = $6,
                 company_name = $7,
                 mc_number = $8,
                 mc_cbsa_usdot_no = $9,
                 ucr_hcc_no = $10,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id",
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(password_hash)
        .bind(&input.phone_no)
        .bind(&input.address)
        .bind(&input.company_name)
        .bind(&input.mc_number)
        .bind(&input.mc_cbsa_usdot_no)
        .bind(&input.ucr_hcc_no)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        sqlx::query_scalar::<_, i64>(
            "UPDATE users
             SET name = $2,
                 email = $3,
                 phone_no = $4,
                 address = $5,
                 company_name = $6,
                 mc_number = $7,
                 mc_cbsa_usdot_no = $8,
                 ucr_hcc_no = $9,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id",
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(&input.phone_no)
        .bind(&input.address)
        .bind(&input.company_name)
        .bind(&input.mc_number)
        .bind(&input.mc_cbsa_usdot_no)
        .bind(&input.ucr_hcc_no)
        .fetch_optional(&mut *tx)
        .await?
    };

    let Some(updated_user_id) = updated_user_id else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "INSERT INTO user_details
            (user_id, company_name, dot_number, mc_number, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (user_id) DO UPDATE SET
            company_name = EXCLUDED.company_name,
            dot_number = EXCLUDED.dot_number,
            mc_number = EXCLUDED.mc_number,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(updated_user_id)
    .bind(&input.company_name)
    .bind(&input.dot_number)
    .bind(&input.mc_number)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, NULL, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(updated_user_id)
    .bind(input.status)
    .bind("Profile updated through the Rust self-service profile.")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_user_by_id(pool, updated_user_id).await
}

pub async fn change_self_password(
    pool: &DbPool,
    input: &ChangeSelfPasswordInput,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let updated_user_id = sqlx::query_scalar::<_, i64>(
        "UPDATE users
         SET password = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
         RETURNING id",
    )
    .bind(input.user_id)
    .bind(&input.password_hash)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(updated_user_id) = updated_user_id else {
        tx.rollback().await?;
        return Ok(None);
    };

    let remarks = input
        .remarks
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Password changed through the Rust account security flow.".into());

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, NULL, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(updated_user_id)
    .bind(input.status)
    .bind(remarks)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_user_by_id(pool, updated_user_id).await
}

pub async fn delete_admin_user_account(pool: &DbPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_admin_user_account(
    pool: &DbPool,
    user_id: i64,
    admin_id: i64,
    role_id: i16,
    next_status: i16,
    remarks: Option<&str>,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let updated_user_id = sqlx::query_scalar::<_, i64>(
        "UPDATE users
         SET role_id = $2,
             status = $3,
             approved_at = CASE WHEN $3 = 1 THEN CURRENT_TIMESTAMP ELSE approved_at END,
             rejected_at = CASE WHEN $3 = 2 THEN CURRENT_TIMESTAMP ELSE rejected_at END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
         RETURNING id",
    )
    .bind(user_id)
    .bind(role_id)
    .bind(next_status)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(updated_user_id) = updated_user_id else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "DELETE FROM model_has_roles
         WHERE model_type = $1 AND model_id = $2",
    )
    .bind("App\\Models\\User")
    .bind(updated_user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO model_has_roles (role_id, model_type, model_id)
         VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(i64::from(role_id))
    .bind("App\\Models\\User")
    .bind(updated_user_id)
    .execute(&mut *tx)
    .await?;

    let history_remarks = remarks
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Rust admin user directory updated the account role or status.".into());

    sqlx::query(
        "INSERT INTO user_history (user_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(updated_user_id)
    .bind(admin_id)
    .bind(next_status)
    .bind(history_remarks)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_user_by_id(pool, updated_user_id).await
}
