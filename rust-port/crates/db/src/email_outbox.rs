use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailOutboxRecord {
    pub id: i64,
    pub template_name: String,
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub html_body: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error: Option<String>,
    pub next_attempt_at: NaiveDateTime,
    pub locked_at: Option<NaiveDateTime>,
    pub sent_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct EnqueueEmailParams<'a> {
    pub template_name: &'a str,
    pub to_email: &'a str,
    pub to_name: Option<&'a str>,
    pub subject: &'a str,
    pub html_body: &'a str,
    pub max_attempts: i32,
}

pub async fn enqueue_email(
    pool: &DbPool,
    params: EnqueueEmailParams<'_>,
) -> Result<EmailOutboxRecord, sqlx::Error> {
    sqlx::query_as::<_, EmailOutboxRecord>(
        r#"
        INSERT INTO email_outbox (
            template_name,
            to_email,
            to_name,
            subject,
            html_body,
            max_attempts,
            status,
            next_attempt_at,
            attempts,
            locked_at,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'processing', CURRENT_TIMESTAMP, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING *
        "#,
    )
    .bind(params.template_name)
    .bind(params.to_email)
    .bind(params.to_name)
    .bind(params.subject)
    .bind(params.html_body)
    .bind(params.max_attempts.max(1))
    .fetch_one(pool)
    .await
}

pub async fn claim_due_emails(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<EmailOutboxRecord>, sqlx::Error> {
    sqlx::query_as::<_, EmailOutboxRecord>(
        r#"
        UPDATE email_outbox
        SET status = 'processing',
            attempts = attempts + 1,
            locked_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE id IN (
            SELECT id
            FROM email_outbox
            WHERE status IN ('pending', 'retry')
              AND next_attempt_at <= CURRENT_TIMESTAMP
              AND attempts < max_attempts
            ORDER BY next_attempt_at ASC, id ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING *
        "#,
    )
    .bind(limit.max(1))
    .fetch_all(pool)
    .await
}

pub async fn mark_email_delivered(pool: &DbPool, id: i64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE email_outbox
        SET status = $2,
            last_error = NULL,
            locked_at = NULL,
            sent_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_email_retry(pool: &DbPool, id: i64, error: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE email_outbox
        SET status = CASE
                WHEN attempts >= max_attempts THEN 'failed'
                ELSE 'retry'
            END,
            last_error = $2,
            locked_at = NULL,
            next_attempt_at = CASE
                WHEN attempts >= max_attempts THEN next_attempt_at
                ELSE CURRENT_TIMESTAMP + (
                    LEAST(POWER(2, attempts), 3600)::TEXT || ' seconds'
                )::INTERVAL
            END,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(error.chars().take(2000).collect::<String>())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn reset_stale_processing_emails(
    pool: &DbPool,
    stale_after_minutes: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE email_outbox
        SET status = 'retry',
            locked_at = NULL,
            next_attempt_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE status = 'processing'
          AND locked_at < CURRENT_TIMESTAMP - (($1::TEXT || ' minutes')::INTERVAL)
        "#,
    )
    .bind(stale_after_minutes.max(1))
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn count_pending_emails(pool: &DbPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM email_outbox
        WHERE status IN ('pending', 'retry', 'processing')
        "#,
    )
    .fetch_one(pool)
    .await
}
