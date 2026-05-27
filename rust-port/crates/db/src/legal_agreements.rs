use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, Row};

use crate::{
    DbPool,
    audit::{AuditEventInput, insert_audit_event},
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LegalAgreementTemplateRecord {
    pub id: i64,
    pub agreement_key: String,
    pub version: String,
    pub title: String,
    pub content_sha256: String,
    pub document_uri: Option<String>,
    pub required_role_key: Option<String>,
    pub requires_user_acceptance: bool,
    pub requires_organization_acceptance: bool,
    pub effective_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LegalAgreementAcceptanceRecord {
    pub id: i64,
    pub template_id: i64,
    pub agreement_key: String,
    pub version: String,
    pub user_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub signer_user_id: i64,
    pub signer_name: String,
    pub signer_email: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub accepted_at: NaiveDateTime,
    pub evidence_snapshot: Value,
    pub audit_event_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct AcceptLegalAgreementInput<'a> {
    pub agreement_key: &'a str,
    pub signer_user_id: i64,
    pub organization_id: i64,
    pub role_key: Option<&'a str>,
    pub signer_name: &'a str,
    pub signer_email: &'a str,
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub accept_for_organization: bool,
}

pub async fn list_required_agreements(
    pool: &DbPool,
    role_key: Option<&str>,
) -> Result<Vec<LegalAgreementTemplateRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegalAgreementTemplateRecord>(
        "SELECT id, agreement_key, version, title, content_sha256, document_uri,
                required_role_key, requires_user_acceptance, requires_organization_acceptance,
                effective_at, expires_at, is_active
         FROM legal_agreement_templates
         WHERE is_active = TRUE
           AND effective_at <= CURRENT_TIMESTAMP
           AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
           AND (required_role_key IS NULL OR required_role_key = $1)
         ORDER BY agreement_key ASC, effective_at DESC, id DESC",
    )
    .bind(role_key)
    .fetch_all(pool)
    .await
}

pub async fn list_missing_required_agreements(
    pool: &DbPool,
    user_id: i64,
    organization_id: i64,
    role_key: Option<&str>,
) -> Result<Vec<LegalAgreementTemplateRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegalAgreementTemplateRecord>(
        "SELECT t.id, t.agreement_key, t.version, t.title, t.content_sha256, t.document_uri,
                t.required_role_key, t.requires_user_acceptance, t.requires_organization_acceptance,
                t.effective_at, t.expires_at, t.is_active
         FROM legal_agreement_templates t
         WHERE t.is_active = TRUE
           AND t.effective_at <= CURRENT_TIMESTAMP
           AND (t.expires_at IS NULL OR t.expires_at > CURRENT_TIMESTAMP)
           AND (t.required_role_key IS NULL OR t.required_role_key = $3)
           AND (
                (t.requires_user_acceptance = TRUE AND NOT EXISTS (
                    SELECT 1 FROM legal_agreement_acceptances a
                    WHERE a.template_id = t.id AND a.user_id = $1
                ))
                OR
                (t.requires_organization_acceptance = TRUE AND NOT EXISTS (
                    SELECT 1 FROM legal_agreement_acceptances a
                    WHERE a.template_id = t.id AND a.organization_id = $2
                ))
           )
         ORDER BY t.agreement_key ASC, t.effective_at DESC, t.id DESC",
    )
    .bind(user_id)
    .bind(organization_id)
    .bind(role_key)
    .fetch_all(pool)
    .await
}

pub async fn list_acceptance_proofs_for_user(
    pool: &DbPool,
    user_id: i64,
    organization_id: i64,
) -> Result<Vec<LegalAgreementAcceptanceRecord>, sqlx::Error> {
    sqlx::query_as::<_, LegalAgreementAcceptanceRecord>(
        "SELECT id, template_id, agreement_key, version, user_id, organization_id,
                signer_user_id, signer_name, signer_email, ip_address, user_agent,
                accepted_at, evidence_snapshot, audit_event_id
         FROM legal_agreement_acceptances
         WHERE user_id = $1 OR organization_id = $2
         ORDER BY accepted_at DESC, id DESC",
    )
    .bind(user_id)
    .bind(organization_id)
    .fetch_all(pool)
    .await
}

