use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
    postgres::PgPoolOptions,
};

pub static MIGRATOR: Migrator = sqlx::migrate!();

pub type DbPool = PgPool;

pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn connect_with_schema(
    database_url: &str,
    schema: Option<&str>,
) -> Result<DbPool, sqlx::Error> {
    let Some(schema) = schema.and_then(normalize_schema_name) else {
        return connect(database_url).await;
    };

    PgPoolOptions::new()
        .max_connections(10)
        .after_connect(move |connection, _meta| {
            let schema = schema.clone();
            Box::pin(async move {
                let create_schema = format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#);
                let set_search_path = format!(r#"SET search_path TO "{schema}", public"#);
                sqlx::query(&create_schema).execute(&mut *connection).await?;
                sqlx::query(&set_search_path).execute(&mut *connection).await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &DbPool) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await
}

fn normalize_schema_name(schema: &str) -> Option<String> {
    let trimmed = schema.trim();
    let valid = !trimmed.is_empty()
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');

    valid.then(|| trimmed.to_string())
}
