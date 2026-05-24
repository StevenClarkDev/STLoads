use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

pub const DEFAULT_ORGANIZATION_ID: i64 = 1;
pub const DEFAULT_ORGANIZATION_SLUG: &str = "stloads-default";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationRecord {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub account_type: String,
    pub status: String,
    pub billing_email: Option<String>,
    pub support_tier: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationMembershipRecord {
    pub id: i64,
    pub organization_id: i64,
    pub user_id: i64,
    pub role_key: String,
    pub status: String,
    pub invited_by_user_id: Option<i64>,
    pub joined_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn default_organization(
    pool: &DbPool,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    organization_by_slug(pool, DEFAULT_ORGANIZATION_SLUG).await
}

pub async fn organization_by_slug(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    sqlx::query_as::<_, OrganizationRecord>(
        "SELECT id, name, slug, account_type, status, billing_email, support_tier, created_at, updated_at
         FROM organizations
         WHERE slug = $1
         LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn list_memberships_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<OrganizationMembershipRecord>, sqlx::Error> {
    sqlx::query_as::<_, OrganizationMembershipRecord>(
        "SELECT id, organization_id, user_id, role_key, status, invited_by_user_id, joined_at, created_at, updated_at
         FROM organization_memberships
         WHERE user_id = $1
         ORDER BY organization_id, id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn primary_membership_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<Option<OrganizationMembershipRecord>, sqlx::Error> {
    sqlx::query_as::<_, OrganizationMembershipRecord>(
        "SELECT id, organization_id, user_id, role_key, status, invited_by_user_id, joined_at, created_at, updated_at
         FROM organization_memberships
         WHERE user_id = $1
           AND status = 'active'
         ORDER BY organization_id, id
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_permission_keys_for_organization_role(
    pool: &DbPool,
    role_key: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT permission_key
         FROM organization_role_permissions
         WHERE role_key = $1
         ORDER BY permission_key",
    )
    .bind(role_key)
    .fetch_all(pool)
    .await
}

pub async fn sync_default_membership_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<Option<OrganizationMembershipRecord>, sqlx::Error> {
    sqlx::query_as::<_, OrganizationMembershipRecord>(
        "INSERT INTO organization_memberships (organization_id, user_id, role_key, status, joined_at, created_at, updated_at)
         SELECT
            users.organization_id,
            users.id,
            CASE WHEN users.role_id = 1 THEN 'admin' ELSE 'member' END,
            'active',
            CURRENT_TIMESTAMP,
            CURRENT_TIMESTAMP,
            CURRENT_TIMESTAMP
         FROM users
         WHERE users.id = $1
         ON CONFLICT (organization_id, user_id) DO UPDATE SET
            role_key = EXCLUDED.role_key,
            status = 'active',
            updated_at = CURRENT_TIMESTAMP
         RETURNING id, organization_id, user_id, role_key, status, invited_by_user_id, joined_at, created_at, updated_at",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}
