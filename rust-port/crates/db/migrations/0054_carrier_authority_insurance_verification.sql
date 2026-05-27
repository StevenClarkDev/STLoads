CREATE TABLE IF NOT EXISTS carrier_authority_verifications (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    carrier_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dot_number TEXT NULL,
    mc_number TEXT NULL,
    legal_name TEXT NULL,
    authority_status TEXT NOT NULL DEFAULT 'pending',
    operating_authority_type TEXT NULL,
    safety_rating TEXT NULL,
    insurance_status TEXT NOT NULL DEFAULT 'pending',
    insurance_provider TEXT NULL,
    insurance_policy_number TEXT NULL,
    cargo_coverage_amount_cents BIGINT NULL,
    liability_coverage_amount_cents BIGINT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    insurance_effective_at DATE NULL,
    insurance_expires_at DATE NULL,
    verification_source TEXT NOT NULL DEFAULT 'manual',
    verified_at TIMESTAMP NULL,
    reviewed_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    notes TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_carrier_authority_status CHECK (
        authority_status IN ('pending', 'active', 'inactive', 'revoked', 'not_found', 'manual_review')
    ),
    CONSTRAINT chk_carrier_insurance_status CHECK (
        insurance_status IN ('pending', 'verified', 'expired', 'rejected', 'missing', 'manual_review')
    ),
    CONSTRAINT chk_carrier_authority_currency CHECK (currency IN ('USD', 'CAD', 'MXN'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_carrier_authority_verifications_carrier
    ON carrier_authority_verifications (carrier_user_id);

CREATE INDEX IF NOT EXISTS idx_carrier_authority_verifications_blocking
    ON carrier_authority_verifications (carrier_user_id, authority_status, insurance_status, insurance_expires_at);

CREATE INDEX IF NOT EXISTS idx_carrier_authority_verifications_expiry
    ON carrier_authority_verifications (insurance_expires_at)
    WHERE insurance_expires_at IS NOT NULL;
