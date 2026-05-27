use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BreakGlassSessionRecord {
    pub id: String,
    pub actor_user_id: i64,
    pub actor_organization_id: Option<i64>,
    pub target_organization_id: i64,
    pub reason: String,
    pub ticket_ref: String,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateBreakGlassSessionInput<'a> {
    pub id: &'a str,
    pub actor_user_id: i64,
    pub actor_organization_id: Option<i64>,
    pub target_organization_id: i64,
    pub reason: &'a str,
    pub ticket_ref: &'a str,
    pub request_id: Option<&'a str>,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct AuditEventInput<'a> {
    pub actor_user_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub target_organization_id: Option<i64>,
    pub entity_type: &'a str,
    pub entity_id: Option<&'a str>,
    pub action: &'a str,
    pub reason: Option<&'a str>,
    pub ticket_ref: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub source: &'a str,
    pub metadata: Option<Value>,
    pub before_state: Option<Value>,
    pub after_state: Option<Value>,
}

pub async fn create_break_glass_session(
    pool: &DbPool,
    input: &CreateBreakGlassSessionInput<'_>,
) -> Result<BreakGlassSessionRecord, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let session = sqlx::query_as::<_, BreakGlassSessionRecord>(
        "INSERT INTO admin_break_glass_sessions (
            id, actor_user_id, actor_organization_id, target_organization_id, reason, ticket_ref,
            expires_at, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, actor_user_id, actor_organization_id, target_organization_id, reason,
             ticket_ref, expires_at, revoked_at, created_at, updated_at",
    )
    .bind(input.id)
    .bind(input.actor_user_id)
    .bind(input.actor_organization_id)
    .bind(input.target_organization_id)
    .bind(input.reason)
    .bind(input.ticket_ref)
    .bind(input.expires_at)
    .fetch_one(&mut *tx)
    .await?;

    insert_audit_event_in_tx(
        &mut tx,
        &AuditEventInput {
            actor_user_id: Some(input.actor_user_id),
            organization_id: input.actor_organization_id,
            target_organization_id: Some(input.target_organization_id),
            entity_type: "admin_break_glass_session",
            entity_id: Some(input.id),
            action: "break_glass_started",
            reason: Some(input.reason),
            ticket_ref: Some(input.ticket_ref),
            request_id: input.request_id,
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "expires_at": input.expires_at,
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(session)
}

pub async fn has_active_break_glass_session(
    pool: &DbPool,
    actor_user_id: i64,
    target_organization_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM admin_break_glass_sessions
            WHERE actor_user_id = $1
              AND target_organization_id = $2
              AND revoked_at IS NULL
              AND expires_at > $3
        )",
    )
    .bind(actor_user_id)
    .bind(target_organization_id)
    .bind(Utc::now().naive_utc())
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

pub async fn insert_audit_event(
    pool: &DbPool,
    input: &AuditEventInput<'_>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO audit_events (
            actor_user_id, organization_id, target_organization_id, entity_type, entity_id,
            action, reason, ticket_ref, request_id, ip_address, user_agent, source, metadata,
            before_state, after_state, created_at
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(input.actor_user_id)
    .bind(input.organization_id)
    .bind(input.target_organization_id)
    .bind(input.entity_type)
    .bind(input.entity_id)
    .bind(input.action)
    .bind(input.reason)
    .bind(input.ticket_ref)
    .bind(input.request_id)
    .bind(input.ip_address)
    .bind(input.user_agent)
    .bind(input.source)
    .bind(input.metadata.as_ref())
    .bind(input.before_state.as_ref())
    .bind(input.after_state.as_ref())
    .fetch_one(pool)
    .await
}

async fn insert_audit_event_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    input: &AuditEventInput<'_>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO audit_events (
            actor_user_id, organization_id, target_organization_id, entity_type, entity_id,
            action, reason, ticket_ref, request_id, ip_address, user_agent, source, metadata,
            before_state, after_state, created_at
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(input.actor_user_id)
    .bind(input.organization_id)
    .bind(input.target_organization_id)
    .bind(input.entity_type)
    .bind(input.entity_id)
    .bind(input.action)
    .bind(input.reason)
    .bind(input.ticket_ref)
    .bind(input.request_id)
    .bind(input.ip_address)
    .bind(input.user_agent)
    .bind(input.source)
    .bind(input.metadata.as_ref())
    .bind(input.before_state.as_ref())
    .bind(input.after_state.as_ref())
    .fetch_one(&mut **tx)
    .await
}
