CREATE TABLE IF NOT EXISTS required_document_rules (
    id BIGSERIAL PRIMARY KEY,
    rule_key TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    requirement_scope TEXT NOT NULL,
    role_key TEXT NULL,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    equipment_key TEXT NULL,
    commodity_key TEXT NULL,
    load_type_key TEXT NULL,
    customer_key TEXT NULL,
    lifecycle_state TEXT NOT NULL,
    document_type_key TEXT NOT NULL,
    blocks_transition BOOLEAN NOT NULL DEFAULT TRUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_required_document_rules_lookup
    ON required_document_rules (requirement_scope, lifecycle_state, is_active);

INSERT INTO required_document_rules (
    rule_key, label, requirement_scope, role_key, lifecycle_state, document_type_key, blocks_transition
) VALUES
    ('carrier_operating_authority', 'Operating authority', 'onboarding', 'carrier', 'submit_onboarding', 'operating_authority', TRUE),
    ('carrier_insurance_certificate', 'Insurance certificate', 'onboarding', 'carrier', 'submit_onboarding', 'insurance_certificate', TRUE),
    ('carrier_w9', 'W-9 tax form', 'onboarding', 'carrier', 'submit_onboarding', 'w9', TRUE),
    ('broker_operating_authority', 'Broker operating authority', 'onboarding', 'broker', 'submit_onboarding', 'broker_authority', TRUE),
    ('load_rate_confirmation', 'Rate confirmation', 'load', NULL, 'booking', 'rate_confirmation', TRUE),
    ('execution_delivery_pod', 'Delivery POD', 'execution', NULL, 'complete_delivery', 'delivery_pod', TRUE)
ON CONFLICT (rule_key) DO NOTHING;
