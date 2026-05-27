CREATE TABLE IF NOT EXISTS driver_equipment_safety_compliance (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    carrier_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    driver_compliance_status TEXT NOT NULL DEFAULT 'pending',
    cdl_expires_at DATE NULL,
    medical_card_expires_at DATE NULL,
    mvr_status TEXT NOT NULL DEFAULT 'pending',
    background_check_status TEXT NOT NULL DEFAULT 'pending',
    endorsements TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    equipment_compliance_status TEXT NOT NULL DEFAULT 'pending',
    truck_unit_identifier TEXT NULL,
    trailer_unit_identifier TEXT NULL,
    vin TEXT NULL,
    ownership_status TEXT NULL,
    inspection_expires_at DATE NULL,
    maintenance_status TEXT NOT NULL DEFAULT 'pending',
    equipment_insurance_status TEXT NOT NULL DEFAULT 'pending',
    safety_rating TEXT NULL,
    csa_alert_level TEXT NOT NULL DEFAULT 'unknown',
    hazmat_eligible BOOLEAN NOT NULL DEFAULT FALSE,
    temperature_control_eligible BOOLEAN NOT NULL DEFAULT FALSE,
    restricted_freight_blocking BOOLEAN NOT NULL DEFAULT FALSE,
    dvir_policy TEXT NOT NULL DEFAULT 'deferred_first_release',
    notes TEXT NULL,
    reviewed_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_driver_compliance_status CHECK (driver_compliance_status IN ('pending', 'eligible', 'expired', 'blocked', 'manual_review')),
    CONSTRAINT chk_equipment_compliance_status CHECK (equipment_compliance_status IN ('pending', 'eligible', 'expired', 'blocked', 'manual_review')),
    CONSTRAINT chk_mvr_status CHECK (mvr_status IN ('pending', 'clear', 'review', 'rejected')),
    CONSTRAINT chk_background_check_status CHECK (background_check_status IN ('pending', 'clear', 'review', 'rejected')),
    CONSTRAINT chk_maintenance_status CHECK (maintenance_status IN ('pending', 'current', 'due', 'overdue', 'blocked')),
    CONSTRAINT chk_equipment_insurance_status CHECK (equipment_insurance_status IN ('pending', 'verified', 'expired', 'rejected', 'missing')),
    CONSTRAINT chk_csa_alert_level CHECK (csa_alert_level IN ('unknown', 'normal', 'watch', 'alert', 'critical'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_driver_equipment_safety_compliance_carrier
    ON driver_equipment_safety_compliance (carrier_user_id);

CREATE INDEX IF NOT EXISTS idx_driver_equipment_safety_compliance_blocking
    ON driver_equipment_safety_compliance (carrier_user_id, restricted_freight_blocking, driver_compliance_status, equipment_compliance_status);

CREATE TABLE IF NOT EXISTS sanctions_tax_profiles (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sanctions_status TEXT NOT NULL DEFAULT 'pending',
    ofac_screened_at TIMESTAMP NULL,
    sanctions_provider TEXT NOT NULL DEFAULT 'manual',
    sanctions_reference TEXT NULL,
    beneficial_owner_status TEXT NOT NULL DEFAULT 'pending',
    tax_document_status TEXT NOT NULL DEFAULT 'pending',
    tax_document_type TEXT NULL,
    tin_masked TEXT NULL,
    tax_reporting_owner TEXT NOT NULL DEFAULT 'finance',
    tax_year INTEGER NULL,
    payout_tax_blocking BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT NULL,
    reviewed_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_sanctions_status CHECK (sanctions_status IN ('pending', 'clear', 'possible_match', 'blocked', 'not_required')),
    CONSTRAINT chk_beneficial_owner_status CHECK (beneficial_owner_status IN ('pending', 'clear', 'review', 'blocked', 'not_required')),
    CONSTRAINT chk_tax_document_status CHECK (tax_document_status IN ('pending', 'received', 'verified', 'rejected', 'not_required'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_sanctions_tax_profiles_user
    ON sanctions_tax_profiles (user_id);

CREATE INDEX IF NOT EXISTS idx_sanctions_tax_profiles_blocking
    ON sanctions_tax_profiles (user_id, sanctions_status, tax_document_status, payout_tax_blocking);

CREATE TABLE IF NOT EXISTS risk_review_items (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    subject_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    load_id BIGINT NULL REFERENCES loads(id) ON DELETE SET NULL,
    leg_id BIGINT NULL REFERENCES load_legs(id) ON DELETE SET NULL,
    review_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    status TEXT NOT NULL DEFAULT 'open',
    score INTEGER NOT NULL DEFAULT 0,
    reasons TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    hold_booking BOOLEAN NOT NULL DEFAULT FALSE,
    hold_payout BOOLEAN NOT NULL DEFAULT FALSE,
    communication_required BOOLEAN NOT NULL DEFAULT FALSE,
    provider_notification_required BOOLEAN NOT NULL DEFAULT FALSE,
    reviewer_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    decision_note TEXT NULL,
    decided_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_risk_review_type CHECK (
        review_type IN ('risk_score', 'double_brokering', 'carrier_fraud', 'aml_transaction', 'account_takeover', 'sanctions', 'tax')
    ),
    CONSTRAINT chk_risk_review_severity CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT chk_risk_review_status CHECK (status IN ('open', 'in_review', 'approved', 'rejected', 'blocked', 'resolved', 'expired'))
);

CREATE INDEX IF NOT EXISTS idx_risk_review_items_booking_holds
    ON risk_review_items (subject_user_id, leg_id, hold_booking, status);

CREATE INDEX IF NOT EXISTS idx_risk_review_items_payout_holds
    ON risk_review_items (subject_user_id, leg_id, hold_payout, status);

CREATE INDEX IF NOT EXISTS idx_risk_review_items_type_status
    ON risk_review_items (review_type, severity, status, created_at DESC);

CREATE TABLE IF NOT EXISTS aml_operating_model_decisions (
    id BIGSERIAL PRIMARY KEY,
    decision_key TEXT NOT NULL UNIQUE,
    obligation_status TEXT NOT NULL,
    owner TEXT NOT NULL,
    decision TEXT NOT NULL,
    review_due_at DATE NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_aml_obligation_status CHECK (obligation_status IN ('implemented', 'provider_owned', 'ruled_out_current_model', 'legal_review_required'))
);

INSERT INTO aml_operating_model_decisions (
    decision_key, obligation_status, owner, decision, review_due_at
) VALUES
    ('money_transmission', 'provider_owned', 'Finance / Legal', 'Stripe/payment provider remains the money movement provider for the first release; STLoads must not market itself as a bank, money transmitter, payment facilitator, or escrow agent.', CURRENT_DATE + INTERVAL '90 days'),
    ('suspicious_activity_reporting', 'legal_review_required', 'Risk / Legal', 'Risk review items capture suspicious activity evidence and provider-notification flags; formal SAR or law-enforcement reporting duties require Legal review before launch.', CURRENT_DATE + INTERVAL '90 days'),
    ('carrier_payout_tax_reporting', 'implemented', 'Finance', 'Tax document status and tax reporting owner are tracked before payout eligibility; annual reporting workflow ownership is assigned to Finance.', CURRENT_DATE + INTERVAL '90 days')
ON CONFLICT (decision_key) DO UPDATE SET
    obligation_status = EXCLUDED.obligation_status,
    owner = EXCLUDED.owner,
    decision = EXCLUDED.decision,
    review_due_at = EXCLUDED.review_due_at,
    updated_at = CURRENT_TIMESTAMP;
