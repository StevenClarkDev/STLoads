use db::DbPool;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::{
    config::RuntimeConfig,
    document_storage::DocumentStorageService,
    email::EmailService,
    rate_limit::{LockoutPolicy, RateLimitDecision, RateLimitPolicy, RateLimiter},
    realtime_bus::RoutedRealtimeEvent,
    stripe::StripeService,
    tms_scheduler,
};

#[derive(Clone)]
pub struct AppState {
    pub config: RuntimeConfig,
    pub pool: Option<DbPool>,
    pub document_storage: DocumentStorageService,
    pub email: EmailService,
    pub stripe: StripeService,
    pub rate_limiter: RateLimiter,
    pub realtime_tx: broadcast::Sender<RoutedRealtimeEvent>,
}

impl AppState {
    pub async fn from_env() -> Self {
        let config = RuntimeConfig::from_env();
        if let Err(error) = config.validate_for_startup() {
            panic!("invalid production runtime configuration: {error}");
        }

        let (realtime_tx, _) = broadcast::channel(256);
        let document_storage = DocumentStorageService::from_config(&config);
        let stripe = StripeService::from_config(&config);
        let mut pool = None;

        if let Some(database_url) = config.database_url.as_deref() {
            match db::connect_with_schema(database_url, config.database_schema.as_deref()).await {
                Ok(connected_pool) => {
                    if config.run_migrations {
                        warn!(
                            "RUN_MIGRATIONS is ignored by the web runtime; use `cargo run -p backend --bin run_migrations` before deployment"
                        );
                    }

                    info!("database pool connected successfully");
                    pool = Some(connected_pool);
                }
                Err(error) => {
                    if config.is_production() {
                        panic!("database connection failed in production: {error}");
                    } else {
                        warn!(error = %error, "database connection failed; backend will serve fallback screen data until DATABASE_URL is fixed");
                    }
                }
            }
        } else {
            if config.is_production() {
                panic!("DATABASE_URL is not set in production");
            } else {
                warn!("DATABASE_URL is not set; backend will serve fallback screen data");
            }
        }

        let email = EmailService::from_config_with_pool(&config, pool.clone());
        email.start_outbox_worker();

        let state = Self {
            config,
            pool,
            document_storage,
            email,
            stripe,
            rate_limiter: RateLimiter::default(),
            realtime_tx,
        };
        tms_scheduler::start_tms_workers(state.clone());
        state
    }

    pub fn database_state(&self) -> &'static str {
        if self.pool.is_some() {
            "connected"
        } else {
            "disabled"
        }
    }

    pub fn publish_realtime(&self, event: RoutedRealtimeEvent) {
        let _ = self.realtime_tx.send(event);
    }

    pub async fn check_rate_limit(
        &self,
        policy: RateLimitPolicy,
        identity: impl AsRef<str>,
    ) -> RateLimitDecision {
        let identity = identity.as_ref();
        if let Some(pool) = self.pool.as_ref() {
            match db::security::check_rate_limit(
                pool,
                &security_key(policy.name, identity),
                policy.max_attempts.min(i32::MAX as u32) as i32,
                duration_seconds(policy.window),
            )
            .await
            {
                Ok(decision) => return decision.into(),
                Err(error) => warn!(
                    policy = policy.name,
                    error = %error,
                    "distributed rate-limit check failed; falling back to process-local limiter"
                ),
            }
        }

        self.rate_limiter.check(policy, identity)
    }

    pub async fn lockout_status(
        &self,
        policy: LockoutPolicy,
        identity: impl AsRef<str>,
    ) -> RateLimitDecision {
        let identity = identity.as_ref();
        if let Some(pool) = self.pool.as_ref() {
            match db::security::lockout_status(pool, &security_key("lockout", identity)).await {
                Ok(decision) => return decision.into(),
                Err(error) => warn!(
                    error = %error,
                    "distributed lockout status check failed; falling back to process-local limiter"
                ),
            }
        }

        self.rate_limiter.lockout_status(policy, identity)
    }

    pub async fn record_lockout_failure(
        &self,
        policy: LockoutPolicy,
        identity: impl AsRef<str>,
    ) -> RateLimitDecision {
        let identity = identity.as_ref();
        if let Some(pool) = self.pool.as_ref() {
            match db::security::record_lockout_failure(
                pool,
                &security_key("lockout", identity),
                policy.max_failures.min(i32::MAX as u32) as i32,
                duration_seconds(policy.window),
                duration_seconds(policy.lockout),
            )
            .await
            {
                Ok(decision) => return decision.into(),
                Err(error) => warn!(
                    error = %error,
                    "distributed lockout failure record failed; falling back to process-local limiter"
                ),
            }
        }

        self.rate_limiter.record_failure(policy, identity)
    }

    pub async fn record_lockout_success(&self, identity: impl AsRef<str>) {
        let identity = identity.as_ref();
        if let Some(pool) = self.pool.as_ref() {
            if let Err(error) =
                db::security::clear_lockout(pool, &security_key("lockout", identity)).await
            {
                warn!(
                    error = %error,
                    "distributed lockout clear failed; clearing process-local limiter"
                );
            }
        }

        self.rate_limiter.record_success(identity);
    }
}

fn security_key(scope: &str, identity: &str) -> String {
    format!(
        "{}:{}:{}",
        "stloads",
        scope,
        identity.trim().to_ascii_lowercase()
    )
}

fn duration_seconds(duration: std::time::Duration) -> i64 {
    duration.as_secs().clamp(1, i64::MAX as u64) as i64
}

impl From<db::security::ThrottleDecision> for RateLimitDecision {
    fn from(value: db::security::ThrottleDecision) -> Self {
        Self {
            allowed: value.allowed,
            retry_after_seconds: value.retry_after_seconds.max(0) as u64,
        }
    }
}
