use std::env;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub bind_addr: String,
    pub port: u16,
    pub deployment_target: String,
    pub environment: String,
    pub public_base_url: Option<String>,
    pub cors_allowed_origins: Vec<String>,
    pub run_migrations: bool,
    pub database_url: Option<String>,
    pub document_storage_backend: String,
    pub document_storage_root: String,
    pub object_storage_bucket: Option<String>,
    pub object_storage_region: String,
    pub object_storage_endpoint: Option<String>,
    pub object_storage_access_key_id: Option<String>,
    pub object_storage_secret_access_key: Option<String>,
    pub object_storage_session_token: Option<String>,
    pub object_storage_force_path_style: bool,
    pub object_storage_prefix: String,
    pub stripe_webhook_shared_secret: Option<String>,
    pub stripe_webhook_connect_secret: Option<String>,
    pub stripe_secret_key: Option<String>,
    pub stripe_api_base_url: String,
    pub stripe_connect_refresh_url: Option<String>,
    pub stripe_connect_return_url: Option<String>,
    pub stripe_live_transfers_required: bool,
    pub tms_shared_secret: Option<String>,
    pub tms_reconciliation_worker_enabled: bool,
    pub tms_reconciliation_interval_seconds: u64,
    pub tms_retry_worker_enabled: bool,
    pub tms_retry_interval_seconds: u64,
    pub tms_retry_batch_size: i64,
    pub tms_retry_max_attempts: i32,
    pub tms_stale_handoff_days: i64,
    pub mail_mailer: String,
    pub mail_host: Option<String>,
    pub mail_port: u16,
    pub mail_username: Option<String>,
    pub mail_password: Option<String>,
    pub mail_encryption: Option<String>,
    pub mail_from_address: String,
    pub mail_from_name: String,
    pub mail_fail_open: bool,
    pub mail_outbox_enabled: bool,
    pub mail_outbox_worker_enabled: bool,
    pub mail_outbox_batch_size: i64,
    pub mail_outbox_retry_interval_seconds: u64,
    pub mail_outbox_max_attempts: i32,
    pub portal_url: String,
}

