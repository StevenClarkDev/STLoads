use tokio::time::{Duration, sleep};
use tracing::{info, warn};

use crate::state::AppState;

pub fn start_tms_workers(state: AppState) {
    if state.pool.is_none() {
        return;
    }

    if state.config.tms_retry_worker_enabled {
        let retry_state = state.clone();
        tokio::spawn(async move {
            run_retry_worker(retry_state).await;
        });
    }

    if state.config.tms_reconciliation_worker_enabled {
        tokio::spawn(async move {
            run_reconciliation_worker(state).await;
        });
    }
}

async fn run_retry_worker(state: AppState) {
    let interval = Duration::from_secs(state.config.tms_retry_interval_seconds);
    info!(
        interval_seconds = state.config.tms_retry_interval_seconds,
        batch_size = state.config.tms_retry_batch_size,
        max_attempts = state.config.tms_retry_max_attempts,
        "TMS retry worker started"
    );

    loop {
        if let Some(pool) = state.pool.as_ref() {
            match db::tms::process_retryable_handoffs(
                pool,
                state.config.tms_retry_batch_size,
                state.config.tms_retry_max_attempts,
            )
            .await
            {
                Ok(summary) if summary.scanned > 0 || summary.failed > 0 => {
                    info!(
                        scanned = summary.scanned,
                        published = summary.published,
                        failed = summary.failed,
                        "TMS retry worker processed handoffs"
                    );
                }
                Ok(_) => {}
                Err(error) => warn!(error = %error, "TMS retry worker failed"),
            }
        }

        sleep(interval).await;
    }
}

async fn run_reconciliation_worker(state: AppState) {
    let interval = Duration::from_secs(state.config.tms_reconciliation_interval_seconds);
    info!(
        interval_seconds = state.config.tms_reconciliation_interval_seconds,
        stale_days = state.config.tms_stale_handoff_days,
        "TMS reconciliation worker started"
    );

    loop {
        if let Some(pool) = state.pool.as_ref() {
            match db::tms::run_reconciliation_scan(pool, state.config.tms_stale_handoff_days).await
            {
                Ok(summary)
                    if summary.auto_archived > 0
                        || summary.cancelled_still_live > 0
                        || summary.delivered_still_open > 0
                        || summary.stale_handoffs > 0 =>
                {
                    info!(
                        auto_archived = summary.auto_archived,
                        cancelled_still_live = summary.cancelled_still_live,
                        delivered_still_open = summary.delivered_still_open,
                        stale_handoffs = summary.stale_handoffs,
                        "TMS reconciliation worker found drift"
                    );
                }
                Ok(_) => {}
                Err(error) => warn!(error = %error, "TMS reconciliation worker failed"),
            }
        }

        sleep(interval).await;
    }
}
