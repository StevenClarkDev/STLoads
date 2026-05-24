use std::env;

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let database_schema = env::var("STLOADS_DATABASE_SCHEMA")
        .or_else(|_| env::var("DATABASE_SCHEMA"))
        .ok()
        .and_then(optional_env_value);

    let pool = db::connect_with_schema(&database_url, database_schema.as_deref())
        .await
        .context("connect to the migration database")?;

    db::migrate(&pool)
        .await
        .context("apply Rust database migrations")?;

    println!("database migrations completed");
    Ok(())
}

fn optional_env_value(value: String) -> Option<String> {
    let trimmed = value.trim().trim_matches('"').to_string();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        None
    } else {
        Some(trimmed)
    }
}
