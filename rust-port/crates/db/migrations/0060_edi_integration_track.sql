-- Phase 11 EDI integration track.

CREATE TABLE IF NOT EXISTS edi_partner_profiles (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    partner_name VARCHAR(180) NOT NULL,
    isa_qualifier VARCHAR(8) NULL,
    isa_id VARCHAR(32) NULL,
    gs_id VARCHAR(32) NULL,
    transport_type VARCHAR(32) NOT NULL DEFAULT 'sftp',
    status VARCHAR(32) NOT NULL DEFAULT 'draft',
    supported_transactions TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    validation_mode VARCHAR(32) NOT NULL DEFAULT 'strict',
    created_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_edi_partner_profiles_status
        CHECK (status IN ('draft', 'active', 'paused', 'disabled')),
    CONSTRAINT chk_edi_partner_profiles_transport
        CHECK (transport_type IN ('sftp', 'as2', 'api', 'manual')),
    CONSTRAINT chk_edi_partner_profiles_validation
        CHECK (validation_mode IN ('strict', 'warn', 'sandbox'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_edi_partner_profiles_org_name
    ON edi_partner_profiles (organization_id, partner_name);

CREATE TABLE IF NOT EXISTS edi_transaction_mappings (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    transaction_code VARCHAR(16) NOT NULL,
    direction VARCHAR(16) NOT NULL,
    stloads_model VARCHAR(80) NOT NULL,
    mapping_version VARCHAR(32) NOT NULL DEFAULT 'v1',
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    required_fields TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_edi_transaction_mappings_direction
        CHECK (direction IN ('inbound', 'outbound', 'bidirectional')),
    CONSTRAINT chk_edi_transaction_mappings_status
        CHECK (status IN ('active', 'draft', 'disabled'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_edi_transaction_mappings_org_code_direction
    ON edi_transaction_mappings (COALESCE(organization_id, 0), transaction_code, direction);

CREATE TABLE IF NOT EXISTS edi_message_logs (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    partner_profile_id BIGINT NULL REFERENCES edi_partner_profiles(id) ON DELETE SET NULL,
    transaction_code VARCHAR(16) NOT NULL,
    direction VARCHAR(16) NOT NULL,
    control_number VARCHAR(80) NULL,
    business_key VARCHAR(160) NULL,
    message_status VARCHAR(32) NOT NULL DEFAULT 'received',
    ack_status VARCHAR(32) NOT NULL DEFAULT 'not_required',
    retry_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMP(6) NULL,
    error_summary TEXT NULL,
    payload_excerpt TEXT NULL,
    replay_of_message_id BIGINT NULL REFERENCES edi_message_logs(id) ON DELETE SET NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_edi_message_logs_direction
        CHECK (direction IN ('inbound', 'outbound')),
    CONSTRAINT chk_edi_message_logs_status
        CHECK (message_status IN ('received', 'validated', 'mapped', 'acked', 'failed', 'retrying', 'dead_letter', 'replay_queued')),
    CONSTRAINT chk_edi_message_logs_ack
        CHECK (ack_status IN ('not_required', 'pending_997', 'accepted_997', 'rejected_997', 'generated_997'))
);

CREATE INDEX IF NOT EXISTS idx_edi_message_logs_org_status
    ON edi_message_logs (organization_id, message_status, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS ux_edi_message_logs_org_control
    ON edi_message_logs (organization_id, transaction_code, direction, control_number)
    WHERE control_number IS NOT NULL;

INSERT INTO edi_transaction_mappings (
    organization_id,
    transaction_code,
    direction,
    stloads_model,
    mapping_version,
    status,
    required_fields,
    notes
)
VALUES
    (NULL, '204', 'inbound', 'load_tender', 'v1', 'active',
     ARRAY['shipper_reference', 'pickup_stop', 'delivery_stop', 'equipment', 'rate_or_terms']::TEXT[],
     'Inbound load tender creates or updates a governed STLoads load tender.'),
    (NULL, '990', 'outbound', 'tender_response', 'v1', 'active',
     ARRAY['tender_reference', 'accept_or_decline', 'carrier_identity']::TEXT[],
     'Outbound tender response communicates carrier acceptance or decline.'),
    (NULL, '214', 'bidirectional', 'shipment_status', 'v1', 'active',
     ARRAY['shipment_reference', 'status_code', 'status_timestamp', 'location']::TEXT[],
     'Shipment status maps to execution lifecycle and customer tracking events.'),
    (NULL, '210', 'inbound', 'freight_invoice', 'v1', 'active',
     ARRAY['invoice_number', 'shipment_reference', 'charges', 'currency']::TEXT[],
     'Freight invoice maps to invoice and settlement review before payment release.'),
    (NULL, '997', 'bidirectional', 'functional_acknowledgement', 'v1', 'active',
     ARRAY['ack_control_number', 'referenced_control_number', 'accepted_or_rejected']::TEXT[],
     'Functional acknowledgement is required for auditable EDI exchange outcomes.')
ON CONFLICT (COALESCE(organization_id, 0), transaction_code, direction)
DO UPDATE SET
    stloads_model = EXCLUDED.stloads_model,
    mapping_version = EXCLUDED.mapping_version,
    status = EXCLUDED.status,
    required_fields = EXCLUDED.required_fields,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;
