-- Phase 11 partner API authentication and request-signing foundations.

CREATE TABLE IF NOT EXISTS partner_api_clients (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    actor_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    client_name VARCHAR(160) NOT NULL,
    key_prefix VARCHAR(32) NOT NULL UNIQUE,
    key_hash VARCHAR(128) NOT NULL UNIQUE,
    scopes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    require_request_signature BOOLEAN NOT NULL DEFAULT TRUE,
    last_used_at TIMESTAMP(6) NULL,
    expires_at TIMESTAMP(6) NULL,
    created_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_partner_api_clients_status
        CHECK (status IN ('active', 'disabled', 'rotating', 'expired')),
    CONSTRAINT chk_partner_api_clients_rate_limit
        CHECK (rate_limit_per_minute BETWEEN 1 AND 10000)
);

CREATE INDEX IF NOT EXISTS idx_partner_api_clients_org_status
    ON partner_api_clients (organization_id, status);

CREATE INDEX IF NOT EXISTS idx_partner_api_clients_actor
    ON partner_api_clients (actor_user_id);

CREATE TABLE IF NOT EXISTS partner_api_auth_events (
    id BIGSERIAL PRIMARY KEY,
    partner_api_client_id BIGINT NULL REFERENCES partner_api_clients(id) ON DELETE SET NULL,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE SET NULL,
    key_prefix VARCHAR(32) NULL,
    request_id VARCHAR(128) NULL,
    route_path VARCHAR(256) NOT NULL,
    method VARCHAR(16) NOT NULL,
    auth_result VARCHAR(32) NOT NULL,
    failure_reason TEXT NULL,
    scopes_checked TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    signature_required BOOLEAN NOT NULL DEFAULT TRUE,
    rate_limit_per_minute INTEGER NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_partner_api_auth_events_result
        CHECK (auth_result IN ('accepted', 'rejected', 'rate_limited'))
);

CREATE INDEX IF NOT EXISTS idx_partner_api_auth_events_client_created
    ON partner_api_auth_events (partner_api_client_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_partner_api_auth_events_result_created
    ON partner_api_auth_events (auth_result, created_at DESC);
