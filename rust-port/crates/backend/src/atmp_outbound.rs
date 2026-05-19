use std::time::Duration;

use db::{DbPool, tms::AtmpOutboundEventRecord};
use reqwest::Client;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AtmpOutboundOptions {
    pub default_base_url: Option<String>,
    pub batch_size: i64,
    pub max_attempts: i32,
}

#[derive(Debug, Clone, Default)]
pub struct AtmpOutboundSummary {
    pub scanned: usize,
    pub delivered: usize,
    pub failed: usize,
    pub dead_lettered: usize,
}

pub fn start_worker(state: AppState) {
    if state.pool.is_none() || !state.config.atmp_outbound_worker_enabled {
        return;
    }

    tokio::spawn(async move {
        let interval = Duration::from_secs(state.config.atmp_outbound_interval_seconds);
        info!(
            interval_seconds = state.config.atmp_outbound_interval_seconds,
            batch_size = state.config.atmp_outbound_batch_size,
            max_attempts = state.config.atmp_outbound_max_attempts,
            "ATMP outbound event worker started"
        );

        loop {
            if let Some(pool) = state.pool.as_ref() {
                let options = AtmpOutboundOptions {
                    default_base_url: state.config.atmp_outbound_base_url.clone(),
                    batch_size: state.config.atmp_outbound_batch_size,
                    max_attempts: state.config.atmp_outbound_max_attempts,
                };

                match process_due_events(pool, options).await {
                    Ok(summary)
                        if summary.scanned > 0
                            || summary.failed > 0
                            || summary.dead_lettered > 0 =>
                    {
                        info!(
                            scanned = summary.scanned,
                            delivered = summary.delivered,
                            failed = summary.failed,
                            dead_lettered = summary.dead_lettered,
                            "ATMP outbound worker processed events"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => warn!(error = %error, "ATMP outbound worker failed"),
                }
            }

            sleep(interval).await;
        }
    });
}

pub async fn process_due_events(
    pool: &DbPool,
    options: AtmpOutboundOptions,
) -> Result<AtmpOutboundSummary, sqlx::Error> {
    let client = Client::new();
    let events = db::tms::claim_due_atmp_outbound_events(pool, options.batch_size).await?;
    let mut summary = AtmpOutboundSummary {
        scanned: events.len(),
        ..AtmpOutboundSummary::default()
    };

    for event in events {
        match deliver_event(&client, &event, options.default_base_url.as_deref()).await {
            Ok(()) => {
                db::tms::mark_atmp_outbound_event_delivered(pool, event.id).await?;
                summary.delivered += 1;
            }
            Err(error) => {
                let dead_lettered = event.attempt_count + 1 >= options.max_attempts;
                db::tms::mark_atmp_outbound_event_failed(
                    pool,
                    event.id,
                    &error,
                    event.attempt_count + 1,
                    dead_lettered,
                )
                .await?;
                if dead_lettered {
                    summary.dead_lettered += 1;
                } else {
                    summary.failed += 1;
                }
            }
        }
    }

    Ok(summary)
}

async fn deliver_event(
    client: &Client,
    event: &AtmpOutboundEventRecord,
    default_base_url: Option<&str>,
) -> Result<(), String> {
    let target_url = event
        .target_url
        .clone()
        .or_else(|| default_base_url.map(default_event_url))
        .ok_or_else(|| "ATMP outbound base URL is not configured.".to_string())?;

    let response = client
        .post(&target_url)
        .header("x-stloads-event-id", &event.event_id)
        .header("x-stloads-tenant", &event.tenant_id)
        .header("x-stloads-event-type", &event.event_type)
        .json(&event.payload)
        .send()
        .await
        .map_err(|error| format!("callback request failed: {error}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "callback returned HTTP {} for {}",
            response.status(),
            event.event_type
        ))
    }
}

fn default_event_url(base_url: &str) -> String {
    format!("{}/api/stloads/events", base_url.trim_end_matches('/'))
}
