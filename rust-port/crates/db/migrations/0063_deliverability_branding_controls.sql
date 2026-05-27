-- Phase 12 deliverability, communication compliance, and tenant branding controls.

CREATE TABLE IF NOT EXISTS message_sender_identities (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    environment_key VARCHAR(32) NOT NULL DEFAULT 'production',
    sender_domain VARCHAR(255) NOT NULL,
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(160) NOT NULL,
    spf_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    dkim_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    dmarc_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    identity_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    verified_at TIMESTAMP(6) NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_message_sender_status
        CHECK (
            spf_status IN ('pending', 'verified', 'failed', 'deferred')
            AND dkim_status IN ('pending', 'verified', 'failed', 'deferred')
            AND dmarc_status IN ('pending', 'verified', 'failed', 'deferred')
            AND identity_status IN ('pending', 'verified', 'failed', 'disabled')
        )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_message_sender_identity_scope
    ON message_sender_identities (
        COALESCE(organization_id, 0),
        environment_key,
        from_email
    );

CREATE TABLE IF NOT EXISTS message_delivery_events (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email_outbox_id BIGINT NULL REFERENCES email_outbox(id) ON DELETE SET NULL,
    channel VARCHAR(24) NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    provider_message_id VARCHAR(255) NULL,
    recipient VARCHAR(255) NOT NULL,
    reason TEXT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    occurred_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_message_delivery_channel
        CHECK (channel IN ('email', 'in_app', 'sms', 'push')),
    CONSTRAINT chk_message_delivery_event_type
        CHECK (event_type IN ('queued', 'sent', 'delivered', 'bounce', 'complaint', 'suppressed', 'failed', 'retry', 'test_send'))
);

CREATE INDEX IF NOT EXISTS idx_message_delivery_events_org_time
    ON message_delivery_events (organization_id, occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_message_delivery_events_outbox
    ON message_delivery_events (email_outbox_id, occurred_at DESC);

CREATE TABLE IF NOT EXISTS message_suppression_entries (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    channel VARCHAR(24) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    suppression_reason VARCHAR(80) NOT NULL,
    source_event_id BIGINT NULL REFERENCES message_delivery_events(id) ON DELETE SET NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    expires_at TIMESTAMP(6) NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_message_suppression_channel
        CHECK (channel IN ('email', 'sms', 'push')),
    CONSTRAINT chk_message_suppression_status
        CHECK (status IN ('active', 'expired', 'removed'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_message_suppression_scope
    ON message_suppression_entries (
        COALESCE(organization_id, 0),
        channel,
        lower(recipient),
        status
    );

CREATE TABLE IF NOT EXISTS message_template_governance (
    id BIGSERIAL PRIMARY KEY,
    template_key VARCHAR(120) NOT NULL,
    channel VARCHAR(24) NOT NULL,
    locale VARCHAR(16) NOT NULL DEFAULT 'en-US',
    version INTEGER NOT NULL DEFAULT 1,
    owner_team VARCHAR(80) NOT NULL,
    approval_status VARCHAR(32) NOT NULL DEFAULT 'draft',
    approved_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMP(6) NULL,
    high_risk BOOLEAN NOT NULL DEFAULT FALSE,
    test_send_required BOOLEAN NOT NULL DEFAULT TRUE,
    body_hash VARCHAR(128) NOT NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_message_template_channel
        CHECK (channel IN ('email', 'sms', 'push', 'in_app')),
    CONSTRAINT chk_message_template_approval
        CHECK (approval_status IN ('draft', 'pending_approval', 'approved', 'retired', 'rejected'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_message_template_governance_version
    ON message_template_governance (template_key, channel, locale, version);

CREATE TABLE IF NOT EXISTS message_monitoring_rules (
    id BIGSERIAL PRIMARY KEY,
    rule_key VARCHAR(120) NOT NULL UNIQUE,
    event_key VARCHAR(120) NULL,
    template_key VARCHAR(120) NULL,
    category VARCHAR(60) NOT NULL,
    priority VARCHAR(24) NOT NULL DEFAULT 'high',
    required_sender_identity BOOLEAN NOT NULL DEFAULT TRUE,
    fallback_channel VARCHAR(24) NOT NULL DEFAULT 'in_app',
    escalation_minutes INTEGER NOT NULL DEFAULT 30,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_message_monitoring_priority
        CHECK (priority IN ('normal', 'high', 'urgent')),
    CONSTRAINT chk_message_monitoring_fallback
        CHECK (fallback_channel IN ('in_app', 'email', 'sms', 'push')),
    CONSTRAINT chk_message_monitoring_escalation
        CHECK (escalation_minutes >= 0)
);

CREATE TABLE IF NOT EXISTS tenant_branding_policies (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    portal_branding_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    document_branding_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    email_branding_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    custom_domain_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    white_label_status VARCHAR(32) NOT NULL DEFAULT 'deferred',
    unsupported_message TEXT NOT NULL DEFAULT 'White-label and custom-domain support require product approval before sale or implementation.',
    fallback_brand_name VARCHAR(160) NOT NULL DEFAULT 'STLoads',
    cache_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_tenant_branding_status
        CHECK (white_label_status IN ('supported', 'deferred', 'disabled'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_tenant_branding_policy_org
    ON tenant_branding_policies (organization_id);

CREATE TABLE IF NOT EXISTS tenant_brand_assets (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    asset_type VARCHAR(40) NOT NULL,
    asset_url TEXT NOT NULL,
    mime_type VARCHAR(80) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    width_px INTEGER NULL,
    height_px INTEGER NULL,
    review_status VARCHAR(32) NOT NULL DEFAULT 'pending_review',
    reviewer_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    cache_key VARCHAR(160) NOT NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_tenant_brand_asset_type
        CHECK (asset_type IN ('logo', 'email_header', 'document_logo')),
    CONSTRAINT chk_tenant_brand_asset_mime
        CHECK (mime_type IN ('image/png', 'image/jpeg', 'image/webp', 'image/svg+xml')),
    CONSTRAINT chk_tenant_brand_asset_size
        CHECK (file_size_bytes > 0 AND file_size_bytes <= 2097152),
    CONSTRAINT chk_tenant_brand_asset_review
        CHECK (review_status IN ('pending_review', 'approved', 'rejected', 'retired'))
);

CREATE INDEX IF NOT EXISTS idx_tenant_brand_assets_org_status
    ON tenant_brand_assets (organization_id, review_status, asset_type);

CREATE TABLE IF NOT EXISTS tenant_custom_domains (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL,
    purpose VARCHAR(32) NOT NULL DEFAULT 'portal',
    verification_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    dns_txt_name VARCHAR(255) NOT NULL,
    dns_txt_value VARCHAR(255) NOT NULL,
    tls_status VARCHAR(32) NOT NULL DEFAULT 'not_requested',
    rollback_status VARCHAR(32) NOT NULL DEFAULT 'ready',
    ownership_checked_at TIMESTAMP(6) NULL,
    last_checked_at TIMESTAMP(6) NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_tenant_custom_domain_purpose
        CHECK (purpose IN ('portal', 'tracking', 'api')),
    CONSTRAINT chk_tenant_custom_domain_verification
        CHECK (verification_status IN ('pending', 'verified', 'failed', 'disabled')),
    CONSTRAINT chk_tenant_custom_domain_tls
        CHECK (tls_status IN ('not_requested', 'pending', 'issued', 'failed', 'revoked')),
    CONSTRAINT chk_tenant_custom_domain_rollback
        CHECK (rollback_status IN ('ready', 'required', 'completed'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_tenant_custom_domains_scope
    ON tenant_custom_domains (organization_id, lower(domain), purpose);

CREATE TABLE IF NOT EXISTS tenant_branded_template_rules (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    template_key VARCHAR(120) NOT NULL,
    template_surface VARCHAR(64) NOT NULL,
    branding_status VARCHAR(32) NOT NULL DEFAULT 'fallback',
    fallback_allowed BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_tenant_branded_template_surface
        CHECK (template_surface IN ('rate_confirmation', 'bol', 'pod_package', 'invoice', 'settlement_packet', 'notification_email')),
    CONSTRAINT chk_tenant_branded_template_status
        CHECK (branding_status IN ('branded', 'fallback', 'unsupported', 'pending_review'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_tenant_branded_template_rules_scope
    ON tenant_branded_template_rules (organization_id, template_key, template_surface);

INSERT INTO message_sender_identities (
    organization_id, environment_key, sender_domain, from_email, from_name,
    spf_status, dkim_status, dmarc_status, identity_status, verified_at, notes
)
VALUES
    (NULL, 'development', 'localhost.stloads.test', 'no-reply@localhost.stloads.test', 'STLoads Development',
     'deferred', 'deferred', 'deferred', 'pending', NULL, 'Development sender is not allowed for production customer messaging.'),
    (NULL, 'production', 'stloads.com', 'no-reply@stloads.com', 'STLoads',
     'pending', 'pending', 'pending', 'pending', NULL, 'Production sender must pass SPF, DKIM, and DMARC before high-risk email sending.')
ON CONFLICT (COALESCE(organization_id, 0), environment_key, from_email)
DO UPDATE SET
    sender_domain = EXCLUDED.sender_domain,
    from_name = EXCLUDED.from_name,
    spf_status = EXCLUDED.spf_status,
    dkim_status = EXCLUDED.dkim_status,
    dmarc_status = EXCLUDED.dmarc_status,
    identity_status = EXCLUDED.identity_status,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO message_template_governance (
    template_key, channel, locale, version, owner_team, approval_status,
    high_risk, test_send_required, body_hash, notes
)
VALUES
    ('otp.login', 'email', 'en-US', 1, 'security', 'pending_approval', TRUE, TRUE, 'seed-otp-login-v1', 'OTP messaging must be test-sent and approved before production use.'),
    ('password.reset', 'email', 'en-US', 1, 'security', 'pending_approval', TRUE, TRUE, 'seed-password-reset-v1', 'Password reset messaging must preserve anti-phishing language.'),
    ('tender.response_required', 'email', 'en-US', 1, 'operations', 'pending_approval', TRUE, TRUE, 'seed-tender-response-v1', 'Tender messages need deadline and fallback wording.'),
    ('pod.rejected', 'email', 'en-US', 1, 'documents', 'pending_approval', TRUE, TRUE, 'seed-pod-rejected-v1', 'POD rejection messages must explain correction steps.'),
    ('payment.hold', 'email', 'en-US', 1, 'finance', 'pending_approval', TRUE, TRUE, 'seed-payment-hold-v1', 'Payment hold messages need finance ownership and escalation path.'),
    ('payout.released', 'email', 'en-US', 1, 'finance', 'pending_approval', TRUE, TRUE, 'seed-payout-release-v1', 'Payout release messages are high-risk financial communication.')
ON CONFLICT (template_key, channel, locale, version)
DO UPDATE SET
    owner_team = EXCLUDED.owner_team,
    approval_status = EXCLUDED.approval_status,
    high_risk = EXCLUDED.high_risk,
    test_send_required = EXCLUDED.test_send_required,
    body_hash = EXCLUDED.body_hash,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO message_monitoring_rules (
    rule_key, event_key, template_key, category, priority,
    required_sender_identity, fallback_channel, escalation_minutes, notes
)
VALUES
    ('otp.delivery', NULL, 'otp.login', 'security', 'urgent', TRUE, 'in_app', 5, 'OTP failures must surface immediately.'),
    ('password_reset.delivery', NULL, 'password.reset', 'security', 'urgent', TRUE, 'in_app', 10, 'Password reset delivery failures require a safe fallback.'),
    ('tender.delivery', 'tender.response_required', 'tender.response_required', 'operations', 'high', TRUE, 'in_app', 30, 'Tender deadlines cannot rely on one channel.'),
    ('pickup.delivery', 'pickup.completed', NULL, 'execution', 'high', FALSE, 'in_app', 45, 'Pickup completion should remain visible even if email is disabled.'),
    ('delivery.delivery', 'delivery.completed', NULL, 'execution', 'high', FALSE, 'in_app', 45, 'Delivery completion should remain visible even if email is disabled.'),
    ('pod_rejection.delivery', 'document.rejected', 'pod.rejected', 'documents', 'urgent', TRUE, 'in_app', 30, 'Rejected POD blocks financial closeout.'),
    ('payment_hold.delivery', 'payment.hold', 'payment.hold', 'finance', 'urgent', TRUE, 'in_app', 30, 'Payment holds need observable escalation.'),
    ('payout_release.delivery', 'payment.released', 'payout.released', 'finance', 'high', TRUE, 'in_app', 60, 'Payout release failures need finance visibility.')
ON CONFLICT (rule_key)
DO UPDATE SET
    event_key = EXCLUDED.event_key,
    template_key = EXCLUDED.template_key,
    category = EXCLUDED.category,
    priority = EXCLUDED.priority,
    required_sender_identity = EXCLUDED.required_sender_identity,
    fallback_channel = EXCLUDED.fallback_channel,
    escalation_minutes = EXCLUDED.escalation_minutes,
    active = TRUE,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;
