-- Governed write controls for required document rules.

ALTER TABLE required_document_rules
    ADD COLUMN IF NOT EXISTS description TEXT NULL,
    ADD COLUMN IF NOT EXISTS requires_approval BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    ADD COLUMN IF NOT EXISTS effective_to DATE NULL;

CREATE INDEX IF NOT EXISTS idx_required_document_rules_org_active
    ON required_document_rules (organization_id, is_active, effective_from DESC);
