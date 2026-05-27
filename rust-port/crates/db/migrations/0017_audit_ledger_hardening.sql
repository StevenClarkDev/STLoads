-- Make the audit ledger explicit, queryable, and append-only.

ALTER TABLE audit_events
    ADD COLUMN IF NOT EXISTS before_state JSONB NULL,
    ADD COLUMN IF NOT EXISTS after_state JSONB NULL;

CREATE INDEX IF NOT EXISTS idx_audit_events_request_id
    ON audit_events (request_id)
    WHERE request_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_events_created_at
    ON audit_events (created_at DESC);

CREATE OR REPLACE FUNCTION prevent_audit_events_mutation()
RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'audit_events is append-only';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_audit_events_prevent_update ON audit_events;
CREATE TRIGGER trg_audit_events_prevent_update
    BEFORE UPDATE ON audit_events
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_events_mutation();

DROP TRIGGER IF EXISTS trg_audit_events_prevent_delete ON audit_events;
CREATE TRIGGER trg_audit_events_prevent_delete
    BEFORE DELETE ON audit_events
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_events_mutation();
