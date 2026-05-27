-- Phase 11 sandbox/demo tenant and API lifecycle governance.

CREATE TABLE IF NOT EXISTS sandbox_tenant_environments (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    environment_key VARCHAR(80) NOT NULL,
    display_name VARCHAR(180) NOT NULL,
    base_url TEXT NOT NULL,
    data_classification VARCHAR(40) NOT NULL DEFAULT 'synthetic',
    pii_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    production_payment_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    production_tms_push_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    production_notification_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    seeded_dataset_version VARCHAR(80) NOT NULL DEFAULT 'demo-v1',
    reset_status VARCHAR(32) NOT NULL DEFAULT 'ready',
    last_reset_at TIMESTAMP(6) NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_sandbox_tenant_data_classification
        CHECK (data_classification IN ('synthetic', 'masked', 'demo_only')),
    CONSTRAINT chk_sandbox_tenant_reset_status
        CHECK (reset_status IN ('ready', 'reset_queued', 'resetting', 'failed', 'disabled')),
    CONSTRAINT chk_sandbox_tenant_safety
        CHECK (
            pii_allowed = FALSE
            AND production_payment_blocked = TRUE
            AND production_tms_push_blocked = TRUE
            AND production_notification_blocked = TRUE
        )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_sandbox_tenant_env_org_key
    ON sandbox_tenant_environments (organization_id, environment_key);

CREATE TABLE IF NOT EXISTS sandbox_reset_jobs (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    sandbox_environment_id BIGINT NOT NULL REFERENCES sandbox_tenant_environments(id) ON DELETE CASCADE,
    requested_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    reset_reason TEXT NULL,
    job_status VARCHAR(32) NOT NULL DEFAULT 'queued',
    safety_checks JSONB NOT NULL DEFAULT '{}'::JSONB,
    result_summary TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP(6) NULL,
    CONSTRAINT chk_sandbox_reset_jobs_status
        CHECK (job_status IN ('queued', 'running', 'completed', 'failed', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_sandbox_reset_jobs_org_status
    ON sandbox_reset_jobs (organization_id, job_status, created_at DESC);

CREATE TABLE IF NOT EXISTS api_lifecycle_policies (
    id BIGSERIAL PRIMARY KEY,
    api_version VARCHAR(40) NOT NULL UNIQUE,
    release_status VARCHAR(32) NOT NULL DEFAULT 'active',
    released_on DATE NOT NULL,
    sunset_on DATE NULL,
    minimum_notice_days INTEGER NOT NULL DEFAULT 180,
    emergency_breaking_change_policy TEXT NOT NULL,
    changelog_url TEXT NOT NULL,
    postman_collection_url TEXT NULL,
    sdk_strategy TEXT NOT NULL,
    compatibility_test_status VARCHAR(32) NOT NULL DEFAULT 'required',
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_api_lifecycle_release_status
        CHECK (release_status IN ('active', 'deprecated', 'sunset', 'preview')),
    CONSTRAINT chk_api_lifecycle_compatibility_status
        CHECK (compatibility_test_status IN ('required', 'passing', 'failing', 'waived'))
);

CREATE TABLE IF NOT EXISTS api_partner_examples (
    id BIGSERIAL PRIMARY KEY,
    api_version VARCHAR(40) NOT NULL REFERENCES api_lifecycle_policies(api_version) ON DELETE CASCADE,
    example_key VARCHAR(100) NOT NULL,
    surface VARCHAR(80) NOT NULL,
    method VARCHAR(12) NOT NULL,
    path TEXT NOT NULL,
    sandbox_runnable BOOLEAN NOT NULL DEFAULT TRUE,
    request_example JSONB NOT NULL DEFAULT '{}'::JSONB,
    expected_response JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (api_version, example_key)
);

INSERT INTO api_lifecycle_policies (
    api_version,
    release_status,
    released_on,
    sunset_on,
    minimum_notice_days,
    emergency_breaking_change_policy,
    changelog_url,
    postman_collection_url,
    sdk_strategy,
    compatibility_test_status
)
VALUES (
    '2026-05-26',
    'active',
    DATE '2026-05-26',
    NULL,
    180,
    'Emergency breaking changes require incident approval, customer-impact notes, compatibility mitigation, and post-incident follow-up before permanent enforcement.',
    '/docs/API_LIFECYCLE_AND_SANDBOX.md',
    '/docs/STLOADS_POSTMAN_COLLECTION.json',
    'Ship OpenAPI-first generated clients and official Postman/sample payloads before hand-written SDKs; add typed SDK packages after two pilot customers stabilize the contract.',
    'passing'
)
ON CONFLICT (api_version)
DO UPDATE SET
    release_status = EXCLUDED.release_status,
    minimum_notice_days = EXCLUDED.minimum_notice_days,
    emergency_breaking_change_policy = EXCLUDED.emergency_breaking_change_policy,
    changelog_url = EXCLUDED.changelog_url,
    postman_collection_url = EXCLUDED.postman_collection_url,
    sdk_strategy = EXCLUDED.sdk_strategy,
    compatibility_test_status = EXCLUDED.compatibility_test_status,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO api_partner_examples (
    api_version,
    example_key,
    surface,
    method,
    path,
    sandbox_runnable,
    request_example,
    expected_response
)
VALUES
    ('2026-05-26', 'partner-load-api-post', 'loads', 'POST', '/dispatch/loads/api-post', TRUE,
     '{"idempotency_key":"demo-load-1001","external_ref":"SANDBOX-LOAD-1001"}'::JSONB,
     '{"success":true,"duplicate":false}'::JSONB),
    ('2026-05-26', 'tms-status-webhook', 'tms', 'POST', '/tms/webhooks/status', TRUE,
     '{"event_id":"sandbox-status-1001","tms_load_id":"TMS-SANDBOX-1001","status":"in_transit"}'::JSONB,
     '{"success":true}'::JSONB),
    ('2026-05-26', 'webhook-replay', 'webhooks', 'POST', '/admin/integrations/webhooks/{delivery_id}/replay', TRUE,
     '{"delivery_id":1001}'::JSONB,
     '{"success":true,"replay_delivery_id":1002}'::JSONB),
    ('2026-05-26', 'edi-204-validation', 'edi', 'POST', '/admin/integrations/edi/messages/validate', TRUE,
     '{"transaction_code":"204","direction":"inbound","control_number":"CTRL2041001"}'::JSONB,
     '{"success":true,"ack_status":"generated_997"}'::JSONB)
ON CONFLICT (api_version, example_key)
DO UPDATE SET
    surface = EXCLUDED.surface,
    method = EXCLUDED.method,
    path = EXCLUDED.path,
    sandbox_runnable = EXCLUDED.sandbox_runnable,
    request_example = EXCLUDED.request_example,
    expected_response = EXCLUDED.expected_response,
    updated_at = CURRENT_TIMESTAMP;
