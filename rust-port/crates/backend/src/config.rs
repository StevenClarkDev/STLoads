use std::env;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub bind_addr: String,
    pub port: u16,
    pub deployment_target: String,
    pub environment: String,
    pub public_base_url: Option<String>,
    pub run_migrations: bool,
    pub database_url: Option<String>,
    pub stripe_webhook_shared_secret: Option<String>,
    pub tms_shared_secret: Option<String>,
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
            run_migrations: env::var("RUN_MIGRATIONS")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(false),
            database_url: env::var("DATABASE_URL").ok(),
            stripe_webhook_shared_secret: env::var("STRIPE_WEBHOOK_SHARED_SECRET").ok(),
            tms_shared_secret: env::var("TMS_SHARED_SECRET").ok(),
        }
    }
}
