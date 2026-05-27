CREATE TABLE IF NOT EXISTS legal_agreement_templates (
    id BIGSERIAL PRIMARY KEY,
    agreement_key TEXT NOT NULL,
    version TEXT NOT NULL,
    title TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    document_uri TEXT NULL,
    required_role_key TEXT NULL,
    requires_user_acceptance BOOLEAN NOT NULL DEFAULT TRUE,
    requires_organization_acceptance BOOLEAN NOT NULL DEFAULT FALSE,
    effective_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (agreement_key, version)
);

CREATE INDEX IF NOT EXISTS idx_legal_templates_active
    ON legal_agreement_templates (agreement_key, required_role_key, is_active, effective_at);

CREATE TABLE IF NOT EXISTS legal_agreement_acceptances (
    id BIGSERIAL PRIMARY KEY,
    template_id BIGINT NOT NULL REFERENCES legal_agreement_templates(id),
    agreement_key TEXT NOT NULL,
    version TEXT NOT NULL,
    user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE SET NULL,
    signer_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    signer_name TEXT NOT NULL,
    signer_email TEXT NOT NULL,
    ip_address TEXT NULL,
    user_agent TEXT NULL,
    accepted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    evidence_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    audit_event_id BIGINT NULL REFERENCES audit_events(id) ON DELETE SET NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (template_id, user_id),
    UNIQUE (template_id, organization_id)
);

CREATE INDEX IF NOT EXISTS idx_legal_acceptances_user
    ON legal_agreement_acceptances (user_id, agreement_key, version, accepted_at DESC);

CREATE INDEX IF NOT EXISTS idx_legal_acceptances_org
    ON legal_agreement_acceptances (organization_id, agreement_key, version, accepted_at DESC);

INSERT INTO legal_agreement_templates (
    agreement_key, version, title, content_sha256, document_uri,
    required_role_key, requires_user_acceptance, requires_organization_acceptance,
    effective_at, is_active
)
VALUES
    ('platform_terms', '2026-05-25', 'STLoads Platform Terms', 'pending-legal-content-sha256-platform-terms-2026-05-25', NULL, NULL, TRUE, FALSE, CURRENT_TIMESTAMP, TRUE),
    ('privacy_policy', '2026-05-25', 'STLoads Privacy Policy', 'pending-legal-content-sha256-privacy-policy-2026-05-25', NULL, NULL, TRUE, FALSE, CURRENT_TIMESTAMP, TRUE),
    ('tracking_consent', '2026-05-25', 'Location Tracking Consent', 'pending-legal-content-sha256-tracking-consent-2026-05-25', NULL, NULL, TRUE, FALSE, CURRENT_TIMESTAMP, TRUE),
    ('payment_terms', '2026-05-25', 'Payment Terms', 'pending-legal-content-sha256-payment-terms-2026-05-25', NULL, NULL, TRUE, FALSE, CURRENT_TIMESTAMP, TRUE),
    ('carrier_operating_agreement', '2026-05-25', 'Carrier Operating Agreement', 'pending-legal-content-sha256-carrier-operating-agreement-2026-05-25', NULL, 'carrier', TRUE, FALSE, CURRENT_TIMESTAMP, TRUE),
    ('broker_customer_contract', '2026-05-25', 'Broker And Customer Contract Terms', 'pending-legal-content-sha256-broker-customer-contract-2026-05-25', NULL, 'broker', TRUE, TRUE, CURRENT_TIMESTAMP, TRUE),
    ('broker_customer_contract', '2026-05-25-shipper', 'Shipper Customer Contract Terms', 'pending-legal-content-sha256-shipper-customer-contract-2026-05-25', NULL, 'shipper', TRUE, TRUE, CURRENT_TIMESTAMP, TRUE),
    ('freight_forwarder_terms', '2026-05-25', 'Freight Forwarder Terms', 'pending-legal-content-sha256-freight-forwarder-terms-2026-05-25', NULL, 'freight_forwarder', TRUE, TRUE, CURRENT_TIMESTAMP, TRUE)
ON CONFLICT (agreement_key, version) DO NOTHING;
