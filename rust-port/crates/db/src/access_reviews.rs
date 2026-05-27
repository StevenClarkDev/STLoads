use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessReviewRecord {
    pub id: i64,
    pub organization_id: i64,
    pub title: String,
    pub status: String,
    pub review_type: String,
    pub period_start: Option<NaiveDateTime>,
    pub period_end: Option<NaiveDateTime>,
    pub due_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub created_by_user_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessReviewItemRecord {
    pub id: i64,
    pub review_id: i64,
    pub organization_id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub user_email: String,
    pub role_key: String,
    pub role_label: String,
    pub account_status: i16,
    pub membership_status: Option<String>,
    pub last_activity_at: Option<NaiveDateTime>,
    pub risk_flags: Vec<String>,
    pub decision: String,
    pub decision_reason: Option<String>,
    pub decided_by_user_id: Option<i64>,
    pub decided_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateAccessReviewInput<'a> {
    pub organization_id: i64,
    pub title: &'a str,
    pub review_type: &'a str,
    pub created_by_user_id: i64,
    pub due_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct AccessReviewDecisionInput<'a> {
    pub item_id: i64,
    pub organization_id: i64,
    pub decided_by_user_id: i64,
    pub decision: &'a str,
    pub decision_reason: Option<&'a str>,
}

pub async fn list_access_reviews(
    pool: &DbPool,
    organization_id: i64,
) -> Result<Vec<AccessReviewRecord>, sqlx::Error> {
    sqlx::query_as::<_, AccessReviewRecord>(
        "SELECT id, organization_id, title, status, review_type, period_start, period_end,
            due_at, completed_at, created_by_user_id, created_at, updated_at
         FROM access_reviews
         WHERE organization_id = $1
         ORDER BY created_at DESC
         LIMIT 12",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
}

pub async fn latest_access_review_items(
    pool: &DbPool,
    organization_id: i64,
) -> Result<Vec<AccessReviewItemRecord>, sqlx::Error> {
    sqlx::query_as::<_, AccessReviewItemRecord>(
        "SELECT ari.id, ari.review_id, ari.organization_id, ari.user_id, ari.user_name,
            ari.user_email, ari.role_key, ari.role_label, ari.account_status,
            ari.membership_status, ari.last_activity_at, ari.risk_flags, ari.decision,
            ari.decision_reason, ari.decided_by_user_id, ari.decided_at, ari.revoked_at,
            ari.created_at, ari.updated_at
         FROM access_review_items ari
         INNER JOIN access_reviews ar ON ar.id = ari.review_id
         WHERE ari.organization_id = $1
           AND ar.id = (
             SELECT id FROM access_reviews
             WHERE organization_id = $1
             ORDER BY created_at DESC
             LIMIT 1
           )
         ORDER BY
           CASE ari.decision WHEN 'pending' THEN 0 ELSE 1 END,
           cardinality(ari.risk_flags) DESC,
           ari.user_email ASC",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
}

pub async fn create_access_review(
    pool: &DbPool,
    input: &CreateAccessReviewInput<'_>,
) -> Result<(AccessReviewRecord, Vec<AccessReviewItemRecord>), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let now = Utc::now().naive_utc();
    let period_start = now - Duration::days(90);

    let review = sqlx::query_as::<_, AccessReviewRecord>(
        "INSERT INTO access_reviews (
            organization_id, title, status, review_type, period_start, period_end, due_at,
            created_by_user_id, created_at, updated_at
         ) VALUES ($1, $2, 'open', $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, organization_id, title, status, review_type, period_start, period_end,
            due_at, completed_at, created_by_user_id, created_at, updated_at",
    )
    .bind(input.organization_id)
    .bind(input.title)
    .bind(input.review_type)
    .bind(period_start)
    .bind(now)
    .bind(input.due_at)
    .bind(input.created_by_user_id)
    .fetch_one(&mut *tx)
    .await?;

    let snapshot_rows = sqlx::query(
        "SELECT DISTINCT ON (u.id, COALESCE(om.role_key, CASE WHEN u.role_id = 1 THEN 'admin' ELSE 'member' END))
            u.id AS user_id,
            u.name AS user_name,
            u.email AS user_email,
            COALESCE(om.role_key, CASE WHEN u.role_id = 1 THEN 'admin' ELSE 'member' END) AS role_key,
            COALESCE(org_role.label, CASE WHEN u.role_id = 1 THEN 'Admin' ELSE 'Member' END) AS role_label,
            u.status AS account_status,
            om.status AS membership_status,
            u.updated_at AS last_activity_at,
            ARRAY_REMOVE(ARRAY[
                CASE WHEN COALESCE(org_role.privileged, FALSE) OR u.role_id = 1 THEN 'privileged_role' END,
                CASE WHEN om.id IS NULL THEN 'outside_approved_membership' END,
                CASE WHEN om.status IS DISTINCT FROM 'active' THEN 'inactive_membership' END,
                CASE WHEN u.status <> 1 THEN 'non_approved_account' END,
                CASE WHEN u.updated_at < CURRENT_TIMESTAMP - INTERVAL '90 days' THEN 'stale_account' END,
                CASE WHEN NOT EXISTS (
                    SELECT 1 FROM organization_domains od
                    WHERE od.organization_id = u.organization_id
                      AND od.verification_status = 'verified'
                      AND lower(split_part(u.email, '@', 2)) = od.domain
                ) THEN 'external_or_unverified_domain' END,
                CASE WHEN EXISTS (
                    SELECT 1 FROM admin_break_glass_sessions bg
                    WHERE bg.actor_user_id = u.id
                      AND bg.revoked_at IS NULL
                      AND bg.expires_at > CURRENT_TIMESTAMP
                ) THEN 'active_break_glass' END
            ]::TEXT[], NULL) AS risk_flags
         FROM users u
         LEFT JOIN organization_memberships om
           ON om.organization_id = u.organization_id
          AND om.user_id = u.id
         LEFT JOIN organization_roles org_role
           ON org_role.key = om.role_key
         WHERE u.organization_id = $1
           AND (
             u.role_id = 1
             OR COALESCE(org_role.privileged, FALSE)
             OR EXISTS (
                SELECT 1 FROM admin_break_glass_sessions bg
                WHERE bg.actor_user_id = u.id
                  AND bg.revoked_at IS NULL
                  AND bg.expires_at > CURRENT_TIMESTAMP
             )
           )
         ORDER BY u.id, COALESCE(om.role_key, CASE WHEN u.role_id = 1 THEN 'admin' ELSE 'member' END)",
    )
    .bind(input.organization_id)
    .fetch_all(&mut *tx)
    .await?;

    let mut items = Vec::new();
    for row in snapshot_rows {
        let item = sqlx::query_as::<_, AccessReviewItemRecord>(
            "INSERT INTO access_review_items (
                review_id, organization_id, user_id, user_name, user_email, role_key, role_label,
                account_status, membership_status, last_activity_at, risk_flags, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (review_id, user_id, role_key) DO UPDATE SET
                user_name = EXCLUDED.user_name,
                user_email = EXCLUDED.user_email,
                role_label = EXCLUDED.role_label,
                account_status = EXCLUDED.account_status,
                membership_status = EXCLUDED.membership_status,
                last_activity_at = EXCLUDED.last_activity_at,
                risk_flags = EXCLUDED.risk_flags,
                updated_at = CURRENT_TIMESTAMP
             RETURNING id, review_id, organization_id, user_id, user_name, user_email, role_key,
                role_label, account_status, membership_status, last_activity_at, risk_flags,
                decision, decision_reason, decided_by_user_id, decided_at, revoked_at,
                created_at, updated_at",
        )
        .bind(review.id)
        .bind(input.organization_id)
        .bind(row.get::<i64, _>("user_id"))
        .bind(row.get::<String, _>("user_name"))
        .bind(row.get::<String, _>("user_email"))
        .bind(row.get::<String, _>("role_key"))
        .bind(row.get::<String, _>("role_label"))
        .bind(row.get::<i16, _>("account_status"))
        .bind(row.get::<Option<String>, _>("membership_status"))
        .bind(row.get::<NaiveDateTime, _>("last_activity_at"))
        .bind(row.get::<Vec<String>, _>("risk_flags"))
        .fetch_one(&mut *tx)
        .await?;
        items.push(item);
    }

    tx.commit().await?;
    Ok((review, items))
}

pub async fn decide_access_review_item(
    pool: &DbPool,
    input: &AccessReviewDecisionInput<'_>,
) -> Result<Option<AccessReviewItemRecord>, sqlx::Error> {
    let item = sqlx::query_as::<_, AccessReviewItemRecord>(
        "UPDATE access_review_items
         SET decision = $3,
             decision_reason = $4,
             decided_by_user_id = $5,
             decided_at = CURRENT_TIMESTAMP,
             revoked_at = CASE WHEN $3 = 'revoke' THEN CURRENT_TIMESTAMP ELSE revoked_at END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
           AND organization_id = $2
         RETURNING id, review_id, organization_id, user_id, user_name, user_email, role_key,
            role_label, account_status, membership_status, last_activity_at, risk_flags,
            decision, decision_reason, decided_by_user_id, decided_at, revoked_at,
            created_at, updated_at",
    )
    .bind(input.item_id)
    .bind(input.organization_id)
    .bind(input.decision)
    .bind(input.decision_reason)
    .bind(input.decided_by_user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(item) = item.as_ref() {
        let remaining = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM access_review_items
             WHERE review_id = $1
               AND decision = 'pending'",
        )
        .bind(item.review_id)
        .fetch_one(pool)
        .await?;

        if remaining == 0 {
            sqlx::query(
                "UPDATE access_reviews
                 SET status = 'completed',
                     completed_at = COALESCE(completed_at, CURRENT_TIMESTAMP),
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1",
            )
            .bind(item.review_id)
            .execute(pool)
            .await?;
        }
    }

    Ok(item)
}
