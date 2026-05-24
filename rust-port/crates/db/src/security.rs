use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThrottleDecision {
    pub allowed: bool,
    pub retry_after_seconds: i64,
}

#[derive(Debug, FromRow)]
struct ThrottleRow {
    allowed: bool,
    retry_after_seconds: i64,
}

pub async fn check_rate_limit(
    pool: &DbPool,
    key: &str,
    max_attempts: i32,
    window_seconds: i64,
) -> Result<ThrottleDecision, sqlx::Error> {
    let row = sqlx::query_as::<_, ThrottleRow>(
        "WITH upserted AS (
            INSERT INTO security_rate_limits
                (key, counter, window_started_at, expires_at, created_at, updated_at)
            VALUES (
                $1,
                1,
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP + ($3 * INTERVAL '1 second'),
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP
            )
            ON CONFLICT (key) DO UPDATE
            SET counter = CASE
                    WHEN security_rate_limits.window_started_at <= CURRENT_TIMESTAMP - ($3 * INTERVAL '1 second')
                    THEN 1
                    ELSE security_rate_limits.counter + 1
                END,
                window_started_at = CASE
                    WHEN security_rate_limits.window_started_at <= CURRENT_TIMESTAMP - ($3 * INTERVAL '1 second')
                    THEN CURRENT_TIMESTAMP
                    ELSE security_rate_limits.window_started_at
                END,
                expires_at = CURRENT_TIMESTAMP + ($3 * INTERVAL '1 second'),
                updated_at = CURRENT_TIMESTAMP
            RETURNING counter, window_started_at
        )
        SELECT
            counter <= $2 AS allowed,
            GREATEST(
                1,
                CEIL(EXTRACT(EPOCH FROM ((window_started_at + ($3 * INTERVAL '1 second')) - CURRENT_TIMESTAMP)))::BIGINT
            ) AS retry_after_seconds
        FROM upserted",
    )
    .bind(key)
    .bind(max_attempts)
    .bind(window_seconds)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn lockout_status(pool: &DbPool, key: &str) -> Result<ThrottleDecision, sqlx::Error> {
    let row = sqlx::query_as::<_, ThrottleRow>(
        "SELECT
            CASE WHEN locked_until IS NULL OR locked_until <= CURRENT_TIMESTAMP THEN TRUE ELSE FALSE END AS allowed,
            CASE
                WHEN locked_until IS NULL OR locked_until <= CURRENT_TIMESTAMP THEN 0
                ELSE GREATEST(1, CEIL(EXTRACT(EPOCH FROM (locked_until - CURRENT_TIMESTAMP)))::BIGINT)
            END AS retry_after_seconds
         FROM security_rate_limits
         WHERE key = $1
         LIMIT 1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into).unwrap_or(ThrottleDecision {
        allowed: true,
        retry_after_seconds: 0,
    }))
}

pub async fn record_lockout_failure(
    pool: &DbPool,
    key: &str,
    max_failures: i32,
    window_seconds: i64,
    lockout_seconds: i64,
) -> Result<ThrottleDecision, sqlx::Error> {
    let row = sqlx::query_as::<_, ThrottleRow>(
        "WITH upserted AS (
            INSERT INTO security_rate_limits
                (key, counter, window_started_at, locked_until, expires_at, created_at, updated_at)
            VALUES (
                $1,
                1,
                CURRENT_TIMESTAMP,
                NULL,
                CURRENT_TIMESTAMP + ($3 * INTERVAL '1 second'),
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP
            )
            ON CONFLICT (key) DO UPDATE
            SET counter = CASE
                    WHEN security_rate_limits.window_started_at <= CURRENT_TIMESTAMP - ($3 * INTERVAL '1 second')
                    THEN 1
                    ELSE security_rate_limits.counter + 1
                END,
                window_started_at = CASE
                    WHEN security_rate_limits.window_started_at <= CURRENT_TIMESTAMP - ($3 * INTERVAL '1 second')
                    THEN CURRENT_TIMESTAMP
                    ELSE security_rate_limits.window_started_at
                END,
                locked_until = CASE
                    WHEN (
                        CASE
                            WHEN security_rate_limits.window_started_at <= CURRENT_TIMESTAMP - ($3 * INTERVAL '1 second')
                            THEN 1
                            ELSE security_rate_limits.counter + 1
                        END
                    ) >= $2
                    THEN CURRENT_TIMESTAMP + ($4 * INTERVAL '1 second')
                    ELSE security_rate_limits.locked_until
                END,
                expires_at = CURRENT_TIMESTAMP + (($3 + $4) * INTERVAL '1 second'),
                updated_at = CURRENT_TIMESTAMP
            RETURNING counter, locked_until
        )
        SELECT
            CASE WHEN locked_until IS NULL OR locked_until <= CURRENT_TIMESTAMP THEN TRUE ELSE FALSE END AS allowed,
            CASE
                WHEN locked_until IS NULL OR locked_until <= CURRENT_TIMESTAMP THEN 0
                ELSE GREATEST(1, CEIL(EXTRACT(EPOCH FROM (locked_until - CURRENT_TIMESTAMP)))::BIGINT)
            END AS retry_after_seconds
        FROM upserted",
    )
    .bind(key)
    .bind(max_failures)
    .bind(window_seconds)
    .bind(lockout_seconds)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn clear_lockout(pool: &DbPool, key: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM security_rate_limits WHERE key = $1")
        .bind(key)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

impl From<ThrottleRow> for ThrottleDecision {
    fn from(row: ThrottleRow) -> Self {
        Self {
            allowed: row.allowed,
            retry_after_seconds: row.retry_after_seconds.max(0),
        }
    }
}
