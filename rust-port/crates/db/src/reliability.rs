use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct ObservabilitySignalRecord {
    pub signal_key: String,
    pub signal_type: String,
    pub surface: String,
    pub owner_team: String,
    pub telemetry_source: String,
    pub retention_days: i32,
    pub required_for_p0: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct AlertRuleRecord {
    pub rule_key: String,
    pub signal_key: String,
    pub severity: String,
    pub owner_team: String,
    pub route_key: String,
    pub evaluation_window_minutes: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct BackgroundJobRecord {
    pub id: i64,
    pub job_type: String,
    pub queue_name: String,
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub locked_by: Option<String>,
    pub dead_letter_reason: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct QueryPerformanceControlRecord {
    pub query_key: String,
    pub surface: String,
    pub expected_p95_ms: i32,
    pub pagination_strategy: String,
    pub required_indexes: Vec<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ReliabilityPolicyRecord {
    pub policy_key: String,
    pub owner_team: String,
    pub notes: Option<String>,
}

pub async fn list_observability_signals(
    pool: &DbPool,
) -> Result<Vec<ObservabilitySignalRecord>, sqlx::Error> {
    sqlx::query_as::<_, ObservabilitySignalRecord>(
        "SELECT signal_key, signal_type, surface, owner_team, telemetry_source,
             retention_days, required_for_p0
         FROM observability_signal_catalog
         ORDER BY surface, signal_key",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_alert_rules(pool: &DbPool) -> Result<Vec<AlertRuleRecord>, sqlx::Error> {
    sqlx::query_as::<_, AlertRuleRecord>(
        "SELECT rule_key, signal_key, severity, owner_team, route_key, evaluation_window_minutes
         FROM alert_rules
         WHERE active = TRUE
         ORDER BY severity, rule_key",
    )
    .fetch_all(pool)
    .await
}

pub async fn claim_background_jobs(
    pool: &DbPool,
    queue_name: &str,
    worker_id: &str,
    limit: i64,
) -> Result<Vec<BackgroundJobRecord>, sqlx::Error> {
    sqlx::query_as::<_, BackgroundJobRecord>(
        "WITH claimable AS (
             SELECT id
             FROM background_jobs
             WHERE queue_name = $1
               AND status IN ('queued', 'retry_scheduled')
               AND next_run_at <= CURRENT_TIMESTAMP
               AND (locked_until IS NULL OR locked_until < CURRENT_TIMESTAMP)
             ORDER BY priority ASC, next_run_at ASC, id ASC
             LIMIT $3
             FOR UPDATE SKIP LOCKED
         )
         UPDATE background_jobs job
         SET status = 'running',
             locked_by = $2,
             locked_until = CURRENT_TIMESTAMP + (visibility_timeout_seconds || ' seconds')::INTERVAL,
             attempt_count = attempt_count + 1,
             updated_at = CURRENT_TIMESTAMP
         FROM claimable
         WHERE job.id = claimable.id
         RETURNING job.id, job.job_type, job.queue_name, job.status, job.attempt_count,
             job.max_attempts, job.locked_by, job.dead_letter_reason",
    )
    .bind(queue_name)
    .bind(worker_id)
    .bind(limit.clamp(1, 500))
    .fetch_all(pool)
    .await
}

pub async fn mark_background_job_dead_letter(
    pool: &DbPool,
    job_id: i64,
    reason: &str,
) -> Result<BackgroundJobRecord, sqlx::Error> {
    sqlx::query_as::<_, BackgroundJobRecord>(
        "UPDATE background_jobs
         SET status = 'dead_letter',
             dead_letter_reason = $2,
             locked_until = NULL,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
         RETURNING id, job_type, queue_name, status, attempt_count, max_attempts, locked_by,
             dead_letter_reason",
    )
    .bind(job_id)
    .bind(reason)
    .fetch_one(pool)
    .await
}

pub async fn list_dead_letter_jobs(
    pool: &DbPool,
    queue_name: Option<&str>,
) -> Result<Vec<BackgroundJobRecord>, sqlx::Error> {
    sqlx::query_as::<_, BackgroundJobRecord>(
        "SELECT id, job_type, queue_name, status, attempt_count, max_attempts, locked_by,
             dead_letter_reason
         FROM background_jobs
         WHERE status = 'dead_letter'
           AND ($1::TEXT IS NULL OR queue_name = $1)
         ORDER BY updated_at DESC
         LIMIT 100",
    )
    .bind(queue_name)
    .fetch_all(pool)
    .await
}

pub async fn list_query_performance_controls(
    pool: &DbPool,
) -> Result<Vec<QueryPerformanceControlRecord>, sqlx::Error> {
    sqlx::query_as::<_, QueryPerformanceControlRecord>(
        "SELECT query_key, surface, expected_p95_ms, pagination_strategy, required_indexes
         FROM query_performance_controls
         ORDER BY surface, query_key",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_backup_restore_policies(
    pool: &DbPool,
) -> Result<Vec<ReliabilityPolicyRecord>, sqlx::Error> {
    sqlx::query_as::<_, ReliabilityPolicyRecord>(
        "SELECT policy_key, owner_team, notes
         FROM backup_restore_policies
         ORDER BY policy_key",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_incident_runbooks(
    pool: &DbPool,
) -> Result<Vec<ReliabilityPolicyRecord>, sqlx::Error> {
    sqlx::query_as::<_, ReliabilityPolicyRecord>(
        "SELECT runbook_key AS policy_key, owner_team, mitigation_steps AS notes
         FROM incident_runbooks
         ORDER BY runbook_key",
    )
    .fetch_all(pool)
    .await
}
