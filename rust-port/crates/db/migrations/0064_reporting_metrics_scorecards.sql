-- Phase 14 reporting, metrics, and scorecard foundations.

CREATE TABLE IF NOT EXISTS business_metric_definitions (
    id BIGSERIAL PRIMARY KEY,
    metric_key VARCHAR(120) NOT NULL UNIQUE,
    display_name VARCHAR(160) NOT NULL,
    category VARCHAR(80) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    business_definition TEXT NOT NULL,
    numerator_definition TEXT NULL,
    denominator_definition TEXT NULL,
    calculation_sql TEXT NULL,
    grain VARCHAR(80) NOT NULL DEFAULT 'organization_day',
    refresh_cadence VARCHAR(80) NOT NULL DEFAULT 'daily',
    target_direction VARCHAR(24) NOT NULL,
    accepted_by VARCHAR(160) NOT NULL,
    accepted_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_business_metric_direction
        CHECK (target_direction IN ('higher_is_better', 'lower_is_better', 'target_band', 'informational'))
);

CREATE TABLE IF NOT EXISTS reporting_read_models (
    id BIGSERIAL PRIMARY KEY,
    model_key VARCHAR(120) NOT NULL UNIQUE,
    display_name VARCHAR(160) NOT NULL,
    source_tables TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    target_table VARCHAR(120) NOT NULL,
    refresh_strategy VARCHAR(80) NOT NULL,
    refresh_cadence VARCHAR(80) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    operational_screen_safe BOOLEAN NOT NULL DEFAULT TRUE,
    last_refresh_status VARCHAR(32) NOT NULL DEFAULT 'not_started',
    last_refreshed_at TIMESTAMP(6) NULL,
    warehouse_export_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_reporting_refresh_status
        CHECK (last_refresh_status IN ('not_started', 'running', 'success', 'failed', 'stale')),
    CONSTRAINT chk_reporting_refresh_strategy
        CHECK (refresh_strategy IN ('materialized_view', 'incremental_table', 'warehouse_export', 'scheduled_snapshot', 'event_projection'))
);

