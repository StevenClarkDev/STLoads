CREATE TABLE IF NOT EXISTS compliance_status_records (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id BIGINT NULL REFERENCES users(id) ON DELETE CASCADE,
    subject_type TEXT NOT NULL,
    compliance_domain TEXT NOT NULL,
    status TEXT NOT NULL,
    eligibility_blocking BOOLEAN NOT NULL DEFAULT FALSE,
    evidence_reference TEXT NULL,
    reviewer_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    reason TEXT NULL,
    effective_at TIMESTAMP NULL,
    expires_at TIMESTAMP NULL,
    reviewed_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_compliance_status_subject_type CHECK (
        subject_type IN ('person', 'company', 'carrier', 'broker', 'freight_forwarder', 'tax', 'payout')
    ),
    CONSTRAINT chk_compliance_status_domain CHECK (
        compliance_domain IN (
            'person_kyc',
            'company_kyb',
            'carrier_compliance',
            'broker_compliance',
            'freight_forwarder_compliance',
            'tax_compliance',
            'payout_compliance'
        )
    ),
    CONSTRAINT chk_compliance_status_value CHECK (
        status IN ('not_required', 'pending', 'in_review', 'approved', 'rejected', 'expired', 'blocked')
    ),
    CONSTRAINT chk_compliance_status_subject_ref CHECK (
        user_id IS NOT NULL OR organization_id IS NOT NULL
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_compliance_status_records_subject_domain
    ON compliance_status_records (
        COALESCE(organization_id, 0),
        COALESCE(user_id, 0),
        subject_type,
        compliance_domain
    );

CREATE INDEX IF NOT EXISTS idx_compliance_status_records_user_blocking
    ON compliance_status_records (user_id, eligibility_blocking, status);

CREATE INDEX IF NOT EXISTS idx_compliance_status_records_expiry
    ON compliance_status_records (expires_at)
    WHERE expires_at IS NOT NULL;
