use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OperatingAuthorityDecisionRecord {
    pub id: i64,
    pub model_key: String,
    pub label: String,
    pub scope: String,
    pub decision: String,
    pub regulatory_note: String,
    pub owner: String,
    pub approved_by: Option<String>,
    pub approved_at: Option<chrono::NaiveDateTime>,
    pub next_review_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceEvidenceRecord {
    pub id: i64,
    pub evidence_key: String,
    pub label: String,
    pub owner: String,
    pub status: String,
    pub evidence_type: String,
    pub issuer: Option<String>,
    pub policy_or_authority_number: Option<String>,
    pub coverage_amount_cents: Option<i64>,
    pub currency: Option<String>,
    pub jurisdiction: Option<String>,
    pub document_uri: Option<String>,
    pub customer_disclosable: bool,
    pub renewal_required: bool,
    pub effective_at: Option<NaiveDate>,
    pub expires_at: Option<NaiveDate>,
    pub review_due_at: Option<NaiveDate>,
    pub notes: Option<String>,
}

pub async fn list_operating_authority_decisions(
    pool: &DbPool,
) -> Result<Vec<OperatingAuthorityDecisionRecord>, sqlx::Error> {
    sqlx::query_as::<_, OperatingAuthorityDecisionRecord>(
        "SELECT id, model_key, label, scope, decision, regulatory_note, owner,
                approved_by, approved_at, next_review_at
         FROM operating_authority_decisions
         ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_customer_disclosable_evidence(
    pool: &DbPool,
) -> Result<Vec<ComplianceEvidenceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ComplianceEvidenceRecord>(
        "SELECT id, evidence_key, label, owner, status, evidence_type, issuer,
                policy_or_authority_number, coverage_amount_cents, currency, jurisdiction,
                document_uri, customer_disclosable, renewal_required, effective_at,
                expires_at, review_due_at, notes
         FROM compliance_evidence_records
         WHERE customer_disclosable = TRUE
         ORDER BY evidence_key ASC, id ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_evidence_due_for_review(
    pool: &DbPool,
    within_days: i64,
) -> Result<Vec<ComplianceEvidenceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ComplianceEvidenceRecord>(
        "SELECT id, evidence_key, label, owner, status, evidence_type, issuer,
                policy_or_authority_number, coverage_amount_cents, currency, jurisdiction,
                document_uri, customer_disclosable, renewal_required, effective_at,
                expires_at, review_due_at, notes
         FROM compliance_evidence_records
         WHERE renewal_required = TRUE
           AND (
                (expires_at IS NOT NULL AND expires_at <= CURRENT_DATE + ($1::text || ' days')::interval)
                OR
                (review_due_at IS NOT NULL AND review_due_at <= CURRENT_DATE + ($1::text || ' days')::interval)
           )
         ORDER BY COALESCE(expires_at, review_due_at) ASC, id ASC",
    )
    .bind(within_days)
    .fetch_all(pool)
    .await
}

pub async fn upsert_compliance_evidence_document(
    pool: &DbPool,
    evidence_key: &str,
    document_uri: &str,
    issuer: Option<&str>,
    policy_or_authority_number: Option<&str>,
    effective_at: Option<NaiveDate>,
    expires_at: Option<NaiveDate>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE compliance_evidence_records
         SET document_uri = $2,
             issuer = $3,
             policy_or_authority_number = $4,
             effective_at = $5,
             expires_at = $6,
             updated_at = CURRENT_TIMESTAMP
         WHERE evidence_key = $1",
    )
    .bind(evidence_key)
    .bind(document_uri)
    .bind(issuer)
    .bind(policy_or_authority_number)
    .bind(effective_at)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}
