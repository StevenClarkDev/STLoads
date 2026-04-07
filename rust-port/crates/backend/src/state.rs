use db::DbPool;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::{config::RuntimeConfig, realtime_bus::RoutedRealtimeEvent};

#[derive(Clone)]
pub struct AppState {
    pub config: RuntimeConfig,
    pub pool: Option<DbPool>,
    pub realtime_tx: broadcast::Sender<RoutedRealtimeEvent>,
}

impl AppState {
    pub async fn from_env() -> Self {
        let config = RuntimeConfig::from_env();
        let (realtime_tx, _) = broadcast::channel(256);
        let mut pool = None;

        if let Some(database_url) = config.database_url.as_deref() {
            match db::connect(database_url).await {
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

        Self {
            config,
            pool,
            realtime_tx,
        }
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
