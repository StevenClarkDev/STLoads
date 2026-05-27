-- Correlate API requests across async jobs, integrations, and realtime events.

ALTER TABLE email_outbox
    ADD COLUMN IF NOT EXISTS request_id VARCHAR(191) NULL;

CREATE INDEX IF NOT EXISTS idx_email_outbox_request_id
    ON email_outbox (request_id)
    WHERE request_id IS NOT NULL;

ALTER TABLE stloads_handoffs
    ADD COLUMN IF NOT EXISTS request_id VARCHAR(191) NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_request_id
    ON stloads_handoffs (request_id)
    WHERE request_id IS NOT NULL;

ALTER TABLE stloads_handoff_events
    ADD COLUMN IF NOT EXISTS request_id VARCHAR(191) NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoff_events_request_id
    ON stloads_handoff_events (request_id)
    WHERE request_id IS NOT NULL;

ALTER TABLE stloads_sync_errors
    ADD COLUMN IF NOT EXISTS request_id VARCHAR(191) NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_sync_errors_request_id
    ON stloads_sync_errors (request_id)
    WHERE request_id IS NOT NULL;

ALTER TABLE stloads_reconciliation_log
    ADD COLUMN IF NOT EXISTS request_id VARCHAR(191) NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_reconciliation_log_request_id
    ON stloads_reconciliation_log (request_id)
    WHERE request_id IS NOT NULL;
