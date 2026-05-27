use std::{collections::BTreeMap, env, fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, Default)]
struct Args {
    database_url: Option<String>,
    expected_json_path: Option<PathBuf>,
    output_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReconciliationResult {
    status: String,
    summary: BTreeMap<String, Value>,
    comparisons: Vec<Comparison>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Comparison {
    section: String,
    status: String,
    actual: Value,
    expected: Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args()?;
    let database_url = args
        .database_url
        .or_else(|| env::var("DATABASE_URL").ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("--database-url or DATABASE_URL is required"))?;

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("connect to Rust PostgreSQL database")?;

    let mut summary = BTreeMap::new();
    summary.insert(
        "generated_at_utc".into(),
        json!(chrono::Utc::now().to_rfc3339()),
    );
    summary.insert("source".into(), json!("rust-postgres"));
    summary.insert(
        "table_counts".into(),
        query_json(&pool, TABLE_COUNTS).await?,
    );
    summary.insert(
        "users_by_role_status".into(),
        query_json(&pool, USERS_BY_ROLE_STATUS).await?,
    );
    summary.insert(
        "loads_by_status".into(),
        query_json(&pool, LOADS_BY_STATUS).await?,
    );
    summary.insert(
        "legs_by_status".into(),
        query_json(&pool, LEGS_BY_STATUS).await?,
    );
    summary.insert(
        "documents_by_provider".into(),
        query_json(&pool, DOCUMENTS_BY_PROVIDER).await?,
    );
    summary.insert(
        "payments_by_status".into(),
        query_json(&pool, PAYMENTS_BY_STATUS).await?,
    );
    summary.insert(
        "tms_by_status".into(),
        query_json(&pool, TMS_BY_STATUS).await?,
    );

    let mut result = ReconciliationResult {
        status: "generated".into(),
        summary,
        comparisons: Vec::new(),
    };

    if let Some(expected_json_path) = args.expected_json_path {
        let expected_text = fs::read_to_string(&expected_json_path)
            .with_context(|| format!("read expected summary {}", expected_json_path.display()))?;
        let expected: Value =
            serde_json::from_str(&expected_text).context("parse expected summary JSON")?;
        for section in [
            "table_counts",
            "users_by_role_status",
            "loads_by_status",
            "legs_by_status",
            "documents_by_provider",
            "payments_by_status",
            "tms_by_status",
        ] {
            let actual = result.summary.get(section).cloned().unwrap_or(Value::Null);
            let expected_value = expected
                .get("summary")
                .and_then(|summary| summary.get(section))
                .cloned()
                .unwrap_or(Value::Null);
            result.comparisons.push(Comparison {
                section: section.into(),
                status: if actual == expected_value {
                    "match".into()
                } else {
                    "mismatch".into()
                },
                actual,
                expected: expected_value,
            });
        }
        result.status = if result
            .comparisons
            .iter()
            .all(|comparison| comparison.status == "match")
        {
            "match".into()
        } else {
            "mismatch".into()
        };
    }

    if let Some(parent) = args.output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output directory {}", parent.display()))?;
        }
    }
    fs::write(&args.output_path, serde_json::to_string_pretty(&result)?)
        .with_context(|| format!("write {}", args.output_path.display()))?;

    println!(
        "Cutover reconciliation {}: {}",
        result.status,
        args.output_path.display()
    );
    if result.status == "mismatch" {
        std::process::exit(1);
    }
    Ok(())
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        output_path: PathBuf::from("runtime/cutover-reconciliation.json"),
        ..Args::default()
    };
    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--database-url" => args.database_url = iter.next(),
            "--expected-json-path" => args.expected_json_path = iter.next().map(PathBuf::from),
            "--output" => {
                args.output_path = iter
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow!("--output requires a path"))?
            }
            "--help" | "-h" => {
                println!(
                    "Usage: cargo run -p backend --bin cutover_reconciliation -- --database-url <url> [--expected-json-path <path>] [--output <path>]"
                );
                std::process::exit(0);
            }
            other => return Err(anyhow!("unknown argument: {other}")),
        }
    }
    Ok(args)
}

async fn query_json(pool: &PgPool, query: &str) -> Result<Value> {
    let value = sqlx::query_scalar::<_, Value>(query)
        .fetch_one(pool)
        .await
        .with_context(|| format!("run reconciliation query: {query}"))?;
    Ok(value)
}