impl RuntimeConfig {
    pub fn from_env() -> Self {
        Self {
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(3001),
            deployment_target: env::var("DEPLOYMENT_TARGET")
                .unwrap_or_else(|_| "ibm-server".to_string()),
            environment: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            public_base_url: env::var("PUBLIC_BASE_URL").ok(),
            cors_allowed_origins: allowed_origin_values(),
            run_migrations: env::var("RUN_MIGRATIONS")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(false),
            database_url: env::var("DATABASE_URL").ok(),
            document_storage_backend: env::var("DOCUMENT_STORAGE_BACKEND")
                .unwrap_or_else(|_| "local".to_string()),
            document_storage_root: env::var("DOCUMENT_STORAGE_ROOT")
                .unwrap_or_else(|_| "./runtime/document-storage".to_string()),
            object_storage_bucket: env::var("OBJECT_STORAGE_BUCKET").ok(),
            object_storage_region: env::var("OBJECT_STORAGE_REGION")
                .unwrap_or_else(|_| "us-south".to_string()),
            object_storage_endpoint: env::var("OBJECT_STORAGE_ENDPOINT").ok(),
            object_storage_access_key_id: env::var("OBJECT_STORAGE_ACCESS_KEY_ID").ok(),
            object_storage_secret_access_key: env::var("OBJECT_STORAGE_SECRET_ACCESS_KEY").ok(),
            object_storage_session_token: env::var("OBJECT_STORAGE_SESSION_TOKEN").ok(),
            object_storage_force_path_style: env::var("OBJECT_STORAGE_FORCE_PATH_STYLE")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(false),
            object_storage_prefix: env::var("OBJECT_STORAGE_PREFIX")
                .unwrap_or_else(|_| "load-documents".to_string()),
            stripe_webhook_shared_secret: env::var("STRIPE_WEBHOOK_SHARED_SECRET")
                .or_else(|_| env::var("STRIPE_WEBHOOK_PLATFORM_SECRET"))
                .or_else(|_| env::var("STRIPE_WEBHOOK_SECRET_PLATFORM"))
                .ok()
                .and_then(optional_env_value),
            stripe_webhook_connect_secret: env::var("STRIPE_WEBHOOK_CONNECT_SECRET")
                .or_else(|_| env::var("STRIPE_WEBHOOK_SECRET_CONNECT"))
                .ok()
                .and_then(optional_env_value),
            stripe_secret_key: env::var("STRIPE_SECRET")
                .or_else(|_| env::var("STRIPE_SECRET_KEY"))
                .ok()
                .and_then(optional_env_value),
            stripe_api_base_url: env::var("STRIPE_API_BASE_URL")
                .ok()
                .and_then(optional_env_value)
                .unwrap_or_else(|| "https://api.stripe.com/v1".to_string()),
            stripe_connect_refresh_url: env::var("STRIPE_CONNECT_REFRESH_URL")
                .ok()
                .and_then(optional_env_value),
            stripe_connect_return_url: env::var("STRIPE_CONNECT_RETURN_URL")
                .ok()
                .and_then(optional_env_value),
            stripe_live_transfers_required: env::var("STRIPE_LIVE_TRANSFERS_REQUIRED")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(false),
            tms_shared_secret: env::var("TMS_SHARED_SECRET").ok(),
            tms_reconciliation_worker_enabled: env::var("TMS_RECONCILIATION_WORKER_ENABLED")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(true),
            tms_reconciliation_interval_seconds: env::var("TMS_RECONCILIATION_INTERVAL_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(21_600)
                .clamp(300, 86_400),
            tms_retry_worker_enabled: env::var("TMS_RETRY_WORKER_ENABLED")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(true),
            tms_retry_interval_seconds: env::var("TMS_RETRY_INTERVAL_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(300)
                .clamp(30, 21_600),
            tms_retry_batch_size: env::var("TMS_RETRY_BATCH_SIZE")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(10)
                .clamp(1, 100),
            tms_retry_max_attempts: env::var("TMS_RETRY_MAX_ATTEMPTS")
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(5)
                .clamp(1, 25),
            tms_stale_handoff_days: env::var("TMS_STALE_HANDOFF_DAYS")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(30)
                .clamp(1, 365),
            mail_mailer: env::var("MAIL_MAILER")
                .or_else(|_| env::var("MAIL_DRIVER"))
                .unwrap_or_else(|_| "log".to_string()),
            mail_host: env::var("MAIL_HOST")
                .ok()
                .and_then(|value| optional_env_value(value)),
            mail_port: env::var("MAIL_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(587),
            mail_username: env::var("MAIL_USERNAME")
                .ok()
                .and_then(|value| optional_env_value(value)),
            mail_password: env::var("MAIL_PASSWORD")
                .ok()
                .and_then(|value| optional_env_value(value)),
            mail_encryption: env::var("MAIL_ENCRYPTION")
                .or_else(|_| env::var("MAIL_SCHEME"))
                .ok()
                .and_then(|value| optional_env_value(value)),
            mail_from_address: env::var("MAIL_FROM_ADDRESS")
                .ok()
                .and_then(optional_env_value)
                .unwrap_or_else(|| "noreply@stloads.com".to_string()),
            mail_from_name: env::var("MAIL_FROM_NAME")
                .ok()
                .and_then(optional_env_value)
                .unwrap_or_else(|| "STLoads".to_string()),
            mail_fail_open: env::var("MAIL_FAIL_OPEN")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(true),
            mail_outbox_enabled: env::var("MAIL_OUTBOX_ENABLED")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(true),
            mail_outbox_worker_enabled: env::var("MAIL_OUTBOX_WORKER_ENABLED")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(true),
            mail_outbox_batch_size: env::var("MAIL_OUTBOX_BATCH_SIZE")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(25)
                .clamp(1, 250),
            mail_outbox_retry_interval_seconds: env::var("MAIL_OUTBOX_RETRY_INTERVAL_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(30)
                .clamp(5, 3600),
            mail_outbox_max_attempts: env::var("MAIL_OUTBOX_MAX_ATTEMPTS")
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(8)
                .clamp(1, 50),
            portal_url: env::var("PORTAL_URL")
                .or_else(|_| env::var("APP_URL"))
                .ok()
                .and_then(optional_env_value)
                .or_else(|| {
                    env::var("FRONTEND_PUBLIC_URL")
                        .ok()
                        .and_then(optional_env_value)
                })
                .or_else(|| {
                    env::var("PUBLIC_BASE_URL")
                        .ok()
                        .and_then(optional_env_value)
                })
                .unwrap_or_else(|| "https://portal.stloads.com".to_string()),
        }
    }
}

fn optional_env_value(value: String) -> Option<String> {
    let trimmed = value.trim().trim_matches('"').to_string();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        None
    } else {
        Some(trimmed)
    }
}

fn allowed_origin_values() -> Vec<String> {
    let mut values = Vec::new();

    if let Ok(raw) = env::var("CORS_ALLOWED_ORIGINS") {
        for value in raw.split(',') {
            let trimmed = value.trim().trim_matches('"');
            if !trimmed.is_empty() {
                values.push(trimmed.to_string());
            }
        }
    }

    for fallback in ["FRONTEND_PUBLIC_URL", "PUBLIC_BASE_URL"] {
        if let Ok(value) = env::var(fallback) {
            if let Some(trimmed) = optional_env_value(value) {
                values.push(trimmed);
            }
        }
    }

    values.sort();
    values.dedup();
    values
}
