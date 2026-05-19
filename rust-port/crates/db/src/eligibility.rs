use std::{error::Error, fmt};

use chrono::NaiveDateTime;
use domain::eligibility::{EligibilityBlock, EligibilityDecision};
use serde_json::json;
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug)]
pub enum EligibilityGateError {
    Database(sqlx::Error),
    Rejected(EligibilityDecision),
}

impl fmt::Display for EligibilityGateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "eligibility database error: {error}"),
            Self::Rejected(decision) => write!(
                formatter,
                "carrier is not eligible: {}",
                decision.result_detail
            ),
        }
    }
}

impl Error for EligibilityGateError {}

impl From<sqlx::Error> for EligibilityGateError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

#[derive(Debug, Clone, FromRow)]
struct CarrierEligibilityProfile {
    tenant_id: String,
    carrier_profile_id: i64,
    status: String,
    compliance_status: String,
    insurance_status: String,
    authority_status: String,
    dot_number: Option<String>,
    mc_number: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct PostingEligibilityProfile {
    posting_id: i64,
    equipment_type: Option<String>,
    visibility: String,
}

#[derive(Debug, Clone, FromRow)]
struct PostingStops {
    origin_state: Option<String>,
    destination_state: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ComplianceOverrideRecord {
    pub id: i64,
    pub tenant_id: String,
    pub carrier_profile_id: i64,
    pub posting_id: Option<i64>,
    pub override_key: String,
    pub reason: String,
    pub expires_at: Option<NaiveDateTime>,
    pub approved_by: i64,
    pub created_at: NaiveDateTime,
}

pub async fn evaluate_carrier_eligibility(
    pool: &DbPool,
    tenant_id: &str,
    posting_id: i64,
    carrier_profile_id: i64,
) -> Result<EligibilityDecision, sqlx::Error> {
    let carrier = sqlx::query_as::<_, CarrierEligibilityProfile>(
        r#"
        SELECT tenant_id,
               id AS carrier_profile_id,
               status,
               compliance_status,
               insurance_status,
               authority_status,
               dot_number,
               mc_number
        FROM carrier_profiles
        WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .fetch_one(pool)
    .await?;

    let posting = sqlx::query_as::<_, PostingEligibilityProfile>(
        r#"
        SELECT id AS posting_id,
               equipment_type,
               visibility
        FROM stloads_postings
        WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(tenant_id)
    .bind(posting_id)
    .fetch_one(pool)
    .await?;

    let stops = posting_stops(pool, tenant_id, posting_id).await?;
    let mut blocks = Vec::new();
    let mut warnings = Vec::new();

    if carrier.status.trim().to_ascii_lowercase() != "active" {
        blocks.push(block(
            "carrier_profile_inactive",
            "Carrier inactive",
            "Carrier profile is not active.",
        ));
    }

    if carrier.compliance_status.trim().to_ascii_lowercase() != "approved" {
        blocks.push(block(
            "carrier_compliance_not_approved",
            "Compliance not approved",
            "Carrier compliance status is not approved.",
        ));
    }

    if !has_current_document(pool, tenant_id, carrier_profile_id, "carrier_packet").await? {
        blocks.push(block(
            "carrier_packet_incomplete",
            "Packet incomplete",
            "Carrier packet is missing, unapproved, or expired.",
        ));
    }

    if !has_current_document(pool, tenant_id, carrier_profile_id, "w9").await? {
        blocks.push(block(
            "w9_missing",
            "W-9 missing",
            "Approved W-9 is missing.",
        ));
    }

    if carrier.insurance_status.trim().to_ascii_lowercase() != "approved" {
        blocks.push(block(
            "insurance_not_approved",
            "Insurance not approved",
            "Carrier insurance status is not approved.",
        ));
    } else if !has_current_document(pool, tenant_id, carrier_profile_id, "insurance_certificate")
        .await?
    {
        blocks.push(block(
            "insurance_expired",
            "Insurance expired",
            "Insurance certificate is missing, unapproved, or expired.",
        ));
    }

    if !matches!(
        carrier
            .authority_status
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "approved" | "verified" | "active"
    ) {
        blocks.push(block(
            "authority_not_verified",
            "Authority not verified",
            "Carrier authority status is not verified.",
        ));
    }

    if carrier
        .dot_number
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && carrier
            .mc_number
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        blocks.push(block(
            "authority_identifier_missing",
            "DOT/MC missing",
            "Carrier must have DOT or MC authority identifiers on file.",
        ));
    }

    if let Some(equipment_type) = posting
        .equipment_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !has_equipment(pool, tenant_id, carrier_profile_id, equipment_type).await? {
            blocks.push(block(
                "equipment_not_eligible",
                "Equipment not eligible",
                "Carrier does not have active equipment for this posting.",
            ));
        }
    }

    if !has_lane(pool, tenant_id, carrier_profile_id, &stops).await? {
        blocks.push(block(
            "lane_not_eligible",
            "Lane not eligible",
            "Carrier does not have an active lane matching this posting.",
        ));
    }

    if has_blocking_visibility_rule(pool, tenant_id, posting_id, carrier_profile_id).await? {
        blocks.push(block(
            "carrier_blocked",
            "Carrier blocked",
            "Carrier is blocked for this customer or posting.",
        ));
    }

    if posting.visibility.trim().eq_ignore_ascii_case("private")
        && !has_allowing_visibility_rule(pool, tenant_id, posting_id, carrier_profile_id).await?
    {
        blocks.push(block(
            "customer_approval_required",
            "Customer approval required",
            "Private posting requires explicit customer approval for this carrier.",
        ));
    }

    for flag in open_blocking_risk_flags(pool, tenant_id, carrier_profile_id).await? {
        match flag.as_str() {
            "fraud" => blocks.push(block(
                "fraud_risk",
                "Fraud risk",
                "Open fraud risk flag blocks booking.",
            )),
            "double_broker" => blocks.push(block(
                "double_broker_risk",
                "Double broker risk",
                "Open double-broker risk flag blocks booking.",
            )),
            other => warnings.push(block(
                "carrier_risk_watch",
                "Carrier risk watch",
                &format!("Open carrier risk flag: {other}."),
            )),
        }
    }

    let mut active_blocks = Vec::new();
    let mut overridden_blocks = Vec::new();
    for item in blocks {
        if has_active_override(pool, tenant_id, carrier_profile_id, posting_id, &item.key).await? {
            overridden_blocks.push(item);
        } else {
            active_blocks.push(item);
        }
    }

    let eligible = active_blocks.is_empty();
    let result_code = if eligible { "eligible" } else { "blocked" }.to_string();
    let result_detail = if eligible {
        if overridden_blocks.is_empty() {
            "Carrier is eligible for view, offer, tender acceptance, and book-now.".into()
        } else {
            format!(
                "Carrier is eligible with {} active override(s).",
                overridden_blocks.len()
            )
        }
    } else {
        active_blocks
            .iter()
            .map(|block| block.label.clone())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let evaluated_at = chrono::Utc::now().naive_utc();

    let decision = EligibilityDecision {
        tenant_id: carrier.tenant_id.clone(),
        posting_id: posting.posting_id,
        carrier_profile_id: carrier.carrier_profile_id,
        eligible,
        result_code,
        result_detail,
        blocks: active_blocks,
        warnings,
        overridden_blocks,
        evaluated_at,
    };

    persist_eligibility_result(pool, &decision).await?;
    Ok(decision)
}

pub async fn find_carrier_profile_id_for_user(
    pool: &DbPool,
    tenant_id: &str,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM carrier_profiles
        WHERE tenant_id = $1 AND user_id = $2 AND deleted_at IS NULL
        ORDER BY id DESC
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn ensure_carrier_can_book_posting(
    pool: &DbPool,
    tenant_id: &str,
    posting_id: i64,
    carrier_profile_id: i64,
) -> Result<EligibilityDecision, EligibilityGateError> {
    let decision =
        evaluate_carrier_eligibility(pool, tenant_id, posting_id, carrier_profile_id).await?;
    if decision.eligible {
        Ok(decision)
    } else {
        Err(EligibilityGateError::Rejected(decision))
    }
}

pub async fn create_compliance_override(
    pool: &DbPool,
    tenant_id: &str,
    carrier_profile_id: i64,
    posting_id: Option<i64>,
    override_key: &str,
    reason: &str,
    expires_at: Option<NaiveDateTime>,
    approved_by: i64,
) -> Result<ComplianceOverrideRecord, sqlx::Error> {
    sqlx::query_as::<_, ComplianceOverrideRecord>(
        r#"
        INSERT INTO compliance_overrides
            (tenant_id, carrier_profile_id, posting_id, override_key, reason,
             expires_at, approved_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, tenant_id, carrier_profile_id, posting_id, override_key, reason,
                  expires_at, approved_by, created_at
        "#,
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .bind(posting_id)
    .bind(override_key.trim())
    .bind(reason.trim())
    .bind(expires_at)
    .bind(approved_by)
    .fetch_one(pool)
    .await
}

async fn persist_eligibility_result(
    pool: &DbPool,
    decision: &EligibilityDecision,
) -> Result<(), sqlx::Error> {
    let evaluated_rules = json!({
        "blocks": decision.blocks,
        "warnings": decision.warnings,
        "overridden_blocks": decision.overridden_blocks,
    });

    sqlx::query(
        r#"
        INSERT INTO eligibility_results
            (tenant_id, posting_id, carrier_profile_id, eligible, result_code,
             result_detail, evaluated_rules, evaluated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP)
        ON CONFLICT (tenant_id, posting_id, carrier_profile_id)
        DO UPDATE SET
            eligible = EXCLUDED.eligible,
            result_code = EXCLUDED.result_code,
            result_detail = EXCLUDED.result_detail,
            evaluated_rules = EXCLUDED.evaluated_rules,
            evaluated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&decision.tenant_id)
    .bind(decision.posting_id)
    .bind(decision.carrier_profile_id)
    .bind(decision.eligible)
    .bind(&decision.result_code)
    .bind(&decision.result_detail)
    .bind(evaluated_rules)
    .execute(pool)
    .await?;
    Ok(())
}

async fn posting_stops(
    pool: &DbPool,
    tenant_id: &str,
    posting_id: i64,
) -> Result<PostingStops, sqlx::Error> {
    sqlx::query_as::<_, PostingStops>(
        r#"
        SELECT
            MAX(CASE WHEN stop_type = 'pickup' THEN state_region END) AS origin_state,
            MAX(CASE WHEN stop_type = 'delivery' THEN state_region END) AS destination_state
        FROM stloads_posting_stops
        WHERE tenant_id = $1 AND posting_id = $2
        "#,
    )
    .bind(tenant_id)
    .bind(posting_id)
    .fetch_one(pool)
    .await
}

async fn has_current_document(
    pool: &DbPool,
    tenant_id: &str,
    carrier_profile_id: i64,
    document_type: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM compliance_documents
            WHERE tenant_id = $1
              AND carrier_profile_id = $2
              AND document_type = $3
              AND status = 'approved'
              AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
        )
        "#,
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .bind(document_type)
    .fetch_one(pool)
    .await
}

async fn has_equipment(
    pool: &DbPool,
    tenant_id: &str,
    carrier_profile_id: i64,
    equipment_type: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM carrier_equipment
            WHERE tenant_id = $1
              AND carrier_profile_id = $2
              AND LOWER(equipment_type) = LOWER($3)
              AND status = 'active'
              AND quantity > 0
        )
        "#,
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .bind(equipment_type)
    .fetch_one(pool)
    .await
}

async fn has_lane(
    pool: &DbPool,
    tenant_id: &str,
    carrier_profile_id: i64,
    stops: &PostingStops,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM carrier_lanes
            WHERE tenant_id = $1
              AND carrier_profile_id = $2
              AND status = 'active'
              AND (origin_state IS NULL OR LOWER(origin_state) = LOWER(COALESCE($3, origin_state)))
              AND (destination_state IS NULL OR LOWER(destination_state) = LOWER(COALESCE($4, destination_state)))
        )
        "#,
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .bind(stops.origin_state.as_deref())
    .bind(stops.destination_state.as_deref())
    .fetch_one(pool)
    .await
}

async fn has_blocking_visibility_rule(
    pool: &DbPool,
    tenant_id: &str,
    posting_id: i64,
    carrier_profile_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM stloads_posting_visibility_rules
            WHERE tenant_id = $1
              AND posting_id = $2
              AND target_type = 'carrier_profile'
              AND target_id = $3
              AND allow = FALSE
              AND (starts_at IS NULL OR starts_at <= CURRENT_TIMESTAMP)
              AND (ends_at IS NULL OR ends_at > CURRENT_TIMESTAMP)
        )
        "#,
    )
    .bind(tenant_id)
    .bind(posting_id)
    .bind(carrier_profile_id.to_string())
    .fetch_one(pool)
    .await
}

async fn has_allowing_visibility_rule(
    pool: &DbPool,
    tenant_id: &str,
    posting_id: i64,
    carrier_profile_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM stloads_posting_visibility_rules
            WHERE tenant_id = $1
              AND posting_id = $2
              AND target_type = 'carrier_profile'
              AND target_id = $3
              AND allow = TRUE
              AND (starts_at IS NULL OR starts_at <= CURRENT_TIMESTAMP)
              AND (ends_at IS NULL OR ends_at > CURRENT_TIMESTAMP)
        )
        "#,
    )
    .bind(tenant_id)
    .bind(posting_id)
    .bind(carrier_profile_id.to_string())
    .fetch_one(pool)
    .await
}

async fn open_blocking_risk_flags(
    pool: &DbPool,
    tenant_id: &str,
    carrier_profile_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT flag_type
        FROM carrier_risk_flags
        WHERE tenant_id = $1
          AND carrier_profile_id = $2
          AND status = 'open'
          AND severity IN ('blocker', 'critical')
        ORDER BY created_at DESC
        "#,
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .fetch_all(pool)
    .await
}

async fn has_active_override(
    pool: &DbPool,
    tenant_id: &str,
    carrier_profile_id: i64,
    posting_id: i64,
    override_key: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM compliance_overrides
            WHERE tenant_id = $1
              AND carrier_profile_id = $2
              AND override_key = $3
              AND (posting_id IS NULL OR posting_id = $4)
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
        )
        "#,
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .bind(override_key)
    .bind(posting_id)
    .fetch_one(pool)
    .await
}

fn block(key: &str, label: &str, detail: &str) -> EligibilityBlock {
    EligibilityBlock::new(key, label, detail)
}
