use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::{Duration, Utc};
use db::DbPool;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::{
    config::RuntimeConfig, document_storage::DocumentStorageService, email::EmailService,
    integration_auth::IntegrationAuthState, realtime_bus::RoutedRealtimeEvent,
    stripe::StripeService, tms_scheduler,
};

#[derive(Clone)]
pub struct AppState {
    pub config: RuntimeConfig,
    pub pool: Option<DbPool>,
    pub document_storage: DocumentStorageService,
    pub email: EmailService,
    pub stripe: StripeService,
    pub integration_auth: IntegrationAuthState,
    pub realtime_tx: broadcast::Sender<RoutedRealtimeEvent>,
    pub security: SecurityState,
}

#[derive(Clone, Default)]
pub struct SecurityState {
    rate_buckets: Arc<Mutex<HashMap<String, SecurityRateBucket>>>,
    auth_failures: Arc<Mutex<HashMap<String, AuthFailureBucket>>>,
}

#[derive(Clone)]
struct SecurityRateBucket {
    window_start: chrono::DateTime<Utc>,
    count: u32,
}

#[derive(Clone)]
struct AuthFailureBucket {
    first_failure_at: chrono::DateTime<Utc>,
    failed_count: u32,
    locked_until: Option<chrono::DateTime<Utc>>,
}

impl AppState {
    pub async fn from_env() -> Self {
        let config = RuntimeConfig::from_env();
        let (realtime_tx, _) = broadcast::channel(256);
        let document_storage = DocumentStorageService::from_config(&config);
        let stripe = StripeService::from_config(&config);
        let mut pool = None;

        if let Some(database_url) = config.database_url.as_deref() {
            match db::connect_with_schema(database_url, config.database_schema.as_deref()).await {
                Ok(connected_pool) => {
                    if config.run_migrations {
                        match db::migrate(&connected_pool).await {
                            Ok(()) => info!("database migrations completed during startup"),
                            Err(error) => {
                                warn!(error = %error, "database migrations failed; continuing without aborting startup")
                            }
                        }
                    }

                    info!("database pool connected successfully");
                    pool = Some(connected_pool);
                }
                Err(error) => {
                    warn!(error = %error, "database connection failed; backend will serve fallback screen data until DATABASE_URL is fixed");
                }
            }
        } else {
            warn!("DATABASE_URL is not set; backend will serve fallback screen data");
        }

        let email = EmailService::from_config_with_pool(&config, pool.clone());
        email.start_outbox_worker();

        let state = Self {
            config,
            pool,
            document_storage,
            email,
            stripe,
            integration_auth: IntegrationAuthState::default(),
            realtime_tx,
            security: SecurityState::default(),
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

    pub fn realtime_receiver_count(&self) -> usize {
        self.realtime_tx.receiver_count()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_audit_event(
        &self,
        category: &'static str,
        route: &str,
        tenant_id: Option<&str>,
        actor_id: Option<&str>,
        correlation_id: Option<&str>,
        event_id: Option<&str>,
        atmp_load_id: Option<&str>,
        posting_id: Option<&str>,
        idempotency_key: Option<&str>,
    ) {
        info!(
            audit_category = category,
            tenant_id = tenant_id.unwrap_or("unknown"),
            actor_id = actor_id.unwrap_or("unknown"),
            route,
            correlation_id = correlation_id.unwrap_or("none"),
            event_id = event_id.unwrap_or("none"),
            atmp_load_id = atmp_load_id.unwrap_or("none"),
            posting_id = posting_id.unwrap_or("none"),
            idempotency_key = idempotency_key.unwrap_or("none"),
            immutable = true,
            "stloads immutable audit entry"
        );
    }

    pub fn enforce_rate_limit(
        &self,
        class: &str,
        key: &str,
        limit_per_minute: u32,
    ) -> Result<(), String> {
        let now = Utc::now();
        let mut buckets = self
            .security
            .rate_buckets
            .lock()
            .map_err(|_| "Security rate-limit state is unavailable.".to_string())?;
        let bucket = buckets
            .entry(key.to_string())
            .or_insert(SecurityRateBucket {
                window_start: now,
                count: 0,
            });

        if now - bucket.window_start >= Duration::seconds(60) {
            bucket.window_start = now;
            bucket.count = 0;
        }

        if bucket.count >= limit_per_minute {
            return Err(format!("{class} rate limit exceeded. Try again shortly."));
        }

        bucket.count += 1;
        Ok(())
    }

    pub fn auth_failure_locked_message(&self, email: &str) -> Option<String> {
        let now = Utc::now();
        let mut failures = self.security.auth_failures.lock().ok()?;
        let key = normalized_auth_key(email);
        let bucket = failures.get_mut(&key)?;
        if let Some(locked_until) = bucket.locked_until {
            if locked_until > now {
                return Some("Too many failed login attempts. Try again in 15 minutes.".into());
            }
        }
        if now - bucket.first_failure_at >= Duration::hours(1) {
            failures.remove(&key);
        }
        None
    }

    pub fn record_auth_failure(&self, email: &str) {
        let now = Utc::now();
        let Ok(mut failures) = self.security.auth_failures.lock() else {
            return;
        };
        let key = normalized_auth_key(email);
        let bucket = failures.entry(key).or_insert(AuthFailureBucket {
            first_failure_at: now,
            failed_count: 0,
            locked_until: None,
        });

        if now - bucket.first_failure_at >= Duration::hours(1) {
            bucket.first_failure_at = now;
            bucket.failed_count = 0;
            bucket.locked_until = None;
        }

        bucket.failed_count += 1;
        if bucket.failed_count >= 5 {
            bucket.locked_until = Some(now + Duration::minutes(15));
        }
    }

    pub fn clear_auth_failures(&self, email: &str) {
        if let Ok(mut failures) = self.security.auth_failures.lock() {
            failures.remove(&normalized_auth_key(email));
        }
    }
}

fn normalized_auth_key(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}