const TABLE_COUNTS: &str = r#"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.domain, rows.table_name), '[]'::jsonb)
FROM (
  SELECT 'identity' AS domain, 'users' AS table_name, COUNT(*)::bigint AS count FROM users
  UNION ALL SELECT 'identity', 'user_details', COUNT(*)::bigint FROM user_details
  UNION ALL SELECT 'identity', 'user_history', COUNT(*)::bigint FROM user_history
  UNION ALL SELECT 'loads', 'loads', COUNT(*)::bigint FROM loads
  UNION ALL SELECT 'loads', 'load_legs', COUNT(*)::bigint FROM load_legs
  UNION ALL SELECT 'loads', 'offers', COUNT(*)::bigint FROM offers
  UNION ALL SELECT 'marketplace', 'conversations', COUNT(*)::bigint FROM conversations
  UNION ALL SELECT 'marketplace', 'messages', COUNT(*)::bigint FROM messages
  UNION ALL SELECT 'documents', 'load_documents', COUNT(*)::bigint FROM load_documents
  UNION ALL SELECT 'documents', 'leg_documents', COUNT(*)::bigint FROM leg_documents
  UNION ALL SELECT 'documents', 'kyc_documents', COUNT(*)::bigint FROM kyc_documents
  UNION ALL SELECT 'payments', 'escrows', COUNT(*)::bigint FROM escrows
  UNION ALL SELECT 'tms', 'stloads_handoffs', COUNT(*)::bigint FROM stloads_handoffs
  UNION ALL SELECT 'tms', 'stloads_handoff_events', COUNT(*)::bigint FROM stloads_handoff_events
  UNION ALL SELECT 'tms', 'stloads_sync_errors', COUNT(*)::bigint FROM stloads_sync_errors
  UNION ALL SELECT 'tms', 'stloads_reconciliation_log', COUNT(*)::bigint FROM stloads_reconciliation_log
  UNION ALL SELECT 'master_data', 'countries', COUNT(*)::bigint FROM countries
  UNION ALL SELECT 'master_data', 'cities', COUNT(*)::bigint FROM cities
  UNION ALL SELECT 'master_data', 'locations', COUNT(*)::bigint FROM locations
  UNION ALL SELECT 'master_data', 'load_types', COUNT(*)::bigint FROM load_types
  UNION ALL SELECT 'master_data', 'equipments', COUNT(*)::bigint FROM equipments
  UNION ALL SELECT 'master_data', 'commodity_types', COUNT(*)::bigint FROM commodity_types
) rows;
"#;

const USERS_BY_ROLE_STATUS: &str = r#"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.role_id, rows.status), '[]'::jsonb)
FROM (
  SELECT role_id, status, COUNT(*)::bigint AS count
  FROM users
  GROUP BY role_id, status
) rows;
"#;

const LOADS_BY_STATUS: &str = r#"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status), '[]'::jsonb)
FROM (
  SELECT status, COUNT(*)::bigint AS count
  FROM loads
  WHERE deleted_at IS NULL
  GROUP BY status
) rows;
"#;

const LEGS_BY_STATUS: &str = r#"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status_id), '[]'::jsonb)
FROM (
  SELECT status_id, COUNT(*)::bigint AS count, COALESCE(SUM(booked_amount), 0)::double precision AS booked_amount_total
  FROM load_legs
  WHERE deleted_at IS NULL
  GROUP BY status_id
) rows;
"#;

const DOCUMENTS_BY_PROVIDER: &str = r#"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.document_scope, rows.storage_provider), '[]'::jsonb)
FROM (
  SELECT 'load' AS document_scope, COALESCE(NULLIF(storage_provider, ''), 'unknown') AS storage_provider, COUNT(*)::bigint AS count
  FROM load_documents
  GROUP BY COALESCE(NULLIF(storage_provider, ''), 'unknown')
  UNION ALL
  SELECT
    'leg',
    COALESCE(
      NULLIF(meta->>'storage_provider', ''),
      CASE
        WHEN path LIKE 'cos://%' THEN 'ibm_cos'
        WHEN path LIKE 's3://%' THEN 's3'
        ELSE 'local'
      END
    ),
    COUNT(*)::bigint
  FROM leg_documents
  GROUP BY COALESCE(
    NULLIF(meta->>'storage_provider', ''),
    CASE
      WHEN path LIKE 'cos://%' THEN 'ibm_cos'
      WHEN path LIKE 's3://%' THEN 's3'
      ELSE 'local'
    END
  )
  UNION ALL
  SELECT 'kyc', 'profile_kyc', COUNT(*)::bigint
  FROM kyc_documents
) rows;
"#;

const PAYMENTS_BY_STATUS: &str = r#"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status, rows.currency), '[]'::jsonb)
FROM (
  SELECT status, currency, COUNT(*)::bigint AS count, COALESCE(SUM(amount), 0)::double precision AS amount_total
  FROM escrows
  GROUP BY status, currency
) rows;
"#;

const TMS_BY_STATUS: &str = r#"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status, rows.tenant_id), '[]'::jsonb)
FROM (
  SELECT status, tenant_id, COUNT(*)::bigint AS count, COUNT(load_id)::bigint AS materialized_loads
  FROM stloads_handoffs
  GROUP BY status, tenant_id
) rows;
"#;
