-- Phase 14 pricing intelligence, global search, and data quality monitoring.

CREATE TABLE IF NOT EXISTS lane_pricing_history (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    lane_key VARCHAR(160) NOT NULL,
    origin_label VARCHAR(160) NOT NULL,
    destination_label VARCHAR(160) NOT NULL,
    equipment_type VARCHAR(80) NULL,
    freight_mode VARCHAR(40) NOT NULL DEFAULT 'truckload',
    observed_rate_cents BIGINT NOT NULL,
    observed_currency VARCHAR(8) NOT NULL DEFAULT 'USD',
    source_type VARCHAR(40) NOT NULL,
    source_entity_id BIGINT NULL,
    pickup_date DATE NULL,
    delivered_at TIMESTAMP(6) NULL,
    margin_cents BIGINT NULL,
    on_time_delivery BOOLEAN NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_lane_pricing_source
        CHECK (source_type IN ('posted_load', 'booked_load', 'contract_lane', 'invoice', 'settlement', 'manual_reference'))
);

CREATE INDEX IF NOT EXISTS idx_lane_pricing_history_lookup
    ON lane_pricing_history (organization_id, lane_key, equipment_type, pickup_date DESC NULLS LAST);

CREATE TABLE IF NOT EXISTS lane_pricing_recommendations (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    lane_key VARCHAR(160) NOT NULL,
    equipment_type VARCHAR(80) NULL,
    recommended_rate_cents BIGINT NOT NULL,
    low_rate_cents BIGINT NULL,
    high_rate_cents BIGINT NULL,
    currency VARCHAR(8) NOT NULL DEFAULT 'USD',
    confidence_score NUMERIC(6,2) NOT NULL DEFAULT 0,
    sample_size INTEGER NOT NULL DEFAULT 0,
    anomaly_status VARCHAR(32) NOT NULL DEFAULT 'normal',
    recommendation_reason TEXT NOT NULL,
    valid_from DATE NOT NULL DEFAULT CURRENT_DATE,
    valid_until DATE NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_lane_pricing_anomaly
        CHECK (anomaly_status IN ('normal', 'low_sample', 'rate_spike', 'rate_drop', 'outlier', 'review_required'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_lane_pricing_recommendation_current
    ON lane_pricing_recommendations (
        organization_id,
        lane_key,
        COALESCE(equipment_type, ''),
        valid_from
    );

CREATE TABLE IF NOT EXISTS global_search_documents (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    entity_type VARCHAR(80) NOT NULL,
    entity_id VARCHAR(120) NOT NULL,
    title VARCHAR(255) NOT NULL,
    subtitle TEXT NULL,
    searchable_text TEXT NOT NULL,
    href TEXT NULL,
    permission_key VARCHAR(120) NULL,
    last_indexed_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_global_search_entity
        CHECK (entity_type IN ('load', 'load_leg', 'user', 'organization', 'document', 'invoice', 'payment', 'conversation', 'tms_handoff', 'support_case'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_global_search_document_entity
    ON global_search_documents (organization_id, entity_type, entity_id);

CREATE INDEX IF NOT EXISTS idx_global_search_documents_text
    ON global_search_documents USING GIN (to_tsvector('simple', searchable_text));

CREATE TABLE IF NOT EXISTS data_quality_rules (
    id BIGSERIAL PRIMARY KEY,
    rule_key VARCHAR(160) NOT NULL UNIQUE,
    category VARCHAR(80) NOT NULL,
    severity VARCHAR(24) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    check_sql TEXT NOT NULL,
    repair_playbook TEXT NOT NULL,
    cadence VARCHAR(80) NOT NULL DEFAULT 'daily',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    alert_threshold INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_data_quality_rule_severity
        CHECK (severity IN ('info', 'warning', 'error', 'critical'))
);

CREATE TABLE IF NOT EXISTS data_quality_runs (
    id BIGSERIAL PRIMARY KEY,
    run_status VARCHAR(32) NOT NULL DEFAULT 'running',
    started_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMP(6) NULL,
    checked_rule_count INTEGER NOT NULL DEFAULT 0,
    finding_count INTEGER NOT NULL DEFAULT 0,
    triggered_by VARCHAR(80) NOT NULL DEFAULT 'scheduled',
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_data_quality_run_status
        CHECK (run_status IN ('running', 'success', 'failed', 'cancelled'))
);

CREATE TABLE IF NOT EXISTS data_quality_findings (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    rule_id BIGINT NOT NULL REFERENCES data_quality_rules(id) ON DELETE CASCADE,
    run_id BIGINT NULL REFERENCES data_quality_runs(id) ON DELETE SET NULL,
    entity_type VARCHAR(80) NOT NULL,
    entity_id VARCHAR(120) NULL,
    severity VARCHAR(24) NOT NULL,
    finding_status VARCHAR(32) NOT NULL DEFAULT 'open',
    owner_team VARCHAR(80) NOT NULL,
    title VARCHAR(255) NOT NULL,
    detail TEXT NOT NULL,
    repair_action TEXT NULL,
    detected_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP(6) NULL,
    audit_event_id BIGINT NULL REFERENCES audit_events(id) ON DELETE SET NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_data_quality_finding_severity
        CHECK (severity IN ('info', 'warning', 'error', 'critical')),
    CONSTRAINT chk_data_quality_finding_status
        CHECK (finding_status IN ('open', 'assigned', 'in_repair', 'resolved', 'suppressed'))
);

CREATE INDEX IF NOT EXISTS idx_data_quality_findings_open
    ON data_quality_findings (organization_id, finding_status, severity, detected_at DESC);

INSERT INTO data_quality_rules (
    rule_key, category, severity, owner_team, check_sql, repair_playbook, cadence, alert_threshold
)
VALUES
    ('orphan_load_legs', 'referential_integrity', 'critical', 'Data/Backend', 'Find load_legs without a parent load.', 'Restore parent load or quarantine the orphan leg before reporting refresh.', 'hourly', 1),
    ('invalid_state_combinations', 'workflow_state', 'critical', 'Data/Ops', 'Find loads, legs, offers, escrow, documents, or TMS handoffs with impossible lifecycle combinations.', 'Route to owning workflow team and repair through audited state transition.', 'hourly', 1),
    ('duplicate_external_references', 'integrations', 'error', 'Integrations', 'Find duplicate external IDs across TMS, EDI, webhooks, and partner APIs.', 'Merge duplicate references or create a conflict record with source-of-truth decision.', 'hourly', 1),
    ('missing_required_documents', 'documents', 'error', 'Ops/Documents', 'Find delivered loads missing required POD, BOL, invoice, or compliance documents.', 'Assign document owner and block release until required documents are attached or waived.', 'daily', 1),
    ('stale_tms_handoffs', 'integrations', 'warning', 'Integrations', 'Find queued or retrying TMS handoffs older than SLA.', 'Replay, repair conflict, or mark dead-letter with customer-facing status.', 'hourly', 5),
    ('unmatched_payments', 'finance', 'error', 'Finance', 'Find payouts, invoices, settlements, or ledger entries that do not reconcile to an owned load leg.', 'Hold release and reconcile ledger or settlement source.', 'daily', 1),
    ('inconsistent_tenant_ownership', 'tenant_isolation', 'critical', 'Security/Backend', 'Find records whose organization ownership does not match parent entities.', 'Escalate as security incident if cross-tenant access may be possible.', 'hourly', 1),
    ('lane_rate_anomaly', 'pricing', 'warning', 'Data/Product', 'Find lane recommendations with rate spikes, drops, outliers, or low sample size.', 'Review pricing recommendation before customer-facing publication.', 'daily', 3),
    ('carrier_score_change_anomaly', 'scorecards', 'warning', 'Data/Ops', 'Find sudden carrier score changes that exceed configured thresholds.', 'Review recent claims, tracking, document quality, and payout signals.', 'daily', 3),
    ('suspicious_tracking_pattern', 'tracking', 'warning', 'Ops/Security', 'Find impossible GPS jumps, stale pings, or repeated identical coordinates.', 'Ask carrier for confirmation and inspect tracking consent/source.', 'hourly', 3),
    ('unusual_document_replacement', 'documents', 'warning', 'Ops/Documents', 'Find excessive document replacement or replacement after closeout.', 'Review document version history and approval trail.', 'daily', 3),
    ('sudden_volume_change', 'volume', 'warning', 'Data/Ops', 'Find sudden load, offer, API, webhook, or search volume changes.', 'Check customer launch, abuse, integration loop, or outage conditions.', 'hourly', 3)
ON CONFLICT (rule_key)
DO UPDATE SET
    category = EXCLUDED.category,
    severity = EXCLUDED.severity,
    owner_team = EXCLUDED.owner_team,
    check_sql = EXCLUDED.check_sql,
    repair_playbook = EXCLUDED.repair_playbook,
    cadence = EXCLUDED.cadence,
    active = TRUE,
    alert_threshold = EXCLUDED.alert_threshold,
    updated_at = CURRENT_TIMESTAMP;
