-- Phase 12 notification center, preferences, provider decisions, and coverage catalog.

CREATE TABLE IF NOT EXISTS notification_events (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    recipient_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_key VARCHAR(120) NOT NULL,
    category VARCHAR(60) NOT NULL,
    priority VARCHAR(24) NOT NULL DEFAULT 'normal',
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    entity_type VARCHAR(80) NULL,
    entity_id BIGINT NULL,
    action_href TEXT NULL,
    channels TEXT[] NOT NULL DEFAULT ARRAY['in_app']::TEXT[],
    delivery_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    read_at TIMESTAMP(6) NULL,
    dismissed_at TIMESTAMP(6) NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_notification_events_priority
        CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
    CONSTRAINT chk_notification_events_status
        CHECK (delivery_status IN ('pending', 'delivered', 'failed', 'suppressed'))
);

CREATE INDEX IF NOT EXISTS idx_notification_events_user_unread
    ON notification_events (recipient_user_id, read_at, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notification_events_org_category
    ON notification_events (organization_id, category, created_at DESC);

CREATE TABLE IF NOT EXISTS notification_preferences (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id BIGINT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_key VARCHAR(120) NOT NULL DEFAULT '*',
    email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    in_app_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    sms_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    push_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    quiet_hours_start TIME NULL,
    quiet_hours_end TIME NULL,
    timezone VARCHAR(80) NOT NULL DEFAULT 'UTC',
    escalation_minutes INTEGER NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_notification_preferences_escalation
        CHECK (escalation_minutes IS NULL OR escalation_minutes >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_notification_preferences_scope_event
    ON notification_preferences (
        COALESCE(organization_id, 0),
        COALESCE(user_id, 0),
        event_key
    );

CREATE TABLE IF NOT EXISTS notification_provider_decisions (
    id BIGSERIAL PRIMARY KEY,
    channel VARCHAR(24) NOT NULL UNIQUE,
    provider_name VARCHAR(120) NOT NULL,
    decision_status VARCHAR(32) NOT NULL DEFAULT 'selected',
    opt_in_required BOOLEAN NOT NULL DEFAULT TRUE,
    opt_out_required BOOLEAN NOT NULL DEFAULT TRUE,
    quiet_hours_required BOOLEAN NOT NULL DEFAULT TRUE,
    emergency_exception_allowed BOOLEAN NOT NULL DEFAULT TRUE,
    provider_audit_logs_required BOOLEAN NOT NULL DEFAULT TRUE,
    compliance_notes TEXT NOT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_notification_provider_channel
        CHECK (channel IN ('email', 'in_app', 'sms', 'push')),
    CONSTRAINT chk_notification_provider_status
        CHECK (decision_status IN ('selected', 'deferred', 'disabled'))
);

CREATE TABLE IF NOT EXISTS notification_coverage_rules (
    id BIGSERIAL PRIMARY KEY,
    event_key VARCHAR(120) NOT NULL UNIQUE,
    category VARCHAR(60) NOT NULL,
    default_priority VARCHAR(24) NOT NULL DEFAULT 'normal',
    default_channels TEXT[] NOT NULL DEFAULT ARRAY['in_app']::TEXT[],
    responsible_party VARCHAR(80) NOT NULL,
    entity_type VARCHAR(80) NULL,
    escalation_minutes INTEGER NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_notification_coverage_priority
        CHECK (default_priority IN ('low', 'normal', 'high', 'urgent'))
);

INSERT INTO notification_provider_decisions (
    channel, provider_name, decision_status, opt_in_required, opt_out_required,
    quiet_hours_required, emergency_exception_allowed, provider_audit_logs_required, compliance_notes
)
VALUES
    ('in_app', 'STLoads notification center', 'selected', FALSE, FALSE, FALSE, TRUE, TRUE,
     'In-app notifications are the canonical operational fallback when email or future external channels are delayed.'),
    ('email', 'Configured SMTP sender', 'selected', TRUE, TRUE, TRUE, TRUE, TRUE,
     'Email remains supported through environment-specific SMTP sender identities and outbox retries.'),
    ('sms', 'Provider deferred', 'deferred', TRUE, TRUE, TRUE, TRUE, TRUE,
     'SMS requires explicit opt-in, opt-out, quiet hours, emergency exception policy, and provider audit logs before production enablement.'),
    ('push', 'Provider deferred', 'deferred', TRUE, TRUE, TRUE, TRUE, TRUE,
     'Push requires device-token governance, opt-out, quiet hours, emergency exception policy, and provider audit logs before production enablement.')
ON CONFLICT (channel)
DO UPDATE SET
    provider_name = EXCLUDED.provider_name,
    decision_status = EXCLUDED.decision_status,
    opt_in_required = EXCLUDED.opt_in_required,
    opt_out_required = EXCLUDED.opt_out_required,
    quiet_hours_required = EXCLUDED.quiet_hours_required,
    emergency_exception_allowed = EXCLUDED.emergency_exception_allowed,
    provider_audit_logs_required = EXCLUDED.provider_audit_logs_required,
    compliance_notes = EXCLUDED.compliance_notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO notification_coverage_rules (
    event_key, category, default_priority, default_channels, responsible_party,
    entity_type, escalation_minutes, notes
)
VALUES
    ('booking.created', 'booking', 'high', ARRAY['in_app','email']::TEXT[], 'shipper_carrier', 'load_leg', 30, 'Booking confirmation must reach both freight parties.'),
    ('offer.countered', 'marketplace', 'normal', ARRAY['in_app','email']::TEXT[], 'offer_owner', 'offer', 120, 'Counteroffers should not rely on chat visibility alone.'),
    ('tender.response_required', 'tender', 'high', ARRAY['in_app','email']::TEXT[], 'carrier', 'load_leg', 60, 'Tender action windows require escalation.'),
    ('tracking.stale', 'execution', 'high', ARRAY['in_app','email']::TEXT[], 'carrier_dispatch', 'load_leg', 45, 'Stale tracking needs operational follow-up.'),
    ('pickup.completed', 'execution', 'normal', ARRAY['in_app']::TEXT[], 'shipper', 'load_leg', NULL, 'Pickup events update customer and ops timelines.'),
    ('delivery.completed', 'execution', 'normal', ARRAY['in_app']::TEXT[], 'shipper', 'load_leg', NULL, 'Delivery events update customer and ops timelines.'),
    ('pod.missing', 'documents', 'urgent', ARRAY['in_app','email']::TEXT[], 'carrier', 'load_leg', 30, 'Missing POD blocks closeout and payment release.'),
    ('payment.hold', 'payments', 'urgent', ARRAY['in_app','email']::TEXT[], 'finance', 'payment', 30, 'Payment holds require accountable finance visibility.'),
    ('payment.released', 'payments', 'high', ARRAY['in_app','email']::TEXT[], 'carrier', 'payment', 60, 'Payout release should be observable by the carrier and finance.'),
    ('compliance.expiring', 'compliance', 'high', ARRAY['in_app','email']::TEXT[], 'carrier', 'compliance', 1440, 'Compliance expiry can block freight if ignored.'),
    ('tms.drift_detected', 'integrations', 'urgent', ARRAY['in_app','email']::TEXT[], 'operations', 'tms_handoff', 30, 'TMS drift must be visible outside the reconciliation page.'),
    ('document.rejected', 'documents', 'high', ARRAY['in_app','email']::TEXT[], 'document_owner', 'document', 120, 'Rejected documents need a direct correction path.')
ON CONFLICT (event_key)
DO UPDATE SET
    category = EXCLUDED.category,
    default_priority = EXCLUDED.default_priority,
    default_channels = EXCLUDED.default_channels,
    responsible_party = EXCLUDED.responsible_party,
    entity_type = EXCLUDED.entity_type,
    escalation_minutes = EXCLUDED.escalation_minutes,
    active = TRUE,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;
