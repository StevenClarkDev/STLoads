-- Phase 11 external idempotency and inbound event de-dupe ledger.

CREATE TABLE IF NOT EXISTS external_idempotency_records (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE SET NULL,
    partner_api_client_id BIGINT NULL REFERENCES partner_api_clients(id) ON DELETE SET NULL,
    idempotency_scope VARCHAR(96) NOT NULL,
    idempotency_key VARCHAR(160) NOT NULL,
    request_hash VARCHAR(128) NULL,
    response_ref VARCHAR(256) NULL,
    response_payload JSONB NULL,
    processing_status VARCHAR(32) NOT NULL DEFAULT 'processing',
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP(6) NULL,
    CONSTRAINT chk_external_idempotency_status
        CHECK (processing_status IN ('processing', 'completed', 'failed'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_external_idempotency_scope_key
    ON external_idempotency_records (idempotency_scope, idempotency_key);

CREATE TABLE IF NOT EXISTS external_event_dedupe_records (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE SET NULL,
    source_system VARCHAR(80) NOT NULL,
    event_type VARCHAR(120) NOT NULL,
    external_event_id VARCHAR(180) NOT NULL,
    request_id VARCHAR(128) NULL,
    processing_status VARCHAR(32) NOT NULL DEFAULT 'processing',
    first_seen_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP(6) NULL,
    payload JSONB NULL,
    result_summary TEXT NULL,
    CONSTRAINT chk_external_event_dedupe_status
        CHECK (processing_status IN ('processing', 'completed', 'failed', 'ignored_duplicate'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_external_event_source_type_id
    ON external_event_dedupe_records (source_system, event_type, external_event_id);

CREATE INDEX IF NOT EXISTS idx_external_event_status_seen
    ON external_event_dedupe_records (processing_status, first_seen_at DESC);
