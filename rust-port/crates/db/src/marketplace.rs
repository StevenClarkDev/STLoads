use chrono::{Duration, NaiveDateTime, Utc};
use domain::auth::UserRole;
use domain::marketplace::{OfferStatus, marketplace_module_contract, offer_status_descriptors};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfferStatusMasterRecord {
    pub id: i16,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfferRecord {
    pub id: i64,
    pub load_leg_id: i64,
    pub carrier_id: i64,
    pub conversation_id: Option<i64>,
    pub amount: f64,
    pub status_id: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl OfferRecord {
    pub fn status(&self) -> Option<OfferStatus> {
        OfferStatus::from_legacy_code(self.status_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationRecord {
    pub id: i64,
    pub load_leg_id: i64,
    pub shipper_id: i64,
    pub carrier_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageRecord {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub body: Option<String>,
    pub meta: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationWorkspaceRecord {
    pub id: i64,
    pub load_leg_id: i64,
    pub load_leg_code: Option<String>,
    pub shipper_id: i64,
    pub shipper_name: String,
    pub carrier_id: i64,
    pub carrier_name: String,
    pub last_message_body: Option<String>,
    pub last_message_user_id: Option<i64>,
    pub last_activity_at: NaiveDateTime,
    pub message_count: i64,
    pub offer_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageDetailRecord {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub author_name: String,
    pub body: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationReadRecord {
    pub conversation_id: i64,
    pub user_id: i64,
    pub last_read_message_id: Option<i64>,
    pub last_read_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationPresenceRecord {
    pub conversation_id: i64,
    pub user_id: i64,
    pub state: String,
    pub last_seen_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn list_offers_for_leg(
    pool: &DbPool,
    load_leg_id: i64,
) -> Result<Vec<OfferRecord>, sqlx::Error> {
    sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount, status_id, created_at, updated_at
         FROM offers
         WHERE load_leg_id = $1
         ORDER BY id DESC",
    )
    .bind(load_leg_id)
    .fetch_all(pool)
    .await
}

pub async fn list_pending_offers_for_leg(
    pool: &DbPool,
    load_leg_id: i64,
) -> Result<Vec<OfferRecord>, sqlx::Error> {
    sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount, status_id, created_at, updated_at
         FROM offers
         WHERE load_leg_id = $1 AND status_id = 1
         ORDER BY id DESC",
    )
    .bind(load_leg_id)
    .fetch_all(pool)
    .await
}

pub async fn list_conversations_for_leg(
    pool: &DbPool,
    load_leg_id: i64,
) -> Result<Vec<ConversationRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE load_leg_id = $1
         ORDER BY id DESC",
    )
    .bind(load_leg_id)
    .fetch_all(pool)
    .await
}

pub async fn find_conversation_for_leg_and_carrier(
    pool: &DbPool,
    load_leg_id: i64,
    carrier_id: i64,
) -> Result<Option<ConversationRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE load_leg_id = $1 AND carrier_id = $2
         LIMIT 1",
    )
    .bind(load_leg_id)
    .bind(carrier_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_messages_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Vec<MessageRecord>, sqlx::Error> {
    sqlx::query_as::<_, MessageRecord>(
        "SELECT id, conversation_id, user_id, body, meta, created_at, updated_at
         FROM messages
         WHERE conversation_id = $1
         ORDER BY id ASC",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_conversation_workspace_records(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<ConversationWorkspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        ORDER BY last_activity_at DESC, c.id DESC\n        LIMIT $1",
        conversation_workspace_select_sql()
    ))
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_conversation_workspace_records_for_user(
    pool: &DbPool,
    viewer_user_id: i64,
    viewer_role: Option<UserRole>,
    limit: i64,
) -> Result<Vec<ConversationWorkspaceRecord>, sqlx::Error> {
    if viewer_role == Some(UserRole::Admin) {
        return list_recent_conversation_workspace_records(pool, limit).await;
    }

    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        WHERE c.shipper_id = $1 OR c.carrier_id = $2\n        ORDER BY last_activity_at DESC, c.id DESC\n        LIMIT $3",
        conversation_workspace_select_sql()
    ))
    .bind(viewer_user_id)
    .bind(viewer_user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn find_conversation_workspace_record(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Option<ConversationWorkspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        WHERE c.id = $1\n        LIMIT 1",
        conversation_workspace_select_sql()
    ))
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_conversation_workspace_record_for_user(
    pool: &DbPool,
    conversation_id: i64,
    viewer_user_id: i64,
    viewer_role: Option<UserRole>,
) -> Result<Option<ConversationWorkspaceRecord>, sqlx::Error> {
    if viewer_role == Some(UserRole::Admin) {
        return find_conversation_workspace_record(pool, conversation_id).await;
    }

    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        WHERE c.id = $1 AND (c.shipper_id = $2 OR c.carrier_id = $3)\n        LIMIT 1",
        conversation_workspace_select_sql()
    ))
    .bind(conversation_id)
    .bind(viewer_user_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_message_details_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
    limit: i64,
) -> Result<Vec<MessageDetailRecord>, sqlx::Error> {
    sqlx::query_as::<_, MessageDetailRecord>(
        r#"
        SELECT
            m.id,
            m.conversation_id,
            m.user_id,
            u.name AS author_name,
            m.body,
            m.created_at
        FROM messages m
        INNER JOIN users u ON u.id = m.user_id
        WHERE m.conversation_id = $1
        ORDER BY m.id DESC
        LIMIT $2
        "#,
    )
    .bind(conversation_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn find_offer_by_id(
    pool: &DbPool,
    offer_id: i64,
) -> Result<Option<OfferRecord>, sqlx::Error> {
    sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount, status_id, created_at, updated_at
         FROM offers
         WHERE id = $1
         LIMIT 1",
    )
    .bind(offer_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_conversation_by_id(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Option<ConversationRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE id = $1
         LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
}

pub async fn latest_message_id_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM messages
         WHERE conversation_id = $1
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn mark_conversation_read(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<Option<ConversationReadRecord>, sqlx::Error> {
    let last_read_message_id = latest_message_id_for_conversation(pool, conversation_id).await?;

    sqlx::query(
        "INSERT INTO conversation_reads (conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (conversation_id, user_id) DO UPDATE SET
             last_read_message_id = EXCLUDED.last_read_message_id,
             last_read_at = EXCLUDED.last_read_at,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(last_read_message_id)
    .execute(pool)
    .await?;

    find_conversation_read_state(pool, conversation_id, user_id).await
}

pub async fn find_conversation_read_state(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<Option<ConversationReadRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationReadRecord>(
        "SELECT conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at
         FROM conversation_reads
         WHERE conversation_id = $1 AND user_id = $2
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_peer_conversation_read_state(
    pool: &DbPool,
    conversation_id: i64,
    viewer_user_id: i64,
) -> Result<Option<ConversationReadRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationReadRecord>(
        "SELECT conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at
         FROM conversation_reads
         WHERE conversation_id = $1 AND user_id <> $2
         ORDER BY last_read_at DESC
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn count_unread_messages_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let unread = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM messages m
        WHERE m.conversation_id = $1
          AND m.user_id <> $2
          AND m.id > COALESCE(
                (
                    SELECT cr.last_read_message_id
                    FROM conversation_reads cr
                    WHERE cr.conversation_id = $3 AND cr.user_id = $4
                    LIMIT 1
                ),
                0
          )
        "#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(unread.max(0) as u64)
}

pub async fn upsert_conversation_presence(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
    state: &str,
) -> Result<Option<ConversationPresenceRecord>, sqlx::Error> {
    sqlx::query(
        "INSERT INTO conversation_presence (conversation_id, user_id, state, last_seen_at, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (conversation_id, user_id) DO UPDATE SET
             state = EXCLUDED.state,
             last_seen_at = EXCLUDED.last_seen_at,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(state)
    .execute(pool)
    .await?;

    find_conversation_presence_state(pool, conversation_id, user_id).await
}

pub async fn find_conversation_presence_state(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<Option<ConversationPresenceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationPresenceRecord>(
        "SELECT conversation_id, user_id, state, last_seen_at, created_at, updated_at
         FROM conversation_presence
         WHERE conversation_id = $1 AND user_id = $2
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_conversation_presence(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM conversation_presence
         WHERE conversation_id = $1 AND user_id = $2",
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn find_active_peer_presence(
    pool: &DbPool,
    conversation_id: i64,
    viewer_user_id: i64,
    max_age_seconds: i64,
) -> Result<Option<ConversationPresenceRecord>, sqlx::Error> {
    let threshold = Utc::now().naive_utc() - Duration::seconds(max_age_seconds.max(5));

    sqlx::query_as::<_, ConversationPresenceRecord>(
        "SELECT conversation_id, user_id, state, last_seen_at, created_at, updated_at
         FROM conversation_presence
         WHERE conversation_id = $1 AND user_id <> $2 AND last_seen_at >= $3
         ORDER BY last_seen_at DESC
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(viewer_user_id)
    .bind(threshold)
    .fetch_optional(pool)
    .await
}

pub async fn review_offer(
    pool: &DbPool,
    offer_id: i64,
    accept: bool,
    actor_user_id: Option<i64>,
) -> Result<Option<OfferRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(offer) = sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount, status_id, created_at, updated_at
         FROM offers
         WHERE id = $1
         LIMIT 1",
    )
    .bind(offer_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let final_status_id = if accept { 3 } else { 0 };

    sqlx::query(
        "UPDATE offers
         SET status_id = $1, updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(final_status_id)
    .bind(offer_id)
    .execute(&mut *tx)
    .await?;

    if accept {
        sqlx::query(
            "UPDATE offers
             SET status_id = 0, updated_at = CURRENT_TIMESTAMP
             WHERE load_leg_id = $1 AND id <> $2 AND status_id = 1",
        )
        .bind(offer.load_leg_id)
        .bind(offer_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE load_legs
             SET accepted_offer_id = $1,
                 booked_carrier_id = $2,
                 booked_amount = $3,
                 booked_at = COALESCE(booked_at, CURRENT_TIMESTAMP),
                 status_id = CASE WHEN status_id < 4 THEN 4 ELSE status_id END,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $4",
        )
        .bind(offer_id)
        .bind(offer.carrier_id)
        .bind(offer.amount)
        .bind(offer.load_leg_id)
        .execute(&mut *tx)
        .await?;
    }

    let load_history_status = if accept { 4 } else { 3 };
    let load_history_note = if accept {
        "Rust marketplace accepted an offer"
    } else {
        "Rust marketplace declined an offer"
    };

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         SELECT load_id, $1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         FROM load_legs
         WHERE id = $4",
    )
    .bind(actor_user_id)
    .bind(load_history_status)
    .bind(load_history_note)
    .bind(offer.load_leg_id)
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount, status_id, created_at, updated_at
         FROM offers
         WHERE id = $1
         LIMIT 1",
    )
    .bind(offer_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}

pub async fn create_message(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
    body: &str,
) -> Result<Option<MessageDetailRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(_conversation) = sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE id = $1
         LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let message_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO messages (conversation_id, user_id, body, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(body)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE conversations
         SET updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(conversation_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO conversation_reads (conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (conversation_id, user_id) DO UPDATE SET
             last_read_message_id = EXCLUDED.last_read_message_id,
             last_read_at = EXCLUDED.last_read_at,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(message_id)
    .execute(&mut *tx)
    .await?;

    let message = sqlx::query_as::<_, MessageDetailRecord>(
        r#"
        SELECT
            m.id,
            m.conversation_id,
            m.user_id,
            u.name AS author_name,
            m.body,
            m.created_at
        FROM messages m
        INNER JOIN users u ON u.id = m.user_id
        WHERE m.id = $1
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(message)
}

pub async fn marketplace_contract_summary() -> domain::marketplace::MarketplaceModuleContract {
    marketplace_module_contract()
}

pub async fn offer_status_catalog() -> &'static [domain::marketplace::OfferStatusDescriptor] {
    offer_status_descriptors()
}

fn conversation_workspace_select_sql() -> &'static str {
    r#"
        SELECT
            c.id,
            c.load_leg_id,
            ll.leg_code AS load_leg_code,
            c.shipper_id,
            shipper.name AS shipper_name,
            c.carrier_id,
            carrier.name AS carrier_name,
            (
                SELECT m.body
                FROM messages m
                WHERE m.conversation_id = c.id
                ORDER BY m.id DESC
                LIMIT 1
            ) AS last_message_body,
            (
                SELECT m.user_id
                FROM messages m
                WHERE m.conversation_id = c.id
                ORDER BY m.id DESC
                LIMIT 1
            ) AS last_message_user_id,
            COALESCE(
                (
                    SELECT m.created_at
                    FROM messages m
                    WHERE m.conversation_id = c.id
                    ORDER BY m.id DESC
                    LIMIT 1
                ),
                c.updated_at
            ) AS last_activity_at,
            (
                SELECT COUNT(*)
                FROM messages m
                WHERE m.conversation_id = c.id
            ) AS message_count,
            (
                SELECT COUNT(*)
                FROM offers o
                WHERE o.conversation_id = c.id OR (o.conversation_id IS NULL AND o.load_leg_id = c.load_leg_id)
            ) AS offer_count
        FROM conversations c
        INNER JOIN load_legs ll ON ll.id = c.load_leg_id AND ll.deleted_at IS NULL
        INNER JOIN users shipper ON shipper.id = c.shipper_id
        INNER JOIN users carrier ON carrier.id = c.carrier_id
    "#
}
