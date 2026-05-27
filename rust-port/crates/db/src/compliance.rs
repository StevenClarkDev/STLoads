use chrono::{NaiveDate, NaiveDateTime};
use serde_json::Value;
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct ComplianceStatusRecord {
    pub id: i64,
    pub organization_id: Option<i64>,
    pub user_id: Option<i64>,
    pub subject_type: String,
    pub compliance_domain: String,
    pub status: String,
    pub eligibility_blocking: bool,
    pub evidence_reference: Option<String>,
    pub reviewer_user_id: Option<i64>,
    pub reason: Option<String>,
    pub effective_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UpsertComplianceStatusParams<'a> {
    pub organization_id: Option<i64>,
    pub user_id: Option<i64>,
    pub subject_type: &'a str,
    pub compliance_domain: &'a str,
    pub status: &'a str,
    pub eligibility_blocking: bool,
    pub evidence_reference: Option<&'a str>,
    pub reviewer_user_id: Option<i64>,
    pub reason: Option<&'a str>,
    pub effective_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct ComplianceEligibilitySummary {
    pub user_id: i64,
    pub total_records: usize,
    pub approved_records: usize,
    pub blocking_records: usize,
    pub expired_records: usize,
    pub missing_domains: Vec<String>,
    pub eligibility_status: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct CarrierAuthorityVerificationRecord {
    pub id: i64,
    pub organization_id: Option<i64>,
    pub carrier_user_id: i64,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub legal_name: Option<String>,
    pub authority_status: String,
    pub operating_authority_type: Option<String>,
    pub safety_rating: Option<String>,
    pub insurance_status: String,
    pub insurance_provider: Option<String>,
    pub insurance_policy_number: Option<String>,
    pub cargo_coverage_amount_cents: Option<i64>,
    pub liability_coverage_amount_cents: Option<i64>,
    pub currency: String,
    pub insurance_effective_at: Option<NaiveDate>,
    pub insurance_expires_at: Option<NaiveDate>,
    pub verification_source: String,
    pub verified_at: Option<NaiveDateTime>,
    pub reviewed_by_user_id: Option<i64>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UpsertCarrierAuthorityVerificationParams<'a> {
    pub organization_id: Option<i64>,
    pub carrier_user_id: i64,
    pub dot_number: Option<&'a str>,
    pub mc_number: Option<&'a str>,
    pub legal_name: Option<&'a str>,
    pub authority_status: &'a str,
    pub operating_authority_type: Option<&'a str>,
    pub safety_rating: Option<&'a str>,
    pub insurance_status: &'a str,
    pub insurance_provider: Option<&'a str>,
    pub insurance_policy_number: Option<&'a str>,
    pub cargo_coverage_amount_cents: Option<i64>,
    pub liability_coverage_amount_cents: Option<i64>,
    pub currency: &'a str,
    pub insurance_effective_at: Option<NaiveDate>,
    pub insurance_expires_at: Option<NaiveDate>,
    pub verification_source: &'a str,
    pub reviewed_by_user_id: Option<i64>,
    pub notes: Option<&'a str>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DriverEquipmentSafetyComplianceRecord {
    pub id: i64,
    pub organization_id: Option<i64>,
    pub carrier_user_id: i64,
    pub driver_compliance_status: String,
    pub cdl_expires_at: Option<NaiveDate>,
    pub medical_card_expires_at: Option<NaiveDate>,
    pub mvr_status: String,
    pub background_check_status: String,
    pub endorsements: Vec<String>,
    pub equipment_compliance_status: String,
    pub truck_unit_identifier: Option<String>,
    pub trailer_unit_identifier: Option<String>,
    pub vin: Option<String>,
    pub ownership_status: Option<String>,
    pub inspection_expires_at: Option<NaiveDate>,
    pub maintenance_status: String,
    pub equipment_insurance_status: String,
    pub safety_rating: Option<String>,
    pub csa_alert_level: String,
    pub hazmat_eligible: bool,
    pub temperature_control_eligible: bool,
    pub restricted_freight_blocking: bool,
    pub dvir_policy: String,
    pub notes: Option<String>,
    pub reviewed_by_user_id: Option<i64>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UpsertDriverEquipmentSafetyParams<'a> {
    pub organization_id: Option<i64>,
    pub carrier_user_id: i64,
    pub driver_compliance_status: &'a str,
    pub cdl_expires_at: Option<NaiveDate>,
    pub medical_card_expires_at: Option<NaiveDate>,
    pub mvr_status: &'a str,
    pub background_check_status: &'a str,
    pub endorsements: Vec<String>,
    pub equipment_compliance_status: &'a str,
    pub truck_unit_identifier: Option<&'a str>,
    pub trailer_unit_identifier: Option<&'a str>,
    pub vin: Option<&'a str>,
    pub ownership_status: Option<&'a str>,
    pub inspection_expires_at: Option<NaiveDate>,
    pub maintenance_status: &'a str,
    pub equipment_insurance_status: &'a str,
    pub safety_rating: Option<&'a str>,
    pub csa_alert_level: &'a str,
    pub hazmat_eligible: bool,
    pub temperature_control_eligible: bool,
    pub restricted_freight_blocking: bool,
    pub dvir_policy: &'a str,
    pub notes: Option<&'a str>,
    pub reviewed_by_user_id: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SanctionsTaxProfileRecord {
    pub id: i64,
    pub organization_id: Option<i64>,
    pub user_id: i64,
    pub sanctions_status: String,
    pub ofac_screened_at: Option<NaiveDateTime>,
    pub sanctions_provider: String,
    pub sanctions_reference: Option<String>,
    pub beneficial_owner_status: String,
    pub tax_document_status: String,
    pub tax_document_type: Option<String>,
    pub tin_masked: Option<String>,
    pub tax_reporting_owner: String,
    pub tax_year: Option<i32>,
    pub payout_tax_blocking: bool,
    pub notes: Option<String>,
    pub reviewed_by_user_id: Option<i64>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UpsertSanctionsTaxProfileParams<'a> {
    pub organization_id: Option<i64>,
    pub user_id: i64,
    pub sanctions_status: &'a str,
    pub sanctions_provider: &'a str,
    pub sanctions_reference: Option<&'a str>,
    pub beneficial_owner_status: &'a str,
    pub tax_document_status: &'a str,
    pub tax_document_type: Option<&'a str>,
    pub tin_masked: Option<&'a str>,
    pub tax_reporting_owner: &'a str,
    pub tax_year: Option<i32>,
    pub payout_tax_blocking: bool,
    pub notes: Option<&'a str>,
    pub reviewed_by_user_id: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct RiskReviewItemRecord {
    pub id: i64,
    pub organization_id: Option<i64>,
    pub subject_user_id: Option<i64>,
    pub load_id: Option<i64>,
    pub leg_id: Option<i64>,
    pub review_type: String,
    pub severity: String,
    pub status: String,
    pub score: i32,
    pub reasons: Vec<String>,
    pub evidence: Value,
    pub hold_booking: bool,
    pub hold_payout: bool,
    pub communication_required: bool,
    pub provider_notification_required: bool,
    pub reviewer_user_id: Option<i64>,
    pub decision_note: Option<String>,
    pub decided_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateRiskReviewItemParams {
    pub organization_id: Option<i64>,
    pub subject_user_id: Option<i64>,
    pub load_id: Option<i64>,
    pub leg_id: Option<i64>,
    pub review_type: String,
    pub severity: String,
    pub score: i32,
    pub reasons: Vec<String>,
    pub evidence: Value,
    pub hold_booking: bool,
    pub hold_payout: bool,
    pub communication_required: bool,
    pub provider_notification_required: bool,
}

pub const ENTERPRISE_COMPLIANCE_DOMAINS: [&str; 7] = [
    "person_kyc",
    "company_kyb",
    "carrier_compliance",
    "broker_compliance",
    "freight_forwarder_compliance",
    "tax_compliance",
    "payout_compliance",
];

pub async fn upsert_compliance_status(
    pool: &DbPool,
    params: UpsertComplianceStatusParams<'_>,
) -> Result<ComplianceStatusRecord, sqlx::Error> {
    sqlx::query_as::<_, ComplianceStatusRecord>(
        "INSERT INTO compliance_status_records (
             organization_id, user_id, subject_type, compliance_domain, status,
             eligibility_blocking, evidence_reference, reviewer_user_id, reason,
             effective_at, expires_at, reviewed_at, created_at, updated_at
         )
         VALUES (
             $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
             CASE WHEN $8::BIGINT IS NULL THEN NULL ELSE CURRENT_TIMESTAMP END,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (
             COALESCE(organization_id, 0), COALESCE(user_id, 0), subject_type, compliance_domain
         )
         DO UPDATE SET
             status = EXCLUDED.status,
             eligibility_blocking = EXCLUDED.eligibility_blocking,
             evidence_reference = EXCLUDED.evidence_reference,
             reviewer_user_id = EXCLUDED.reviewer_user_id,
             reason = EXCLUDED.reason,
             effective_at = EXCLUDED.effective_at,
             expires_at = EXCLUDED.expires_at,
             reviewed_at = EXCLUDED.reviewed_at,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, organization_id, user_id, subject_type, compliance_domain, status,
             eligibility_blocking, evidence_reference, reviewer_user_id, reason,
             effective_at, expires_at, reviewed_at, created_at, updated_at",
    )
    .bind(params.organization_id)
    .bind(params.user_id)
    .bind(params.subject_type)
    .bind(params.compliance_domain)
    .bind(params.status)
    .bind(params.eligibility_blocking)
    .bind(params.evidence_reference)
    .bind(params.reviewer_user_id)
    .bind(params.reason)
    .bind(params.effective_at)
    .bind(params.expires_at)
    .fetch_one(pool)
    .await
}

pub async fn list_compliance_statuses_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<ComplianceStatusRecord>, sqlx::Error> {
    sqlx::query_as::<_, ComplianceStatusRecord>(
        "SELECT id, organization_id, user_id, subject_type, compliance_domain, status,
             eligibility_blocking, evidence_reference, reviewer_user_id, reason,
             effective_at, expires_at, reviewed_at, created_at, updated_at
         FROM compliance_status_records
         WHERE user_id = $1
         ORDER BY compliance_domain, subject_type",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn compliance_eligibility_summary_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<ComplianceEligibilitySummary, sqlx::Error> {
    let records = list_compliance_statuses_for_user(pool, user_id).await?;
    let approved_records = records
        .iter()
        .filter(|record| matches!(record.status.as_str(), "approved" | "not_required"))
        .count();
    let blocking_records = records
        .iter()
        .filter(|record| {
            record.eligibility_blocking
                || matches!(record.status.as_str(), "blocked" | "rejected" | "expired")
        })
        .count();
    let expired_records = records
        .iter()
        .filter(|record| {
            record.status == "expired"
                || record
                    .expires_at
                    .map(|expires_at| expires_at < chrono::Utc::now().naive_utc())
                    .unwrap_or(false)
        })
        .count();
    let missing_domains = ENTERPRISE_COMPLIANCE_DOMAINS
        .iter()
        .filter(|domain| {
            !records
                .iter()
                .any(|record| record.compliance_domain == **domain)
        })
        .map(|domain| (*domain).to_string())
        .collect::<Vec<_>>();
    let eligibility_status = if blocking_records > 0 || expired_records > 0 {
        "blocked"
    } else if missing_domains.is_empty() && approved_records == ENTERPRISE_COMPLIANCE_DOMAINS.len()
    {
        "eligible"
    } else {
        "incomplete"
    }
    .to_string();

    Ok(ComplianceEligibilitySummary {
        user_id,
        total_records: records.len(),
        approved_records,
        blocking_records,
        expired_records,
        missing_domains,
        eligibility_status,
    })
}

pub async fn upsert_carrier_authority_verification(
    pool: &DbPool,
    params: UpsertCarrierAuthorityVerificationParams<'_>,
) -> Result<CarrierAuthorityVerificationRecord, sqlx::Error> {
    sqlx::query_as::<_, CarrierAuthorityVerificationRecord>(
        "INSERT INTO carrier_authority_verifications (
             organization_id, carrier_user_id, dot_number, mc_number, legal_name,
             authority_status, operating_authority_type, safety_rating, insurance_status,
             insurance_provider, insurance_policy_number, cargo_coverage_amount_cents,
             liability_coverage_amount_cents, currency, insurance_effective_at,
             insurance_expires_at, verification_source, verified_at, reviewed_by_user_id,
             notes, created_at, updated_at
         )
         VALUES (
             $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
             $15, $16, $17, CURRENT_TIMESTAMP, $18, $19, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (carrier_user_id) DO UPDATE SET
             organization_id = EXCLUDED.organization_id,
             dot_number = EXCLUDED.dot_number,
             mc_number = EXCLUDED.mc_number,
             legal_name = EXCLUDED.legal_name,
             authority_status = EXCLUDED.authority_status,
             operating_authority_type = EXCLUDED.operating_authority_type,
             safety_rating = EXCLUDED.safety_rating,
             insurance_status = EXCLUDED.insurance_status,
             insurance_provider = EXCLUDED.insurance_provider,
             insurance_policy_number = EXCLUDED.insurance_policy_number,
             cargo_coverage_amount_cents = EXCLUDED.cargo_coverage_amount_cents,
             liability_coverage_amount_cents = EXCLUDED.liability_coverage_amount_cents,
             currency = EXCLUDED.currency,
             insurance_effective_at = EXCLUDED.insurance_effective_at,
             insurance_expires_at = EXCLUDED.insurance_expires_at,
             verification_source = EXCLUDED.verification_source,
             verified_at = EXCLUDED.verified_at,
             reviewed_by_user_id = EXCLUDED.reviewed_by_user_id,
             notes = EXCLUDED.notes,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, organization_id, carrier_user_id, dot_number, mc_number, legal_name,
             authority_status, operating_authority_type, safety_rating, insurance_status,
             insurance_provider, insurance_policy_number, cargo_coverage_amount_cents,
             liability_coverage_amount_cents, currency, insurance_effective_at,
             insurance_expires_at, verification_source, verified_at, reviewed_by_user_id,
             notes, created_at, updated_at",
    )
    .bind(params.organization_id)
    .bind(params.carrier_user_id)
    .bind(params.dot_number)
    .bind(params.mc_number)
    .bind(params.legal_name)
    .bind(params.authority_status)
    .bind(params.operating_authority_type)
    .bind(params.safety_rating)
    .bind(params.insurance_status)
    .bind(params.insurance_provider)
    .bind(params.insurance_policy_number)
    .bind(params.cargo_coverage_amount_cents)
    .bind(params.liability_coverage_amount_cents)
    .bind(params.currency)
    .bind(params.insurance_effective_at)
    .bind(params.insurance_expires_at)
    .bind(params.verification_source)
    .bind(params.reviewed_by_user_id)
    .bind(params.notes)
    .fetch_one(pool)
    .await
}

pub async fn find_carrier_authority_verification(
    pool: &DbPool,
    carrier_user_id: i64,
) -> Result<Option<CarrierAuthorityVerificationRecord>, sqlx::Error> {
    sqlx::query_as::<_, CarrierAuthorityVerificationRecord>(
        "SELECT id, organization_id, carrier_user_id, dot_number, mc_number, legal_name,
             authority_status, operating_authority_type, safety_rating, insurance_status,
             insurance_provider, insurance_policy_number, cargo_coverage_amount_cents,
             liability_coverage_amount_cents, currency, insurance_effective_at,
             insurance_expires_at, verification_source, verified_at, reviewed_by_user_id,
             notes, created_at, updated_at
         FROM carrier_authority_verifications
         WHERE carrier_user_id = $1
         LIMIT 1",
    )
    .bind(carrier_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn carrier_authority_booking_blocker(
    pool: &DbPool,
    carrier_user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
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
    .bind(carrier_user_id)
    .fetch_one(pool)
    .await
}

pub async fn upsert_driver_equipment_safety_compliance(
    pool: &DbPool,
    params: UpsertDriverEquipmentSafetyParams<'_>,
) -> Result<DriverEquipmentSafetyComplianceRecord, sqlx::Error> {
    sqlx::query_as::<_, DriverEquipmentSafetyComplianceRecord>(
        "INSERT INTO driver_equipment_safety_compliance (
             organization_id, carrier_user_id, driver_compliance_status, cdl_expires_at,
             medical_card_expires_at, mvr_status, background_check_status, endorsements,
             equipment_compliance_status, truck_unit_identifier, trailer_unit_identifier, vin,
             ownership_status, inspection_expires_at, maintenance_status, equipment_insurance_status,
             safety_rating, csa_alert_level, hazmat_eligible, temperature_control_eligible,
             restricted_freight_blocking, dvir_policy, notes, reviewed_by_user_id,
             reviewed_at, created_at, updated_at
         ) VALUES (
             $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
             $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
             CASE WHEN $24::BIGINT IS NULL THEN NULL ELSE CURRENT_TIMESTAMP END,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (carrier_user_id) DO UPDATE SET
             organization_id = EXCLUDED.organization_id,
             driver_compliance_status = EXCLUDED.driver_compliance_status,
             cdl_expires_at = EXCLUDED.cdl_expires_at,
             medical_card_expires_at = EXCLUDED.medical_card_expires_at,
             mvr_status = EXCLUDED.mvr_status,
             background_check_status = EXCLUDED.background_check_status,
             endorsements = EXCLUDED.endorsements,
             equipment_compliance_status = EXCLUDED.equipment_compliance_status,
             truck_unit_identifier = EXCLUDED.truck_unit_identifier,
             trailer_unit_identifier = EXCLUDED.trailer_unit_identifier,
             vin = EXCLUDED.vin,
             ownership_status = EXCLUDED.ownership_status,
             inspection_expires_at = EXCLUDED.inspection_expires_at,
             maintenance_status = EXCLUDED.maintenance_status,
             equipment_insurance_status = EXCLUDED.equipment_insurance_status,
             safety_rating = EXCLUDED.safety_rating,
             csa_alert_level = EXCLUDED.csa_alert_level,
             hazmat_eligible = EXCLUDED.hazmat_eligible,
             temperature_control_eligible = EXCLUDED.temperature_control_eligible,
             restricted_freight_blocking = EXCLUDED.restricted_freight_blocking,
             dvir_policy = EXCLUDED.dvir_policy,
             notes = EXCLUDED.notes,
             reviewed_by_user_id = EXCLUDED.reviewed_by_user_id,
             reviewed_at = EXCLUDED.reviewed_at,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, organization_id, carrier_user_id, driver_compliance_status,
             cdl_expires_at, medical_card_expires_at, mvr_status, background_check_status,
             endorsements, equipment_compliance_status, truck_unit_identifier,
             trailer_unit_identifier, vin, ownership_status, inspection_expires_at,
             maintenance_status, equipment_insurance_status, safety_rating, csa_alert_level,
             hazmat_eligible, temperature_control_eligible, restricted_freight_blocking,
             dvir_policy, notes, reviewed_by_user_id, reviewed_at, created_at, updated_at",
    )
    .bind(params.organization_id)
    .bind(params.carrier_user_id)
    .bind(params.driver_compliance_status)
    .bind(params.cdl_expires_at)
    .bind(params.medical_card_expires_at)
    .bind(params.mvr_status)
    .bind(params.background_check_status)
    .bind(params.endorsements)
    .bind(params.equipment_compliance_status)
    .bind(params.truck_unit_identifier)
    .bind(params.trailer_unit_identifier)
    .bind(params.vin)
    .bind(params.ownership_status)
    .bind(params.inspection_expires_at)
    .bind(params.maintenance_status)
    .bind(params.equipment_insurance_status)
    .bind(params.safety_rating)
    .bind(params.csa_alert_level)
    .bind(params.hazmat_eligible)
    .bind(params.temperature_control_eligible)
    .bind(params.restricted_freight_blocking)
    .bind(params.dvir_policy)
    .bind(params.notes)
    .bind(params.reviewed_by_user_id)
    .fetch_one(pool)
    .await
}

pub async fn driver_equipment_booking_blocker(
    pool: &DbPool,
    carrier_user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
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
    .bind(carrier_user_id)
    .fetch_one(pool)
    .await
}

pub async fn upsert_sanctions_tax_profile(
    pool: &DbPool,
    params: UpsertSanctionsTaxProfileParams<'_>,
) -> Result<SanctionsTaxProfileRecord, sqlx::Error> {
    sqlx::query_as::<_, SanctionsTaxProfileRecord>(
        "INSERT INTO sanctions_tax_profiles (
             organization_id, user_id, sanctions_status, ofac_screened_at, sanctions_provider,
             sanctions_reference, beneficial_owner_status, tax_document_status, tax_document_type,
             tin_masked, tax_reporting_owner, tax_year, payout_tax_blocking, notes,
             reviewed_by_user_id, reviewed_at, created_at, updated_at
         ) VALUES (
             $1, $2, $3, CURRENT_TIMESTAMP, $4, $5, $6, $7, $8, $9, $10, $11, $12,
             $13, $14, CASE WHEN $14::BIGINT IS NULL THEN NULL ELSE CURRENT_TIMESTAMP END,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (user_id) DO UPDATE SET
             organization_id = EXCLUDED.organization_id,
             sanctions_status = EXCLUDED.sanctions_status,
             ofac_screened_at = EXCLUDED.ofac_screened_at,
             sanctions_provider = EXCLUDED.sanctions_provider,
             sanctions_reference = EXCLUDED.sanctions_reference,
             beneficial_owner_status = EXCLUDED.beneficial_owner_status,
             tax_document_status = EXCLUDED.tax_document_status,
             tax_document_type = EXCLUDED.tax_document_type,
             tin_masked = EXCLUDED.tin_masked,
             tax_reporting_owner = EXCLUDED.tax_reporting_owner,
             tax_year = EXCLUDED.tax_year,
             payout_tax_blocking = EXCLUDED.payout_tax_blocking,
             notes = EXCLUDED.notes,
             reviewed_by_user_id = EXCLUDED.reviewed_by_user_id,
             reviewed_at = EXCLUDED.reviewed_at,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, organization_id, user_id, sanctions_status, ofac_screened_at,
             sanctions_provider, sanctions_reference, beneficial_owner_status,
             tax_document_status, tax_document_type, tin_masked, tax_reporting_owner,
             tax_year, payout_tax_blocking, notes, reviewed_by_user_id, reviewed_at,
             created_at, updated_at",
    )
    .bind(params.organization_id)
    .bind(params.user_id)
    .bind(params.sanctions_status)
    .bind(params.sanctions_provider)
    .bind(params.sanctions_reference)
    .bind(params.beneficial_owner_status)
    .bind(params.tax_document_status)
    .bind(params.tax_document_type)
    .bind(params.tin_masked)
    .bind(params.tax_reporting_owner)
    .bind(params.tax_year)
    .bind(params.payout_tax_blocking)
    .bind(params.notes)
    .bind(params.reviewed_by_user_id)
    .fetch_one(pool)
    .await
}

pub async fn sanctions_tax_booking_blocker(
    pool: &DbPool,
    user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
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
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn sanctions_tax_payout_blocker(
    pool: &DbPool,
    user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
        "WITH profile AS (
             SELECT sanctions_status, beneficial_owner_status, tax_document_status, payout_tax_blocking
             FROM sanctions_tax_profiles
             WHERE user_id = $1
             LIMIT 1
         )
         SELECT CASE
             WHEN NOT EXISTS (SELECT 1 FROM profile)
                 THEN 'Payout tax and sanctions profile is missing.'
             WHEN EXISTS (SELECT 1 FROM profile WHERE sanctions_status IN ('possible_match', 'blocked'))
                 THEN 'Payout blocked by sanctions screening.'
             WHEN EXISTS (SELECT 1 FROM profile WHERE beneficial_owner_status = 'blocked')
                 THEN 'Payout blocked by beneficial owner review.'
             WHEN EXISTS (SELECT 1 FROM profile WHERE payout_tax_blocking = TRUE AND tax_document_status NOT IN ('verified', 'not_required'))
                 THEN 'Payout blocked until tax documentation is verified or assigned not required.'
             ELSE NULL
         END",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn create_risk_review_item(
    pool: &DbPool,
    params: CreateRiskReviewItemParams,
) -> Result<RiskReviewItemRecord, sqlx::Error> {
    sqlx::query_as::<_, RiskReviewItemRecord>(
        "INSERT INTO risk_review_items (
             organization_id, subject_user_id, load_id, leg_id, review_type, severity,
             status, score, reasons, evidence, hold_booking, hold_payout,
             communication_required, provider_notification_required, created_at, updated_at
         ) VALUES (
             $1, $2, $3, $4, $5, $6, 'open', $7, $8, $9, $10, $11, $12, $13,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id, organization_id, subject_user_id, load_id, leg_id, review_type,
             severity, status, score, reasons, evidence, hold_booking, hold_payout,
             communication_required, provider_notification_required, reviewer_user_id,
             decision_note, decided_at, created_at, updated_at",
    )
    .bind(params.organization_id)
    .bind(params.subject_user_id)
    .bind(params.load_id)
    .bind(params.leg_id)
    .bind(params.review_type)
    .bind(params.severity)
    .bind(params.score)
    .bind(params.reasons)
    .bind(params.evidence)
    .bind(params.hold_booking)
    .bind(params.hold_payout)
    .bind(params.communication_required)
    .bind(params.provider_notification_required)
    .fetch_one(pool)
    .await
}

pub async fn risk_review_booking_blocker(
    pool: &DbPool,
    carrier_user_id: i64,
    leg_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
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
    .bind(carrier_user_id)
    .bind(leg_id)
    .fetch_optional(pool)
    .await
    .map(Option::flatten)
}

pub async fn risk_review_payout_blocker(
    pool: &DbPool,
    user_id: i64,
    leg_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT CONCAT('Payout blocked by ', review_type, ' review: ', array_to_string(reasons, '; '))
         FROM risk_review_items
         WHERE hold_payout = TRUE
           AND status IN ('open', 'in_review', 'blocked')
           AND (subject_user_id = $1 OR leg_id = $2)
         ORDER BY
           CASE severity WHEN 'critical' THEN 4 WHEN 'high' THEN 3 WHEN 'medium' THEN 2 ELSE 1 END DESC,
           score DESC,
           created_at DESC
         LIMIT 1",
    )
    .bind(user_id)
    .bind(leg_id)
    .fetch_optional(pool)
    .await
    .map(Option::flatten)
}