CREATE TABLE IF NOT EXISTS reporting_refresh_runs (
    id BIGSERIAL PRIMARY KEY,
    read_model_id BIGINT NOT NULL REFERENCES reporting_read_models(id) ON DELETE CASCADE,
    run_status VARCHAR(32) NOT NULL DEFAULT 'running',
    started_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMP(6) NULL,
    row_count BIGINT NULL,
    error_message TEXT NULL,
    triggered_by VARCHAR(80) NOT NULL DEFAULT 'scheduled',
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_reporting_refresh_run_status
        CHECK (run_status IN ('running', 'success', 'failed', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_reporting_refresh_runs_model_time
    ON reporting_refresh_runs (read_model_id, started_at DESC);

CREATE TABLE IF NOT EXISTS customer_scorecards (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    customer_organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE SET NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    posted_loads INTEGER NOT NULL DEFAULT 0,
    booked_loads INTEGER NOT NULL DEFAULT 0,
    acceptance_rate NUMERIC(7,4) NULL,
    quote_to_book_minutes INTEGER NULL,
    on_time_pickup_rate NUMERIC(7,4) NULL,
    on_time_delivery_rate NUMERIC(7,4) NULL,
    document_cycle_minutes INTEGER NULL,
    gross_margin_cents BIGINT NULL,
    dispute_rate NUMERIC(7,4) NULL,
    score NUMERIC(6,2) NULL,
    score_tone VARCHAR(32) NOT NULL DEFAULT 'info',
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_customer_scorecard_dates CHECK (period_end >= period_start),
    CONSTRAINT chk_customer_scorecard_tone CHECK (score_tone IN ('success', 'warning', 'danger', 'info'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_customer_scorecard_period
    ON customer_scorecards (
        organization_id,
        COALESCE(customer_organization_id, 0),
        period_start,
        period_end
    );

CREATE TABLE IF NOT EXISTS carrier_scorecards (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    carrier_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    carrier_organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE SET NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    offered_loads INTEGER NOT NULL DEFAULT 0,
    accepted_loads INTEGER NOT NULL DEFAULT 0,
    acceptance_rate NUMERIC(7,4) NULL,
    tracking_compliance_rate NUMERIC(7,4) NULL,
    on_time_pickup_rate NUMERIC(7,4) NULL,
    on_time_delivery_rate NUMERIC(7,4) NULL,
    claims_rate NUMERIC(7,4) NULL,
    document_quality_rate NUMERIC(7,4) NULL,
    payout_cycle_hours INTEGER NULL,
    score NUMERIC(6,2) NULL,
    score_tone VARCHAR(32) NOT NULL DEFAULT 'info',
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_carrier_scorecard_dates CHECK (period_end >= period_start),
    CONSTRAINT chk_carrier_scorecard_tone CHECK (score_tone IN ('success', 'warning', 'danger', 'info'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_carrier_scorecard_period
    ON carrier_scorecards (
        organization_id,
        COALESCE(carrier_user_id, 0),
        COALESCE(carrier_organization_id, 0),
        period_start,
        period_end
    );

INSERT INTO business_metric_definitions (
    metric_key, display_name, category, owner_team, business_definition,
    numerator_definition, denominator_definition, grain, refresh_cadence,
    target_direction, accepted_by, notes
)
VALUES
    ('posted_loads', 'Posted loads', 'volume', 'Product/Data/Ops', 'Count of loads posted by an organization during the reporting period.', 'Loads created with a posted or later lifecycle state.', NULL, 'organization_day', 'hourly', 'higher_is_better', 'Product/Data/Ops', 'Primary demand metric.'),
    ('booked_loads', 'Booked loads', 'volume', 'Product/Data/Ops', 'Count of load legs with a booked carrier during the reporting period.', 'Booked load legs.', NULL, 'organization_day', 'hourly', 'higher_is_better', 'Product/Data/Ops', 'Used for marketplace and broker throughput.'),
    ('acceptance_rate', 'Acceptance rate', 'marketplace', 'Product/Data/Ops', 'Share of offers or tenders accepted by carriers.', 'Accepted offers or tenders.', 'Total eligible offers or tenders.', 'organization_carrier_period', 'daily', 'higher_is_better', 'Product/Data/Ops', 'Scorecard input.'),
    ('quote_to_book_time', 'Quote-to-book time', 'marketplace', 'Product/Data/Ops', 'Elapsed time from quote/load posting to carrier booking.', 'Minutes between quote/post and booking.', 'Booked loads.', 'organization_lane_period', 'daily', 'lower_is_better', 'Product/Data/Ops', 'Operational speed indicator.'),
    ('tracking_compliance', 'Tracking compliance', 'execution', 'Product/Data/Ops', 'Share of active execution legs with consented and timely tracking updates.', 'Legs meeting tracking consent and freshness rules.', 'Active execution legs requiring tracking.', 'organization_carrier_period', 'hourly', 'higher_is_better', 'Product/Data/Ops', 'Customer-visible execution metric.'),
    ('on_time_pickup', 'On-time pickup', 'execution', 'Product/Data/Ops', 'Share of pickup appointments completed within the accepted on-time window.', 'On-time pickup events.', 'Pickup appointments with required timestamps.', 'organization_carrier_period', 'daily', 'higher_is_better', 'Product/Data/Ops', 'Scorecard input.'),
    ('on_time_delivery', 'On-time delivery', 'execution', 'Product/Data/Ops', 'Share of delivery appointments completed within the accepted on-time window.', 'On-time delivery events.', 'Delivery appointments with required timestamps.', 'organization_carrier_period', 'daily', 'higher_is_better', 'Product/Data/Ops', 'Scorecard input.'),
    ('document_cycle_time', 'Document cycle time', 'documents', 'Product/Data/Ops', 'Elapsed time from delivery completion to required document approval.', 'Minutes between delivery completion and final required document approval.', 'Delivered loads requiring closeout documents.', 'organization_customer_period', 'daily', 'lower_is_better', 'Product/Data/Ops', 'Closeout and payment readiness metric.'),
    ('margin', 'Margin', 'finance', 'Product/Data/Ops', 'Gross margin generated by completed loads before platform overhead.', 'Customer revenue minus carrier cost and approved accessorials.', 'Completed loads with finance ledger entries.', 'organization_customer_period', 'daily', 'higher_is_better', 'Product/Data/Ops', 'Finance reporting input.'),
    ('payout_time', 'Payout time', 'finance', 'Product/Data/Ops', 'Elapsed time from financial release eligibility to carrier payout completion.', 'Hours from release eligibility to payout complete.', 'Carrier payouts in period.', 'organization_carrier_period', 'daily', 'lower_is_better', 'Product/Data/Ops', 'Carrier experience metric.'),
    ('dispute_rate', 'Dispute rate', 'finance', 'Product/Data/Ops', 'Share of completed loads with claims, detention disputes, accessorial disputes, or settlement disputes.', 'Completed loads with open or resolved disputes.', 'Completed loads.', 'organization_customer_period', 'daily', 'lower_is_better', 'Product/Data/Ops', 'Risk and customer quality metric.')
ON CONFLICT (metric_key)
DO UPDATE SET
    display_name = EXCLUDED.display_name,
    category = EXCLUDED.category,
    owner_team = EXCLUDED.owner_team,
    business_definition = EXCLUDED.business_definition,
    numerator_definition = EXCLUDED.numerator_definition,
    denominator_definition = EXCLUDED.denominator_definition,
    grain = EXCLUDED.grain,
    refresh_cadence = EXCLUDED.refresh_cadence,
    target_direction = EXCLUDED.target_direction,
    accepted_by = EXCLUDED.accepted_by,
    active = TRUE,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO reporting_read_models (
    model_key, display_name, source_tables, target_table, refresh_strategy,
    refresh_cadence, owner_team, operational_screen_safe, warehouse_export_enabled, notes
)
VALUES
    ('load_operational_metrics_daily', 'Daily load operational metrics', ARRAY['loads','load_legs','offers','tracking_events','load_documents'], 'reporting_load_operational_metrics_daily', 'incremental_table', 'hourly', 'Data/Backend', TRUE, TRUE, 'Feeds Phase 14 operational dashboards without querying hot workflow screens.'),
    ('finance_metrics_daily', 'Daily finance metrics', ARRAY['payment_ledger_entries','finance_approval_requests','invoices','carrier_settlements'], 'reporting_finance_metrics_daily', 'incremental_table', 'daily', 'Data/Finance', TRUE, TRUE, 'Feeds margin, payout, dispute, invoice, and settlement reporting.'),
    ('customer_scorecards_monthly', 'Monthly customer scorecards', ARRAY['loads','load_legs','payment_ledger_entries','load_documents'], 'customer_scorecards', 'scheduled_snapshot', 'daily', 'Data/Ops', TRUE, TRUE, 'One row per customer and period.'),
    ('carrier_scorecards_monthly', 'Monthly carrier scorecards', ARRAY['offers','load_legs','tracking_events','load_documents','carrier_settlements'], 'carrier_scorecards', 'scheduled_snapshot', 'daily', 'Data/Ops', TRUE, TRUE, 'One row per carrier and period.')
ON CONFLICT (model_key)
DO UPDATE SET
    display_name = EXCLUDED.display_name,
    source_tables = EXCLUDED.source_tables,
    target_table = EXCLUDED.target_table,
    refresh_strategy = EXCLUDED.refresh_strategy,
    refresh_cadence = EXCLUDED.refresh_cadence,
    owner_team = EXCLUDED.owner_team,
    operational_screen_safe = EXCLUDED.operational_screen_safe,
    warehouse_export_enabled = EXCLUDED.warehouse_export_enabled,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;
