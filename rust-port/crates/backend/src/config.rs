use std::env;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub bind_addr: String,
    pub port: u16,
    pub deployment_target: String,
    pub environment: String,
    pub runtime_mode: String,
    pub log_format: String,
    pub otel_exporter_endpoint: Option<String>,
    pub public_base_url: Option<String>,
    pub cors_allowed_origins: Vec<String>,
    pub run_migrations: bool,
    pub database_url: Option<String>,
    pub database_schema: Option<String>,
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
    pub kill_switch_payments: bool,
    pub kill_switch_booking: bool,
    pub kill_switch_tms_pushes: bool,
    pub kill_switch_notifications: bool,
    pub kill_switch_document_uploads: bool,
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
            runtime_mode: env::var("STLOADS_RUNTIME_MODE")
                .or_else(|_| env::var("RUNTIME_MODE"))
                .unwrap_or_else(|_| "web".to_string()),
            log_format: env::var("LOG_FORMAT").unwrap_or_else(|_| {
                if env::var("APP_ENV")
                    .unwrap_or_else(|_| "development".to_string())
                    .trim()
                    .eq_ignore_ascii_case("production")
                {
                    "json".to_string()
                } else {
                    "pretty".to_string()
                }
            }),
            otel_exporter_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .ok()
                .and_then(optional_env_value),
            public_base_url: env::var("PUBLIC_BASE_URL").ok(),
            cors_allowed_origins: allowed_origin_values(),
            run_migrations: env::var("RUN_MIGRATIONS")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(false),
            database_url: env::var("DATABASE_URL").ok(),
            database_schema: env::var("STLOADS_DATABASE_SCHEMA")
                .or_else(|_| env::var("DATABASE_SCHEMA"))
                .ok()
                .and_then(optional_env_value),
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
            mail_host: env::var("MAIL_HOST").ok().and_then(optional_env_value),
            mail_port: env::var("MAIL_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(587),
            mail_username: env::var("MAIL_USERNAME").ok().and_then(optional_env_value),
            mail_password: env::var("MAIL_PASSWORD").ok().and_then(optional_env_value),
            mail_encryption: env::var("MAIL_ENCRYPTION")
                .or_else(|_| env::var("MAIL_SCHEME"))
                .ok()
                .and_then(optional_env_value),
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
            kill_switch_payments: env_bool("KILL_SWITCH_PAYMENTS", false),
            kill_switch_booking: env_bool("KILL_SWITCH_BOOKING", false),
            kill_switch_tms_pushes: env_bool("KILL_SWITCH_TMS_PUSHES", false),
            kill_switch_notifications: env_bool("KILL_SWITCH_NOTIFICATIONS", false),
            kill_switch_document_uploads: env_bool("KILL_SWITCH_DOCUMENT_UPLOADS", false),
        }
    }

    pub fn is_production(&self) -> bool {
        self.environment.trim().eq_ignore_ascii_case("production")
    }

    pub fn should_start_web(&self) -> bool {
        matches!(
            self.runtime_mode.trim().to_ascii_lowercase().as_str(),
            "web" | "all"
        )
    }

    pub fn should_start_workers(&self) -> bool {
        matches!(
            self.runtime_mode.trim().to_ascii_lowercase().as_str(),
            "worker" | "workers" | "all"
        )
    }

    pub fn should_emit_json_logs(&self) -> bool {
        self.log_format.trim().eq_ignore_ascii_case("json") || self.is_production()
    }

    pub fn validate_for_startup(&self) -> Result<(), String> {
        if !self.is_production() {
            return Ok(());
        }

        let mut errors = Vec::new();
        if !matches!(
            self.runtime_mode.trim().to_ascii_lowercase().as_str(),
            "web" | "worker" | "workers" | "all"
        ) {
            errors.push("STLOADS_RUNTIME_MODE must be one of web, worker, workers, or all".into());
        }
        if !self.should_emit_json_logs() {
            errors.push("production LOG_FORMAT must be json".into());
        }

        require_option(&mut errors, "DATABASE_URL", self.database_url.as_deref());
        require_option(
            &mut errors,
            "PUBLIC_BASE_URL",
            self.public_base_url.as_deref(),
        );
        require_value(&mut errors, "PORTAL_URL", &self.portal_url);

        if self.cors_allowed_origins.is_empty() {
            errors.push("CORS_ALLOWED_ORIGINS must include at least one production origin".into());
        }
        if self
            .cors_allowed_origins
            .iter()
            .any(|origin| origin.trim() == "*")
        {
            errors.push("CORS_ALLOWED_ORIGINS cannot include '*' in production".into());
        }
        for origin in &self.cors_allowed_origins {
            require_value(&mut errors, "CORS_ALLOWED_ORIGINS", origin);
        }

        if self
            .document_storage_backend
            .trim()
            .eq_ignore_ascii_case("local")
        {
            errors.push("DOCUMENT_STORAGE_BACKEND cannot be local in production".into());
        }
        require_value(
            &mut errors,
            "DOCUMENT_STORAGE_BACKEND",
            &self.document_storage_backend,
        );
        require_option(
            &mut errors,
            "OBJECT_STORAGE_BUCKET",
            self.object_storage_bucket.as_deref(),
        );
        require_value(
            &mut errors,
            "OBJECT_STORAGE_REGION",
            &self.object_storage_region,
        );
        require_option(
            &mut errors,
            "OBJECT_STORAGE_ENDPOINT",
            self.object_storage_endpoint.as_deref(),
        );
        require_option(
            &mut errors,
            "OBJECT_STORAGE_ACCESS_KEY_ID",
            self.object_storage_access_key_id.as_deref(),
        );
        require_option(
            &mut errors,
            "OBJECT_STORAGE_SECRET_ACCESS_KEY",
            self.object_storage_secret_access_key.as_deref(),
        );

        if self.stripe_live_transfers_required {
            require_option(
                &mut errors,
                "STRIPE_SECRET",
                self.stripe_secret_key.as_deref(),
            );
            require_option(
                &mut errors,
                "STRIPE_WEBHOOK_SHARED_SECRET",
                self.stripe_webhook_shared_secret.as_deref(),
            );
            require_option(
                &mut errors,
                "STRIPE_WEBHOOK_CONNECT_SECRET",
                self.stripe_webhook_connect_secret.as_deref(),
            );
            require_option(
                &mut errors,
                "STRIPE_CONNECT_REFRESH_URL",
                self.stripe_connect_refresh_url.as_deref(),
            );
            require_option(
                &mut errors,
                "STRIPE_CONNECT_RETURN_URL",
                self.stripe_connect_return_url.as_deref(),
            );
        }

        require_option(
            &mut errors,
            "TMS_SHARED_SECRET",
            self.tms_shared_secret.as_deref(),
        );

        if !self.mail_mailer.trim().eq_ignore_ascii_case("smtp") {
            errors.push("MAIL_MAILER must be smtp in production".into());
        }
        require_option(&mut errors, "MAIL_HOST", self.mail_host.as_deref());
        require_value(&mut errors, "MAIL_FROM_ADDRESS", &self.mail_from_address);
        if self.mail_fail_open {
            errors.push("MAIL_FAIL_OPEN must be false in production".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    pub fn has_required_object_storage_config(&self) -> bool {
        !self
            .document_storage_backend
            .trim()
            .eq_ignore_ascii_case("local")
            && !is_placeholder_value(&self.document_storage_backend)
            && self
                .object_storage_bucket
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
            && !is_placeholder_value(&self.object_storage_region)
            && self
                .object_storage_endpoint
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
            && self
                .object_storage_access_key_id
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
            && self
                .object_storage_secret_access_key
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
    }

    pub fn has_required_stripe_config(&self) -> bool {
        self.stripe_secret_key
            .as_deref()
            .is_some_and(|value| !is_placeholder_value(value))
            && self
                .stripe_webhook_shared_secret
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
            && self
                .stripe_webhook_connect_secret
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
            && self
                .stripe_connect_refresh_url
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
            && self
                .stripe_connect_return_url
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
    }

    pub fn has_required_mail_config(&self) -> bool {
        self.mail_mailer.trim().eq_ignore_ascii_case("smtp")
            && self
                .mail_host
                .as_deref()
                .is_some_and(|value| !is_placeholder_value(value))
            && !is_placeholder_value(&self.mail_from_address)
            && !self.mail_fail_open
    }

    pub fn has_required_worker_config(&self) -> bool {
        self.tms_shared_secret
            .as_deref()
            .is_some_and(|value| !is_placeholder_value(value))
            && self.tms_retry_worker_enabled
            && self.tms_reconciliation_worker_enabled
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

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(default)
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
        if let Ok(value) = env::var(fallback)
            && let Some(trimmed) = optional_env_value(value)
        {
            values.push(trimmed);
        }
    }

    values.sort();
    values.dedup();
    values
}

fn require_option(errors: &mut Vec<String>, name: &str, value: Option<&str>) {
    match value {
        Some(value) => require_value(errors, name, value),
        None => errors.push(format!("{name} is required in production")),
    }
}

fn require_value(errors: &mut Vec<String>, name: &str, value: &str) {
    if is_placeholder_value(value) {
        errors.push(format!(
            "{name} is required in production and cannot be a placeholder"
        ));
    }
}

fn is_placeholder_value(value: &str) -> bool {
    let trimmed = value.trim().trim_matches('"').trim_matches('\'');
    if trimmed.is_empty() {
        return true;
    }

    let lower = trimmed.to_ascii_lowercase();
    lower == "null"
        || lower == "none"
        || lower == "todo"
        || lower == "tbd"
        || lower == "changeme"
        || lower == "change_me"
        || lower == "replace_me"
        || lower == "placeholder"
        || lower.contains("changeme")
        || lower.contains("placeholder")
        || lower.starts_with("your_")
        || lower.starts_with("your-")
        || (lower.contains("example.") && !lower.contains("stloads"))
        || lower.starts_with("http://localhost")
        || lower.starts_with("https://localhost")
}

#[cfg(test)]
mod tests {
    use super::RuntimeConfig;

    #[test]
    fn development_allows_current_permissive_defaults() {
        let config = RuntimeConfig {
            bind_addr: "127.0.0.1".into(),
            port: 3001,
            deployment_target: "local".into(),
            environment: "development".into(),
            runtime_mode: "web".into(),
            log_format: "pretty".into(),
            otel_exporter_endpoint: None,
            public_base_url: None,
            cors_allowed_origins: Vec::new(),
            run_migrations: false,
            database_url: None,
            database_schema: None,
            document_storage_backend: "local".into(),
            document_storage_root: "./runtime/document-storage".into(),
            object_storage_bucket: None,
            object_storage_region: "us-south".into(),
            object_storage_endpoint: None,
            object_storage_access_key_id: None,
            object_storage_secret_access_key: None,
            object_storage_session_token: None,
            object_storage_force_path_style: false,
            object_storage_prefix: "load-documents".into(),
            stripe_webhook_shared_secret: None,
            stripe_webhook_connect_secret: None,
            stripe_secret_key: None,
            stripe_api_base_url: "https://api.stripe.com/v1".into(),
            stripe_connect_refresh_url: None,
            stripe_connect_return_url: None,
            stripe_live_transfers_required: false,
            tms_shared_secret: None,
            tms_reconciliation_worker_enabled: true,
            tms_reconciliation_interval_seconds: 21_600,
            tms_retry_worker_enabled: true,
            tms_retry_interval_seconds: 300,
            tms_retry_batch_size: 10,
            tms_retry_max_attempts: 5,
            tms_stale_handoff_days: 30,
            mail_mailer: "log".into(),
            mail_host: None,
            mail_port: 587,
            mail_username: None,
            mail_password: None,
            mail_encryption: None,
            mail_from_address: "noreply@stloads.com".into(),
            mail_from_name: "STLoads".into(),
            mail_fail_open: true,
            mail_outbox_enabled: true,
            mail_outbox_worker_enabled: true,
            mail_outbox_batch_size: 25,
            mail_outbox_retry_interval_seconds: 30,
            mail_outbox_max_attempts: 8,
            portal_url: "https://portal.stloads.com".into(),
            kill_switch_payments: false,
            kill_switch_booking: false,
            kill_switch_tms_pushes: false,
            kill_switch_notifications: false,
            kill_switch_document_uploads: false,
        };

        assert!(config.validate_for_startup().is_ok());
    }

    #[test]
    fn production_accepts_complete_runtime_config() {
        assert!(production_config().validate_for_startup().is_ok());
    }

    #[test]
    fn production_rejects_missing_database_url() {
        let mut config = production_config();
        config.database_url = None;

        let error = config
            .validate_for_startup()
            .expect_err("config should fail");

        assert!(error.contains("DATABASE_URL"));
    }

    #[test]
    fn production_rejects_permissive_cors() {
        let mut config = production_config();
        config.cors_allowed_origins = vec!["*".into()];

        let error = config
            .validate_for_startup()
            .expect_err("config should fail");

        assert!(error.contains("CORS_ALLOWED_ORIGINS"));
        assert!(error.contains("'*'"));
    }

    #[test]
    fn production_rejects_local_document_storage() {
        let mut config = production_config();
        config.document_storage_backend = "local".into();

        let error = config
            .validate_for_startup()
            .expect_err("config should fail");

        assert!(error.contains("DOCUMENT_STORAGE_BACKEND"));
    }

    #[test]
    fn production_rejects_placeholder_secrets_and_urls() {
        let mut config = production_config();
        config.stripe_secret_key = Some("changeme".into());
        config.public_base_url = Some("https://api.example.test".into());

        let error = config
            .validate_for_startup()
            .expect_err("config should fail");

        assert!(error.contains("STRIPE_SECRET"));
        assert!(error.contains("PUBLIC_BASE_URL"));
    }

    #[test]
    fn production_allows_manual_finance_without_stripe() {
        let mut config = production_config();
        config.stripe_live_transfers_required = false;
        config.stripe_secret_key = None;
        config.stripe_webhook_shared_secret = None;
        config.stripe_webhook_connect_secret = None;
        config.stripe_connect_refresh_url = None;
        config.stripe_connect_return_url = None;

        assert!(config.validate_for_startup().is_ok());
    }

    #[test]
    fn production_rejects_fail_open_mailer() {
        let mut config = production_config();
        config.mail_fail_open = true;

        let error = config
            .validate_for_startup()
            .expect_err("config should fail");

        assert!(error.contains("MAIL_FAIL_OPEN"));
    }

    fn production_config() -> RuntimeConfig {
        RuntimeConfig {
            bind_addr: "0.0.0.0".into(),
            port: 3001,
            deployment_target: "ibm-code-engine".into(),
            environment: "production".into(),
            runtime_mode: "web".into(),
            log_format: "json".into(),
            otel_exporter_endpoint: None,
            public_base_url: Some("https://api.stloads.com".into()),
            cors_allowed_origins: vec!["https://portal.stloads.com".into()],
            run_migrations: false,
            database_url: Some("postgres://stloads:secret@db.stloads.internal:5432/stloads".into()),
            database_schema: Some("public".into()),
            document_storage_backend: "s3".into(),
            document_storage_root: "./runtime/document-storage".into(),
            object_storage_bucket: Some("stloads-prod-documents".into()),
            object_storage_region: "us-south".into(),
            object_storage_endpoint: Some(
                "https://s3.us-south.cloud-object-storage.appdomain.cloud".into(),
            ),
            object_storage_access_key_id: Some("prod-access-key".into()),
            object_storage_secret_access_key: Some("prod-secret-key".into()),
            object_storage_session_token: None,
            object_storage_force_path_style: true,
            object_storage_prefix: "load-documents".into(),
            stripe_webhook_shared_secret: Some("whsec_platform_secret".into()),
            stripe_webhook_connect_secret: Some("whsec_connect_secret".into()),
            stripe_secret_key: Some("sk_live_real_secret".into()),
            stripe_api_base_url: "https://api.stripe.com/v1".into(),
            stripe_connect_refresh_url: Some(
                "https://portal.stloads.com/settings/payouts/refresh".into(),
            ),
            stripe_connect_return_url: Some(
                "https://portal.stloads.com/settings/payouts/return".into(),
            ),
            stripe_live_transfers_required: true,
            tms_shared_secret: Some("prod-tms-secret".into()),
            tms_reconciliation_worker_enabled: true,
            tms_reconciliation_interval_seconds: 21_600,
            tms_retry_worker_enabled: true,
            tms_retry_interval_seconds: 300,
            tms_retry_batch_size: 10,
            tms_retry_max_attempts: 5,
            tms_stale_handoff_days: 30,
            mail_mailer: "smtp".into(),
            mail_host: Some("smtp.stloads.com".into()),
            mail_port: 587,
            mail_username: Some("mailer".into()),
            mail_password: Some("mail-secret".into()),
            mail_encryption: Some("tls".into()),
            mail_from_address: "noreply@stloads.com".into(),
            mail_from_name: "STLoads".into(),
            mail_fail_open: false,
            mail_outbox_enabled: true,
            mail_outbox_worker_enabled: true,
            mail_outbox_batch_size: 25,
            mail_outbox_retry_interval_seconds: 30,
            mail_outbox_max_attempts: 8,
            portal_url: "https://portal.stloads.com".into(),
            kill_switch_payments: false,
            kill_switch_booking: false,
            kill_switch_tms_pushes: false,
            kill_switch_notifications: false,
            kill_switch_document_uploads: false,
        }
    }
}