pub async fn accept_latest_legal_agreement(
    pool: &DbPool,
    input: &AcceptLegalAgreementInput<'_>,
) -> Result<LegalAgreementAcceptanceRecord, sqlx::Error> {
    let template = sqlx::query_as::<_, LegalAgreementTemplateRecord>(
        "SELECT id, agreement_key, version, title, content_sha256, document_uri,
                required_role_key, requires_user_acceptance, requires_organization_acceptance,
                effective_at, expires_at, is_active
         FROM legal_agreement_templates
         WHERE agreement_key = $1
           AND is_active = TRUE
           AND effective_at <= CURRENT_TIMESTAMP
           AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
           AND (required_role_key IS NULL OR required_role_key = $2)
         ORDER BY effective_at DESC, id DESC
         LIMIT 1",
    )
    .bind(input.agreement_key)
    .bind(input.role_key)
    .fetch_one(pool)
    .await?;

    let user_id = template
        .requires_user_acceptance
        .then_some(input.signer_user_id);
    let organization_id = (template.requires_organization_acceptance
        || input.accept_for_organization)
        .then_some(input.organization_id);

    let evidence_snapshot = json!({
        "agreement_key": template.agreement_key,
        "version": template.version,
        "title": template.title,
        "content_sha256": template.content_sha256,
        "document_uri": template.document_uri,
        "required_role_key": template.required_role_key,
        "requires_user_acceptance": template.requires_user_acceptance,
        "requires_organization_acceptance": template.requires_organization_acceptance,
    });

    let acceptance = sqlx::query_as::<_, LegalAgreementAcceptanceRecord>(
        "INSERT INTO legal_agreement_acceptances (
            template_id, agreement_key, version, user_id, organization_id,
            signer_user_id, signer_name, signer_email, ip_address, user_agent,
            evidence_snapshot, created_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP)
         ON CONFLICT (template_id, user_id)
         DO UPDATE SET
            signer_user_id = EXCLUDED.signer_user_id,
            signer_name = EXCLUDED.signer_name,
            signer_email = EXCLUDED.signer_email,
            ip_address = EXCLUDED.ip_address,
            user_agent = EXCLUDED.user_agent,
            accepted_at = CURRENT_TIMESTAMP,
            evidence_snapshot = EXCLUDED.evidence_snapshot
         RETURNING id, template_id, agreement_key, version, user_id, organization_id,
                   signer_user_id, signer_name, signer_email, ip_address, user_agent,
                   accepted_at, evidence_snapshot, audit_event_id",
    )
    .bind(template.id)
    .bind(&template.agreement_key)
    .bind(&template.version)
    .bind(user_id)
    .bind(organization_id)
    .bind(input.signer_user_id)
    .bind(input.signer_name)
    .bind(input.signer_email)
    .bind(input.ip_address)
    .bind(input.user_agent)
    .bind(&evidence_snapshot)
    .fetch_one(pool)
    .await?;

    if organization_id.is_some() {
        sqlx::query(
            "INSERT INTO legal_agreement_acceptances (
                template_id, agreement_key, version, user_id, organization_id,
                signer_user_id, signer_name, signer_email, ip_address, user_agent,
                evidence_snapshot, created_at
             )
             VALUES ($1, $2, $3, NULL, $4, $5, $6, $7, $8, $9, $10, CURRENT_TIMESTAMP)
             ON CONFLICT (template_id, organization_id)
             DO UPDATE SET
                signer_user_id = EXCLUDED.signer_user_id,
                signer_name = EXCLUDED.signer_name,
                signer_email = EXCLUDED.signer_email,
                ip_address = EXCLUDED.ip_address,
                user_agent = EXCLUDED.user_agent,
                accepted_at = CURRENT_TIMESTAMP,
                evidence_snapshot = EXCLUDED.evidence_snapshot",
        )
        .bind(template.id)
        .bind(&template.agreement_key)
        .bind(&template.version)
        .bind(input.organization_id)
        .bind(input.signer_user_id)
        .bind(input.signer_name)
        .bind(input.signer_email)
        .bind(input.ip_address)
        .bind(input.user_agent)
        .bind(&evidence_snapshot)
        .execute(pool)
        .await?;
    }

    let audit_event_id = insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(input.signer_user_id),
            organization_id: Some(input.organization_id),
            target_organization_id: Some(input.organization_id),
            entity_type: "legal_agreement",
            entity_id: Some(&acceptance.agreement_key),
            action: "legal_agreement_accepted",
            reason: Some("User accepted required legal agreement"),
            ticket_ref: None,
            request_id: input.request_id,
            ip_address: input.ip_address,
            user_agent: input.user_agent,
            source: "rust-backend",
            metadata: Some(json!({
                "template_id": template.id,
                "agreement_key": acceptance.agreement_key,
                "version": acceptance.version,
                "acceptance_id": acceptance.id,
                "organization_acceptance": organization_id.is_some(),
            })),
            before_state: None,
            after_state: Some(evidence_snapshot),
        },
    )
    .await?;

    sqlx::query("UPDATE legal_agreement_acceptances SET audit_event_id = $1 WHERE id = $2")
        .bind(audit_event_id)
        .bind(acceptance.id)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, LegalAgreementAcceptanceRecord>(
        "SELECT id, template_id, agreement_key, version, user_id, organization_id,
                signer_user_id, signer_name, signer_email, ip_address, user_agent,
                accepted_at, evidence_snapshot, audit_event_id
         FROM legal_agreement_acceptances
         WHERE id = $1",
    )
    .bind(acceptance.id)
    .fetch_one(pool)
    .await
}

pub async fn legal_acceptance_summary(
    pool: &DbPool,
    user_id: i64,
    organization_id: i64,
    role_key: Option<&str>,
) -> Result<
    (
        Vec<LegalAgreementTemplateRecord>,
        Vec<LegalAgreementAcceptanceRecord>,
    ),
    sqlx::Error,
> {
    let missing =
        list_missing_required_agreements(pool, user_id, organization_id, role_key).await?;
    let proofs = list_acceptance_proofs_for_user(pool, user_id, organization_id).await?;
    Ok((missing, proofs))
}

pub async fn acceptance_has_audit_event(
    pool: &DbPool,
    acceptance_id: i64,
) -> Result<bool, sqlx::Error> {
    let value = sqlx::query(
        "SELECT audit_event_id
         FROM legal_agreement_acceptances
         WHERE id = $1",
    )
    .bind(acceptance_id)
    .fetch_optional(pool)
    .await?;
    Ok(value
        .and_then(|row| row.get::<Option<i64>, _>("audit_event_id"))
        .is_some())
}
