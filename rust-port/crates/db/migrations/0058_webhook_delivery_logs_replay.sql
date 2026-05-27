-- Phase 11 outbound webhook delivery logs and replay queue.

CREATE TABLE IF NOT EXISTS outbound_webhook_endpoints (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    endpoint_name VARCHAR(160) NOT NULL,
    target_url TEXT NOT NULL,
    event_types TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    signing_secret_ref VARCHAR(160) NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_outbound_webhook_endpoints_status
        CHECK (status IN ('active', 'disabled', 'paused'))
);

CREATE INDEX IF NOT EXISTS idx_outbound_webhook_endpoints_org_status
    ON outbound_webhook_endpoints (organization_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS ux_outbound_webhook_endpoints_org_name
    ON outbound_webhook_endpoints (organization_id, endpoint_name);

CREATE TABLE IF NOT EXISTS webhook_delivery_logs (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    endpoint_id BIGINT NULL REFERENCES outbound_webhook_endpoints(id) ON DELETE SET NULL,
    event_type VARCHAR(120) NOT NULL,
    event_id VARCHAR(180) NOT NULL,
    delivery_status VARCHAR(32) NOT NULL DEFAULT 'queued',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMP(6) NULL,
    last_attempt_at TIMESTAMP(6) NULL,
    response_status_code INTEGER NULL,
    response_latency_ms INTEGER NULL,
    response_body_excerpt TEXT NULL,
    request_payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    request_headers JSONB NOT NULL DEFAULT '{}'::JSONB,
    dead_letter_reason TEXT NULL,
    replay_of_delivery_id BIGINT NULL REFERENCES webhook_delivery_logs(id) ON DELETE SET NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_webhook_delivery_logs_status
        CHECK (delivery_status IN ('queued', 'delivered', 'retrying', 'failed', 'dead_letter', 'replay_queued'))
);

CREATE INDEX IF NOT EXISTS idx_webhook_delivery_logs_org_status
    ON webhook_delivery_logs (organization_id, delivery_status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_webhook_delivery_logs_endpoint_event
    ON webhook_delivery_logs (endpoint_id, event_type, event_id);
