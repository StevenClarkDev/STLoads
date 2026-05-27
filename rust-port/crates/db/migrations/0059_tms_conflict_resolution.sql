-- Phase 11 TMS conflict queue and field ownership policy.

CREATE TABLE IF NOT EXISTS tms_source_of_truth_rules (
    id BIGSERIAL PRIMARY KEY,
    field_key VARCHAR(120) NOT NULL UNIQUE,
    owning_system VARCHAR(32) NOT NULL,
    conflict_policy VARCHAR(64) NOT NULL,
    default_repair_action VARCHAR(64) NOT NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_tms_source_owner
        CHECK (owning_system IN ('stloads', 'tms', 'manual_review')),
    CONSTRAINT chk_tms_repair_action
        CHECK (default_repair_action IN ('requeue_tms_push', 'accept_tms_value', 'accept_stloads_value', 'manual_review'))
);

INSERT INTO tms_source_of_truth_rules (field_key, owning_system, conflict_policy, default_repair_action, notes)
VALUES
    ('load_status', 'manual_review', 'terminal status drift requires ops decision', 'manual_review', 'Cancellation, close, and delivered drift can affect customer visibility and payment release.'),
    ('rate', 'stloads', 'STLoads finance/rating controls own board and invoice rate', 'requeue_tms_push', 'Rate changes from TMS are treated as drift until STLoads reprices or requeues.'),
    ('pickup_window', 'tms', 'Upstream appointment system owns TMS pickup window after dispatch', 'accept_tms_value', 'Operational appointment drift should be reviewed and accepted or pushed back.'),
    ('delivery_window', 'tms', 'Upstream appointment system owns TMS delivery window after dispatch', 'accept_tms_value', 'Operational appointment drift should be reviewed and accepted or pushed back.'),
    ('equipment_type', 'stloads', 'STLoads load tender owns equipment after posting', 'requeue_tms_push', 'Equipment drift may invalidate carrier matching and tender rules.'),
    ('external_status', 'tms', 'TMS owns upstream execution event labels', 'accept_tms_value', 'Inbound webhook state is stored on the handoff but not blindly applied to every local lifecycle field.')
ON CONFLICT (field_key) DO UPDATE SET
    owning_system = EXCLUDED.owning_system,
    conflict_policy = EXCLUDED.conflict_policy,
    default_repair_action = EXCLUDED.default_repair_action,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

CREATE TABLE IF NOT EXISTS tms_conflict_queue (
    id BIGSERIAL PRIMARY KEY,
    handoff_id BIGINT NOT NULL REFERENCES stloads_handoffs(id) ON DELETE CASCADE,
    field_key VARCHAR(120) NOT NULL,
    source_of_truth VARCHAR(32) NOT NULL,
    stloads_value TEXT NULL,
    tms_value TEXT NULL,
    conflict_status VARCHAR(32) NOT NULL DEFAULT 'open',
    severity VARCHAR(32) NOT NULL DEFAULT 'medium',
    repair_action VARCHAR(64) NOT NULL,
    detected_by VARCHAR(120) NOT NULL DEFAULT 'reconciliation',
    resolution_note TEXT NULL,
    resolved_by VARCHAR(160) NULL,
    resolved_at TIMESTAMP(6) NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_tms_conflict_source
        CHECK (source_of_truth IN ('stloads', 'tms', 'manual_review')),
    CONSTRAINT chk_tms_conflict_status
        CHECK (conflict_status IN ('open', 'in_review', 'replay_queued', 'repaired', 'resolved', 'ignored')),
    CONSTRAINT chk_tms_conflict_severity
        CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT chk_tms_conflict_repair
        CHECK (repair_action IN ('requeue_tms_push', 'accept_tms_value', 'accept_stloads_value', 'manual_review'))
);

CREATE INDEX IF NOT EXISTS idx_tms_conflict_queue_status
    ON tms_conflict_queue (conflict_status, severity, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_tms_conflict_queue_handoff
    ON tms_conflict_queue (handoff_id, field_key, conflict_status);
