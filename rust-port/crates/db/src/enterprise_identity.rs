use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnterpriseSsoDiscoveryRecord {
    pub provider_id: Option<i64>,
    pub organization_id: i64,
    pub organization_name: String,
    pub domain: String,
    pub domain_verified: bool,
    pub login_routing_enabled: bool,
    pub provider_type: Option<String>,
    pub provider_status: Option<String>,
    pub provider_display_name: Option<String>,
    pub sso_url: Option<String>,
    pub issuer: Option<String>,
    pub jwks_url: Option<String>,
    pub client_id: Option<String>,
    pub jit_enabled: bool,
    pub default_role_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimTokenRecord {
    pub id: i64,
    pub organization_id: i64,
    pub label: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimDeprovisionInput {
    pub organization_id: i64,
    pub external_id: Option<String>,
    pub email: Option<String>,
    pub user_id: Option<i64>,
    pub reason: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimDeprovisionOutcome {
    pub user_id: Option<i64>,
    pub organization_id: i64,
    pub membership_rows_changed: u64,
    pub user_rows_changed: u64,
    pub link_rows_changed: u64,
    pub event_id: i64,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUpsertUserInput {
    pub organization_id: i64,
    pub external_id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role_id: i16,
    pub role_key: String,
    pub active: bool,
    pub reason: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUpsertUserOutcome {
    pub user_id: i64,
    pub organization_id: i64,
    pub created: bool,
    pub reactivated: bool,
    pub event_id: i64,
}

pub async fn discover_sso_for_email(
    pool: &DbPool,
    email: &str,
) -> Result<Option<EnterpriseSsoDiscoveryRecord>, sqlx::Error> {
    let Some(domain) = email
        .split('@')
        .nth(1)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        return Ok(None);
    };

    sqlx::query_as::<_, EnterpriseSsoDiscoveryRecord>(
        "SELECT
            od.organization_id,
            organizations.name AS organization_name,
            od.domain,
            od.verification_status = 'verified' AS domain_verified,
            od.login_routing_enabled,
            idp.id AS provider_id,
            idp.provider_type,
            idp.status AS provider_status,
            idp.display_name AS provider_display_name,
            idp.sso_url,
            idp.issuer,
            idp.jwks_url,
            idp.client_id,
            COALESCE(idp.jit_enabled, FALSE) AS jit_enabled,
            idp.default_role_key
         FROM organization_domains od
         INNER JOIN organizations ON organizations.id = od.organization_id
         LEFT JOIN LATERAL (
            SELECT id, provider_type, status, display_name, sso_url, issuer, jwks_url, client_id, jit_enabled, default_role_key
            FROM enterprise_identity_providers
            WHERE organization_id = od.organization_id
              AND status = 'active'
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
         ) idp ON TRUE
         WHERE LOWER(od.domain) = LOWER($1)
         LIMIT 1",
    )
    .bind(domain)
    .fetch_optional(pool)
    .await
}

pub async fn find_scim_token_by_hash(
    pool: &DbPool,
    token_prefix: &str,
    token_hash: &str,
) -> Result<Option<ScimTokenRecord>, sqlx::Error> {
    let record = sqlx::query_as::<_, ScimTokenRecord>(
        "UPDATE scim_tokens
         SET last_used_at = CURRENT_TIMESTAMP
         WHERE token_prefix = $1
           AND token_hash = $2
           AND status = 'active'
           AND revoked_at IS NULL
         RETURNING id, organization_id, label, created_at",
    )
    .bind(token_prefix)
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

pub async fn deprovision_scim_user(
    pool: &DbPool,
    input: &ScimDeprovisionInput,
) -> Result<ScimDeprovisionOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let user_id = if let Some(user_id) = input.user_id {
        sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE id = $1 AND organization_id = $2")
            .bind(user_id)
            .bind(input.organization_id)
            .fetch_optional(&mut *tx)
            .await?
    } else if let Some(external_id) = input.external_id.as_deref() {
        sqlx::query_scalar::<_, i64>(
            "SELECT user_id
             FROM scim_user_links
             WHERE organization_id = $1
               AND external_id = $2",
        )
        .bind(input.organization_id)
        .bind(external_id)
        .fetch_optional(&mut *tx)
        .await?
    } else if let Some(email) = input.email.as_deref() {
        sqlx::query_scalar::<_, i64>(
            "SELECT id
             FROM users
             WHERE organization_id = $1
               AND LOWER(email) = LOWER($2)",
        )
        .bind(input.organization_id)
        .bind(email)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        None
    };

    let mut membership_rows_changed = 0;
    let mut user_rows_changed = 0;
    let mut link_rows_changed = 0;
    let outcome = if let Some(user_id) = user_id {
        membership_rows_changed = sqlx::query(
            "UPDATE organization_memberships
             SET status = 'deprovisioned',
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1
               AND user_id = $2
               AND status <> 'deprovisioned'",
        )
        .bind(input.organization_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        user_rows_changed = sqlx::query(
            "UPDATE users
             SET status = 2,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
               AND organization_id = $2
               AND status <> 2",
        )
        .bind(user_id)
        .bind(input.organization_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if let Some(external_id) = input.external_id.as_deref() {
            link_rows_changed = sqlx::query(
                "INSERT INTO scim_user_links (
                    organization_id, user_id, external_id, active, deprovisioned_at, created_at, updated_at
                 )
                 VALUES ($1, $2, $3, FALSE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 ON CONFLICT (organization_id, external_id) DO UPDATE SET
                    user_id = EXCLUDED.user_id,
                    active = FALSE,
                    deprovisioned_at = CURRENT_TIMESTAMP,
                    updated_at = CURRENT_TIMESTAMP",
            )
            .bind(input.organization_id)
            .bind(user_id)
            .bind(external_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        }

        ("accepted", Some(user_id))
    } else {
        ("noop", None)
    };

    let event_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO scim_events (
            organization_id, external_id, user_id, action, outcome, reason, payload, created_at
         )
         VALUES ($1, $2, $3, 'deprovision', $4, $5, $6, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(input.organization_id)
    .bind(input.external_id.as_deref())
    .bind(outcome.1)
    .bind(outcome.0)
    .bind(input.reason.as_deref())
    .bind(&input.payload)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(ScimDeprovisionOutcome {
        user_id: outcome.1,
        organization_id: input.organization_id,
        membership_rows_changed,
        user_rows_changed,
        link_rows_changed,
        event_id,
        outcome: outcome.0.into(),
    })
}

pub async fn upsert_scim_user(
    pool: &DbPool,
    input: &ScimUpsertUserInput,
) -> Result<ScimUpsertUserOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let existing = sqlx::query_as::<_, (i64, bool)>(
        "SELECT users.id, scim_user_links.active
         FROM scim_user_links
         INNER JOIN users ON users.id = scim_user_links.user_id
         WHERE scim_user_links.organization_id = $1
           AND scim_user_links.external_id = $2
         UNION
         SELECT users.id, TRUE
         FROM users
         WHERE users.organization_id = $1
           AND LOWER(users.email) = LOWER($3)
         LIMIT 1",
    )
    .bind(input.organization_id)
    .bind(&input.external_id)
    .bind(&input.email)
    .fetch_optional(&mut *tx)
    .await?;

    let status = if input.active { 1_i16 } else { 2_i16 };
    let (user_id, created, reactivated) = if let Some((user_id, was_active)) = existing {
        sqlx::query(
            "UPDATE users
             SET name = $2,
                 email = $3,
                 role_id = $4,
                 status = $5,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
               AND organization_id = $6",
        )
        .bind(user_id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(input.role_id)
        .bind(status)
        .bind(input.organization_id)
        .execute(&mut *tx)
        .await?;
        (user_id, false, input.active && !was_active)
    } else {
        let user_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO users (
                organization_id, name, email, password, role_id, email_verified_at, status, created_at, updated_at
             )
             VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .bind(input.organization_id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(&input.password_hash)
        .bind(input.role_id)
        .bind(status)
        .fetch_one(&mut *tx)
        .await?;
        (user_id, true, false)
    };

    sqlx::query(
        "DELETE FROM model_has_roles
         WHERE model_type = 'App\\Models\\User'
           AND model_id = $1",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO model_has_roles (role_id, model_type, model_id)
         VALUES ($1, 'App\\Models\\User', $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(i64::from(input.role_id))
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    let membership_status = if input.active {
        "active"
    } else {
        "deprovisioned"
    };
    sqlx::query(
        "INSERT INTO organization_memberships (
            organization_id, user_id, role_key, status, joined_at, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (organization_id, user_id) DO UPDATE SET
            role_key = EXCLUDED.role_key,
            status = EXCLUDED.status,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(input.organization_id)
    .bind(user_id)
    .bind(&input.role_key)
    .bind(membership_status)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO scim_user_links (
            organization_id, user_id, external_id, active, deprovisioned_at, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, CASE WHEN $4 THEN NULL ELSE CURRENT_TIMESTAMP END, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (organization_id, external_id) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            active = EXCLUDED.active,
            deprovisioned_at = EXCLUDED.deprovisioned_at,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(input.organization_id)
    .bind(user_id)
    .bind(&input.external_id)
    .bind(input.active)
    .execute(&mut *tx)
    .await?;

    let action = if created {
        "provision"
    } else if reactivated {
        "reactivate"
    } else {
        "update"
    };
    let event_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO scim_events (
            organization_id, external_id, user_id, action, outcome, reason, payload, created_at
         )
         VALUES ($1, $2, $3, $4, 'accepted', $5, $6, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(input.organization_id)
    .bind(&input.external_id)
    .bind(user_id)
    .bind(action)
    .bind(input.reason.as_deref())
    .bind(&input.payload)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(ScimUpsertUserOutcome {
        user_id,
        organization_id: input.organization_id,
        created,
        reactivated,
        event_id,
    })
}
