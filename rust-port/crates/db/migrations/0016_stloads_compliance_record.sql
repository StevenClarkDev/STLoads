ALTER TABLE stloads_handoffs
    ADD COLUMN IF NOT EXISTS paperwork_packet_id TEXT,
    ADD COLUMN IF NOT EXISTS document_packet_url TEXT,
    ADD COLUMN IF NOT EXISTS document_packet_hash TEXT,
    ADD COLUMN IF NOT EXISTS bol_number TEXT,
    ADD COLUMN IF NOT EXISTS freight_bill_number TEXT,
    ADD COLUMN IF NOT EXISTS atmp_operating_role TEXT,
    ADD COLUMN IF NOT EXISTS carrier_authority_snapshot JSONB,
    ADD COLUMN IF NOT EXISTS insurance_snapshot JSONB,
    ADD COLUMN IF NOT EXISTS compliance_blockers JSONB,
    ADD COLUMN IF NOT EXISTS retention_metadata JSONB,
    ADD COLUMN IF NOT EXISTS audit_event_ids JSONB,
    ADD COLUMN IF NOT EXISTS executive_override BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS executive_override_reason TEXT,
    ADD COLUMN IF NOT EXISTS executive_override_by TEXT,
    ADD COLUMN IF NOT EXISTS executive_override_at TEXT,
    ADD COLUMN IF NOT EXISTS customs_movement_type TEXT,
    ADD COLUMN IF NOT EXISTS customs_readiness TEXT,
    ADD COLUMN IF NOT EXISTS customs_documents_status JSONB,
    ADD COLUMN IF NOT EXISTS ace_entry_number TEXT,
    ADD COLUMN IF NOT EXISTS isf_status TEXT,
    ADD COLUMN IF NOT EXISTS in_bond_status TEXT,
    ADD COLUMN IF NOT EXISTS aes_itn TEXT,
    ADD COLUMN IF NOT EXISTS pga_requirements JSONB;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_paperwork_packet_id
    ON stloads_handoffs (paperwork_packet_id)
    WHERE paperwork_packet_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_bol_number
    ON stloads_handoffs (bol_number)
    WHERE bol_number IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_freight_bill_number
    ON stloads_handoffs (freight_bill_number)
    WHERE freight_bill_number IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_document_packet_hash
    ON stloads_handoffs (document_packet_hash)
    WHERE document_packet_hash IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_ace_entry_number
    ON stloads_handoffs (ace_entry_number)
    WHERE ace_entry_number IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_in_bond_status
    ON stloads_handoffs (in_bond_status)
    WHERE in_bond_status IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stloads_handoffs_customs_movement_type
    ON stloads_handoffs (customs_movement_type)
    WHERE customs_movement_type IS NOT NULL;
