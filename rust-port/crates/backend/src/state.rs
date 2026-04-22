use db::DbPool;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::{
    config::RuntimeConfig, document_storage::DocumentStorageService, email::EmailService,
    realtime_bus::RoutedRealtimeEvent, stripe::StripeService, tms_scheduler,
};

#[derive(Clone)]
pub struct AppState {
    pub config: RuntimeConfig,
    pub pool: Option<DbPool>,
    pub document_storage: DocumentStorageService,
    pub email: EmailService,
    pub stripe: StripeService,
    pub realtime_tx: broadcast::Sender<RoutedRealtimeEvent>,
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
}
