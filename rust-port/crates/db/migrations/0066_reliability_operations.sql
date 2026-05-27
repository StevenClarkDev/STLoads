-- Phase 15 observability, workers, scale, disaster recovery, and guardrails.

CREATE TABLE IF NOT EXISTS observability_signal_catalog (
    id BIGSERIAL PRIMARY KEY,
    signal_key VARCHAR(160) NOT NULL UNIQUE,
    signal_type VARCHAR(32) NOT NULL,
    surface VARCHAR(80) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    telemetry_source VARCHAR(120) NOT NULL,
    retention_days INTEGER NOT NULL DEFAULT 30,
    required_for_p0 BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_observability_signal_type
        CHECK (signal_type IN ('log', 'metric', 'trace', 'event'))
);

CREATE TABLE IF NOT EXISTS alert_rules (
    id BIGSERIAL PRIMARY KEY,
    rule_key VARCHAR(160) NOT NULL UNIQUE,
    signal_key VARCHAR(160) NOT NULL,
    severity VARCHAR(24) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    condition_summary TEXT NOT NULL,
    threshold_value VARCHAR(80) NOT NULL,
    evaluation_window_minutes INTEGER NOT NULL DEFAULT 5,
    route_key VARCHAR(80) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    runbook_url TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_alert_rule_severity CHECK (severity IN ('P0', 'P1', 'P2', 'P3'))
);

CREATE TABLE IF NOT EXISTS on_call_escalation_policies (
    id BIGSERIAL PRIMARY KEY,
    route_key VARCHAR(80) NOT NULL UNIQUE,
    owner_team VARCHAR(80) NOT NULL,
    primary_ack_minutes INTEGER NOT NULL,
    secondary_escalation_minutes INTEGER NOT NULL,
    executive_escalation_minutes INTEGER NOT NULL,
    paging_tool VARCHAR(80) NOT NULL,
    security_log_export_required BOOLEAN NOT NULL DEFAULT FALSE,
    customer_update_required BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS security_log_export_policies (
    id BIGSERIAL PRIMARY KEY,
    export_key VARCHAR(160) NOT NULL UNIQUE,
    event_family VARCHAR(80) NOT NULL,
    destination_type VARCHAR(80) NOT NULL,
    retention_days INTEGER NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    customer_export_supported BOOLEAN NOT NULL DEFAULT FALSE,
    evidence_request_sla_hours INTEGER NOT NULL DEFAULT 72,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS background_jobs (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_type VARCHAR(120) NOT NULL,
    queue_name VARCHAR(80) NOT NULL DEFAULT 'default',
    status VARCHAR(32) NOT NULL DEFAULT 'queued',
    priority INTEGER NOT NULL DEFAULT 100,
    payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    locked_by VARCHAR(120) NULL,
    locked_until TIMESTAMP(6) NULL,
    visibility_timeout_seconds INTEGER NOT NULL DEFAULT 300,
    next_run_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_error TEXT NULL,
    dead_letter_reason TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_background_job_status
        CHECK (status IN ('queued', 'running', 'retry_scheduled', 'succeeded', 'dead_letter', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_background_jobs_claim
    ON background_jobs (queue_name, status, next_run_at, priority, id)
    WHERE status IN ('queued', 'retry_scheduled');

CREATE INDEX IF NOT EXISTS idx_background_jobs_dead_letter
    ON background_jobs (queue_name, status, updated_at DESC)
    WHERE status = 'dead_letter';

CREATE TABLE IF NOT EXISTS query_performance_controls (
    id BIGSERIAL PRIMARY KEY,
    query_key VARCHAR(160) NOT NULL UNIQUE,
    surface VARCHAR(80) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    expected_p95_ms INTEGER NOT NULL,
    pagination_strategy VARCHAR(80) NOT NULL,
    required_indexes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    explain_plan_required BOOLEAN NOT NULL DEFAULT TRUE,
    last_reviewed_at TIMESTAMP(6) NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS backup_restore_policies (
    id BIGSERIAL PRIMARY KEY,
    policy_key VARCHAR(160) NOT NULL UNIQUE,
    asset_type VARCHAR(80) NOT NULL,
    rpo_minutes INTEGER NOT NULL,
    rto_minutes INTEGER NOT NULL,
    backup_strategy TEXT NOT NULL,
    restore_strategy TEXT NOT NULL,
    failover_capability VARCHAR(80) NOT NULL,
    last_restore_test_at TIMESTAMP(6) NULL,
    owner_team VARCHAR(80) NOT NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS archive_policies (
    id BIGSERIAL PRIMARY KEY,
    policy_key VARCHAR(160) NOT NULL UNIQUE,
    table_family VARCHAR(120) NOT NULL,
    retention_days INTEGER NOT NULL,
    archive_strategy VARCHAR(80) NOT NULL,
    restore_supported BOOLEAN NOT NULL DEFAULT TRUE,
    owner_team VARCHAR(80) NOT NULL,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS incident_runbooks (
    id BIGSERIAL PRIMARY KEY,
    runbook_key VARCHAR(160) NOT NULL UNIQUE,
    severity_default VARCHAR(24) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    customer_communication_required BOOLEAN NOT NULL DEFAULT TRUE,
    status_page_required BOOLEAN NOT NULL DEFAULT FALSE,
    detection_signals TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    first_15_minutes TEXT NOT NULL,
    mitigation_steps TEXT NOT NULL,
    post_incident_template TEXT NOT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_incident_runbook_severity CHECK (severity_default IN ('P0', 'P1', 'P2', 'P3'))
);

CREATE TABLE IF NOT EXISTS continuity_exercises (
    id BIGSERIAL PRIMARY KEY,
    exercise_key VARCHAR(160) NOT NULL UNIQUE,
    scenario VARCHAR(160) NOT NULL,
    owner_team VARCHAR(80) NOT NULL,
    planned_at TIMESTAMP(6) NOT NULL,
    completed_at TIMESTAMP(6) NULL,
    evidence_url TEXT NULL,
    gaps_found INTEGER NOT NULL DEFAULT 0,
    follow_up_owner VARCHAR(80) NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'planned',
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_continuity_exercise_status CHECK (status IN ('planned', 'completed', 'missed', 'cancelled'))
);

CREATE TABLE IF NOT EXISTS usage_quota_policies (
    id BIGSERIAL PRIMARY KEY,
    quota_key VARCHAR(160) NOT NULL UNIQUE,
    provider_key VARCHAR(80) NOT NULL,
    metered_resource VARCHAR(120) NOT NULL,
    scope VARCHAR(80) NOT NULL,
    soft_limit BIGINT NOT NULL,
    hard_limit BIGINT NOT NULL,
    reset_period VARCHAR(40) NOT NULL,
    alert_route_key VARCHAR(80) NOT NULL,
    approval_required_above_hard_limit BOOLEAN NOT NULL DEFAULT TRUE,
    customer_visible BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS provider_spend_controls (
    id BIGSERIAL PRIMARY KEY,
    provider_key VARCHAR(80) NOT NULL UNIQUE,
    owner_team VARCHAR(80) NOT NULL,
    monthly_budget_cents BIGINT NOT NULL,
    warning_threshold_percent INTEGER NOT NULL DEFAULT 80,
    critical_threshold_percent INTEGER NOT NULL DEFAULT 100,
    alert_route_key VARCHAR(80) NOT NULL,
    customer_chargeback_supported BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO observability_signal_catalog (signal_key, signal_type, surface, owner_team, telemetry_source, retention_days, required_for_p0, notes)
VALUES
    ('http.request.latency', 'metric', 'http', 'Backend/DevOps', 'tower_http_trace', 30, TRUE, 'Request latency by route, method, status, and tenant.'),
    ('http.request.error_rate', 'metric', 'http', 'Backend/DevOps', 'tower_http_trace', 30, TRUE, '5xx and auth/session error rate.'),
    ('db.pool.usage', 'metric', 'database', 'Backend/DBA', 'sqlx_pool', 30, TRUE, 'Pool utilization and acquire latency.'),
    ('job.queue.lag', 'metric', 'workers', 'background_jobs', 30, TRUE, 'Oldest queued job by queue.'),
    ('worker.outcome', 'event', 'workers', 'Backend/DevOps', 'worker_runtime', 90, TRUE, 'Worker success, retry, dead letter, and shutdown events.'),
    ('email.delivery.failure', 'event', 'email', 'Backend/Ops', 'message_delivery_events', 180, TRUE, 'Bounces, complaints, retry exhaustion.'),
    ('webhook.delivery.failure', 'event', 'integrations', 'Integrations', 'webhook_delivery_logs', 180, TRUE, 'Webhook retry exhaustion and dead letters.'),
    ('storage.operation.error', 'event', 'documents', 'Backend/DevOps', 'document_storage', 180, TRUE, 'Object storage read/write/delete errors.'),
    ('payment.operation.failure', 'event', 'payments', 'Finance/Backend', 'stripe_and_payment_ledger', 365, TRUE, 'Payment release, webhook, and payout failures.'),
    ('tms.drift', 'event', 'integrations', 'Integrations', 'stloads_reconciliation_log', 180, TRUE, 'TMS/STLoads source-of-truth drift.')
ON CONFLICT (signal_key) DO UPDATE SET
    signal_type = EXCLUDED.signal_type,
    surface = EXCLUDED.surface,
    owner_team = EXCLUDED.owner_team,
    telemetry_source = EXCLUDED.telemetry_source,
    retention_days = EXCLUDED.retention_days,
    required_for_p0 = EXCLUDED.required_for_p0,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO on_call_escalation_policies (
    route_key, owner_team, primary_ack_minutes, secondary_escalation_minutes,
    executive_escalation_minutes, paging_tool, security_log_export_required,
    customer_update_required, notes
)
VALUES
    ('backend', 'Backend/DevOps', 5, 15, 30, 'PagerDuty-or-equivalent', FALSE, TRUE, 'API, auth, database, and app runtime incidents.'),
    ('workers', 'Backend/DevOps', 10, 20, 45, 'PagerDuty-or-equivalent', FALSE, TRUE, 'Email, TMS, webhook, reporting, and async processing incidents.'),
    ('payments', 'Finance/Backend', 5, 15, 30, 'PagerDuty-or-equivalent', TRUE, TRUE, 'Payment, payout, Stripe, invoice, settlement incidents.'),
    ('integrations', 'Integrations', 10, 30, 60, 'PagerDuty-or-equivalent', FALSE, TRUE, 'TMS, EDI, webhook, API partner incidents.'),
    ('security', 'Security/Ops', 5, 10, 20, 'PagerDuty-or-equivalent', TRUE, TRUE, 'Data exposure, suspicious auth, WAF, audit, or tenant-isolation incidents.'),
    ('cost', 'DevOps/Product/Finance', 60, 240, 480, 'Opsgenie-or-equivalent', FALSE, FALSE, 'Spend and usage guardrail alerts.')
ON CONFLICT (route_key) DO UPDATE SET
    owner_team = EXCLUDED.owner_team,
    primary_ack_minutes = EXCLUDED.primary_ack_minutes,
    secondary_escalation_minutes = EXCLUDED.secondary_escalation_minutes,
    executive_escalation_minutes = EXCLUDED.executive_escalation_minutes,
    paging_tool = EXCLUDED.paging_tool,
    security_log_export_required = EXCLUDED.security_log_export_required,
    customer_update_required = EXCLUDED.customer_update_required,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO alert_rules (rule_key, signal_key, severity, owner_team, condition_summary, threshold_value, evaluation_window_minutes, route_key, runbook_url)
VALUES
    ('p0_api_5xx_rate', 'http.request.error_rate', 'P0', 'Backend/DevOps', 'Sustained API 5xx rate impacts production traffic.', '>5% 5xx', 5, 'backend', 'docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md#api-or-auth-outage'),
    ('p1_request_latency', 'http.request.latency', 'P1', 'Backend/DevOps', 'P95 request latency exceeds customer-facing target.', 'p95 > 2000ms', 10, 'backend', 'docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md#api-or-auth-outage'),
    ('p0_payment_failure', 'payment.operation.failure', 'P0', 'Finance/Backend', 'Payment release, payout, or webhook failures exceed tolerance.', '>=1 release-blocking failure', 5, 'payments', 'docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md#payment-incident'),
    ('p1_worker_dead_letters', 'worker.outcome', 'P1', 'Backend/DevOps', 'Worker dead letters appear in any critical queue.', 'dead_letter >= 1', 15, 'workers', 'docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md#worker-or-queue-incident'),
    ('p1_tms_drift', 'tms.drift', 'P1', 'Integrations', 'TMS source-of-truth drift remains unresolved.', 'drift >= 1', 15, 'integrations', 'docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md#tms-outage'),
    ('p1_storage_errors', 'storage.operation.error', 'P1', 'Backend/DevOps', 'Object storage errors block document workflows.', 'error >= 1', 5, 'backend', 'docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md#object-storage-outage')
ON CONFLICT (rule_key) DO UPDATE SET
    signal_key = EXCLUDED.signal_key,
    severity = EXCLUDED.severity,
    owner_team = EXCLUDED.owner_team,
    condition_summary = EXCLUDED.condition_summary,
    threshold_value = EXCLUDED.threshold_value,
    evaluation_window_minutes = EXCLUDED.evaluation_window_minutes,
    route_key = EXCLUDED.route_key,
    runbook_url = EXCLUDED.runbook_url,
    active = TRUE,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO security_log_export_policies (export_key, event_family, destination_type, retention_days, owner_team, customer_export_supported, evidence_request_sla_hours, notes)
VALUES
    ('audit_events_siem', 'audit_events', 'siem_or_log_drain', 365, 'Security/Ops', TRUE, 72, 'Admin, support, finance, and break-glass events.'),
    ('auth_events_siem', 'auth_events', 'siem_or_log_drain', 365, 'Security/Ops', TRUE, 72, 'Login, MFA, SSO, SCIM, lockout, and token events.'),
    ('payment_risk_events_siem', 'payment_risk_events', 'siem_or_log_drain', 365, 'Finance/Security', TRUE, 72, 'Payment release, payout, webhook, and risk review events.'),
    ('infrastructure_logs_archive', 'infrastructure_logs', 'object_archive', 180, 'DevOps/Security', FALSE, 120, 'Runtime, WAF, deploy, and platform logs.')
ON CONFLICT (export_key) DO UPDATE SET
    event_family = EXCLUDED.event_family,
    destination_type = EXCLUDED.destination_type,
    retention_days = EXCLUDED.retention_days,
    owner_team = EXCLUDED.owner_team,
    customer_export_supported = EXCLUDED.customer_export_supported,
    evidence_request_sla_hours = EXCLUDED.evidence_request_sla_hours,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO query_performance_controls (query_key, surface, owner_team, expected_p95_ms, pagination_strategy, required_indexes, notes)
VALUES
    ('load_board_search', 'load_board', 'Backend/DBA', 500, 'cursor_by_created_at_and_id', ARRAY['idx_loads_org_status_created','idx_load_legs_status_pickup'], 'Carrier and broker load board search.'),
    ('chat_threads', 'chat', 'Backend/DBA', 500, 'cursor_by_last_message_id', ARRAY['idx_conversations_org_updated','idx_messages_conversation_created'], 'Conversation and message history.'),
    ('tracking_points', 'execution', 'Backend/DBA', 400, 'cursor_by_recorded_at', ARRAY['idx_tracking_leg_recorded_at'], 'Driver location history.'),
    ('admin_queues', 'admin', 'Backend/DBA', 700, 'cursor_by_status_created', ARRAY['idx_users_org_status','idx_loads_review_status'], 'Admin user and load review queues.'),
    ('tms_reconciliation', 'integrations', 'Backend/DBA', 1000, 'cursor_by_updated_at', ARRAY['idx_tms_handoffs_status_updated','idx_tms_reconciliation_log_created'], 'TMS drift and reconciliation queues.'),
    ('global_search', 'search', 'Backend/DBA', 700, 'cursor_by_last_indexed_at', ARRAY['idx_global_search_documents_text','ux_global_search_document_entity'], 'Permission-aware global search.'),
    ('reporting_scorecards', 'reporting', 'Backend/DBA', 800, 'period_then_score', ARRAY['ux_customer_scorecard_period','ux_carrier_scorecard_period'], 'Customer and carrier scorecards.')
ON CONFLICT (query_key) DO UPDATE SET
    surface = EXCLUDED.surface,
    owner_team = EXCLUDED.owner_team,
    expected_p95_ms = EXCLUDED.expected_p95_ms,
    pagination_strategy = EXCLUDED.pagination_strategy,
    required_indexes = EXCLUDED.required_indexes,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO backup_restore_policies (policy_key, asset_type, rpo_minutes, rto_minutes, backup_strategy, restore_strategy, failover_capability, owner_team, notes)
VALUES
    ('postgres_primary', 'postgresql', 15, 240, 'Managed backups with PITR where available plus pre-migration snapshots.', 'Restore latest safe point into staging, validate, then promote or reconnect app runtime.', 'restore_only_first_release', 'DevOps/DBA', 'Replica/failover can be added after enterprise pilot traffic justifies it.'),
    ('object_storage_documents', 'object_storage', 60, 360, 'Versioned bucket retention and lifecycle-protected document objects.', 'Restore objects by key/version and reconcile document metadata.', 'regional_provider_recovery', 'DevOps', 'Document restores must preserve hash/audit evidence.'),
    ('search_reporting_rebuild', 'derived_data', 240, 480, 'Derived read models can be rebuilt from operational tables.', 'Replay reporting refresh and search indexing jobs after DB restore.', 'rebuildable', 'Data/Backend', 'Search and reporting are eventually consistent.'),
    ('queue_replay', 'background_jobs', 15, 240, 'Durable job rows and external event dedupe records retained through backup.', 'Replay queued/retry_scheduled jobs after duplicate-safety checks.', 'restore_only_first_release', 'Backend/DevOps', 'Dead-letter rows are evidence, not automatically replayed.')
ON CONFLICT (policy_key) DO UPDATE SET
    asset_type = EXCLUDED.asset_type,
    rpo_minutes = EXCLUDED.rpo_minutes,
    rto_minutes = EXCLUDED.rto_minutes,
    backup_strategy = EXCLUDED.backup_strategy,
    restore_strategy = EXCLUDED.restore_strategy,
    failover_capability = EXCLUDED.failover_capability,
    owner_team = EXCLUDED.owner_team,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO archive_policies (policy_key, table_family, retention_days, archive_strategy, restore_supported, owner_team, notes)
VALUES
    ('location_pings', 'tracking_events', 180, 'partition_or_cold_table', TRUE, 'Backend/Data', 'Keep active execution hot; archive historical pings by retention policy.'),
    ('messages', 'chat_messages', 730, 'cold_table_with_search_summary', TRUE, 'Backend/Data', 'Preserve customer communication evidence.'),
    ('audit_events', 'audit_events', 2555, 'immutable_archive', TRUE, 'Security/Ops', 'Seven-year audit retention unless customer contract says otherwise.'),
    ('tms_handoffs', 'tms_handoffs', 730, 'closed_handoff_archive', TRUE, 'Integrations', 'Closed and terminal handoffs can move out of hot queues.'),
    ('document_metadata', 'load_documents', 2555, 'metadata_archive_with_object_retention', TRUE, 'Backend/Documents', 'Metadata archive must preserve file hash, version history, and legal hold.')
ON CONFLICT (policy_key) DO UPDATE SET
    table_family = EXCLUDED.table_family,
    retention_days = EXCLUDED.retention_days,
    archive_strategy = EXCLUDED.archive_strategy,
    restore_supported = EXCLUDED.restore_supported,
    owner_team = EXCLUDED.owner_team,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO incident_runbooks (runbook_key, severity_default, owner_team, customer_communication_required, status_page_required, detection_signals, first_15_minutes, mitigation_steps, post_incident_template)
VALUES
    ('auth_outage', 'P0', 'Backend/DevOps', TRUE, TRUE, ARRAY['http.request.error_rate','auth_events_siem'], 'Declare incident, assign commander, verify login/session/MFA scope, freeze risky deploys.', 'Rollback auth changes, disable risky SSO routing if needed, preserve audit evidence.', 'Impact, timeline, root cause, customer impact, remediation, prevention.'),
    ('database_outage', 'P0', 'DevOps/DBA', TRUE, TRUE, ARRAY['db.pool.usage','http.request.error_rate'], 'Confirm DB connectivity, check provider status, pause migrations/workers if needed.', 'Fail closed for writes, restore or reconnect, validate readiness and data integrity.', 'Impact, RPO/RTO, data validation, remediation, prevention.'),
    ('object_storage_outage', 'P1', 'Backend/DevOps', TRUE, TRUE, ARRAY['storage.operation.error'], 'Confirm upload/read scope and affected tenants.', 'Enable upload kill switch if needed, queue intake manually, reconcile after recovery.', 'Documents affected, manual workarounds, recovery evidence.'),
    ('payment_incident', 'P0', 'Finance/Backend', TRUE, TRUE, ARRAY['payment.operation.failure'], 'Freeze payout release if money movement risk exists, notify finance owner.', 'Use Stripe dashboard/provider evidence, reconcile ledger, require approval before replay.', 'Financial exposure, reconciled transactions, customer communication.'),
    ('duplicate_booking', 'P0', 'Backend/Ops', TRUE, TRUE, ARRAY['worker.outcome','audit_events_siem'], 'Stop affected booking surface and identify duplicate load legs/offers.', 'Pick source of truth, notify carriers/customer, compensate workflow manually.', 'Duplicate cause, affected parties, financial impact, prevention.'),
    ('tms_outage', 'P1', 'Integrations', TRUE, TRUE, ARRAY['tms.drift','webhook.delivery.failure'], 'Identify partner/TMS scope and pause unsafe pushes if needed.', 'Queue handoffs, replay after partner recovery, reconcile source-of-truth conflicts.', 'Partner outage window, queued/replayed handoffs, drift cleanup.'),
    ('email_outage', 'P1', 'Backend/Ops', TRUE, FALSE, ARRAY['email.delivery.failure'], 'Check SMTP/provider state, suppress risky retries if provider rejects traffic.', 'Use in-app notifications and support outreach; replay outbox when safe.', 'Templates affected, delivery evidence, fallback used.'),
    ('data_exposure', 'P0', 'Security/Ops', TRUE, TRUE, ARRAY['audit_events_siem','auth_events_siem'], 'Start security incident, preserve logs, stop exposure path, notify legal/security.', 'Revoke access, rotate secrets, export evidence, follow breach decision process.', 'Scope, timeline, data classes, notifications, corrective controls.'),
    ('bad_deploy', 'P0', 'DevOps/Engineering', TRUE, TRUE, ARRAY['http.request.error_rate','worker.outcome'], 'Freeze deploys, identify revision, compare health and errors.', 'Rollback app, disable feature flags, verify smoke and readiness.', 'Change cause, detection gap, rollback timeline, tests added.')
ON CONFLICT (runbook_key) DO UPDATE SET
    severity_default = EXCLUDED.severity_default,
    owner_team = EXCLUDED.owner_team,
    customer_communication_required = EXCLUDED.customer_communication_required,
    status_page_required = EXCLUDED.status_page_required,
    detection_signals = EXCLUDED.detection_signals,
    first_15_minutes = EXCLUDED.first_15_minutes,
    mitigation_steps = EXCLUDED.mitigation_steps,
    post_incident_template = EXCLUDED.post_incident_template,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO continuity_exercises (exercise_key, scenario, owner_team, planned_at, status, notes)
VALUES
    ('regional_provider_outage_tabletop', 'Regional provider outage with degraded API and object storage.', 'Ops/DevOps/Security/Customer Success', CURRENT_TIMESTAMP + INTERVAL '30 days', 'planned', 'Exercise manual active-load fallback, customer updates, and recovery sequencing.'),
    ('payment_provider_outage_tabletop', 'Stripe/payment provider outage during carrier payout window.', 'Finance/Ops/DevOps', CURRENT_TIMESTAMP + INTERVAL '45 days', 'planned', 'Exercise payment holds, customer/carrier communication, and replay sequencing.'),
    ('tms_provider_outage_tabletop', 'Primary TMS partner outage with queued handoffs and stale tracking.', 'Integrations/Ops', CURRENT_TIMESTAMP + INTERVAL '60 days', 'planned', 'Exercise manual updates, queue replay, source-of-truth repair, and customer notice.'),
    ('email_sms_outage_tabletop', 'Email/SMS provider outage during tender and POD workflows.', 'Ops/Support', CURRENT_TIMESTAMP + INTERVAL '75 days', 'planned', 'Exercise in-app fallback, manual outreach, and outbox replay.')
ON CONFLICT (exercise_key) DO UPDATE SET
    scenario = EXCLUDED.scenario,
    owner_team = EXCLUDED.owner_team,
    planned_at = EXCLUDED.planned_at,
    status = EXCLUDED.status,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO usage_quota_policies (quota_key, provider_key, metered_resource, scope, soft_limit, hard_limit, reset_period, alert_route_key, customer_visible, notes)
VALUES
    ('document_uploads_per_tenant', 'object_storage', 'document_uploads', 'tenant_month', 10000, 15000, 'monthly', 'cost', TRUE, 'Fair-use document upload guardrail.'),
    ('api_calls_per_partner', 'api_gateway', 'partner_api_calls', 'partner_day', 100000, 150000, 'daily', 'cost', TRUE, 'Partner API abuse and runaway integration guardrail.'),
    ('webhook_deliveries_per_tenant', 'webhooks', 'webhook_deliveries', 'tenant_day', 50000, 75000, 'daily', 'integrations', TRUE, 'Webhook fan-out guardrail.'),
    ('geocoding_requests_per_tenant', 'maps', 'geocoding_requests', 'tenant_month', 25000, 30000, 'monthly', 'cost', TRUE, 'Maps/geocoding spend guardrail.'),
    ('tracking_pings_per_tenant', 'tracking', 'location_pings', 'tenant_day', 250000, 350000, 'daily', 'workers', FALSE, 'Tracking storm guardrail.'),
    ('sandbox_resets_per_tenant', 'sandbox', 'sandbox_resets', 'tenant_day', 10, 20, 'daily', 'integrations', FALSE, 'Sandbox reset abuse guardrail.'),
    ('report_exports_per_tenant', 'reporting', 'report_exports', 'tenant_day', 100, 200, 'daily', 'cost', TRUE, 'Report export fair-use limit.'),
    ('notifications_per_tenant', 'messaging', 'notifications', 'tenant_day', 100000, 150000, 'daily', 'workers', TRUE, 'Notification runaway producer guardrail.')
ON CONFLICT (quota_key) DO UPDATE SET
    provider_key = EXCLUDED.provider_key,
    metered_resource = EXCLUDED.metered_resource,
    scope = EXCLUDED.scope,
    soft_limit = EXCLUDED.soft_limit,
    hard_limit = EXCLUDED.hard_limit,
    reset_period = EXCLUDED.reset_period,
    alert_route_key = EXCLUDED.alert_route_key,
    customer_visible = EXCLUDED.customer_visible,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO provider_spend_controls (provider_key, owner_team, monthly_budget_cents, warning_threshold_percent, critical_threshold_percent, alert_route_key, customer_chargeback_supported, notes)
VALUES
    ('database', 'DevOps/Finance', 250000, 80, 100, 'cost', FALSE, 'PostgreSQL compute, storage, backup, and egress spend.'),
    ('object_storage', 'DevOps/Finance', 150000, 80, 100, 'cost', TRUE, 'Document object storage, versioning, and egress spend.'),
    ('maps', 'Product/Finance', 100000, 75, 100, 'cost', TRUE, 'Geocoding, maps, and route distance spend.'),
    ('telematics', 'Product/Finance', 150000, 75, 100, 'cost', TRUE, 'ELD and telematics provider spend.'),
    ('email', 'Ops/Finance', 75000, 80, 100, 'cost', FALSE, 'SMTP/email provider spend.'),
    ('observability', 'DevOps/Finance', 125000, 80, 100, 'cost', FALSE, 'Logs, traces, metrics, and SIEM forwarding spend.'),
    ('edi', 'Integrations/Finance', 100000, 80, 100, 'cost', TRUE, 'EDI mailbox, VAN, parser, and replay spend.')
ON CONFLICT (provider_key) DO UPDATE SET
    owner_team = EXCLUDED.owner_team,
    monthly_budget_cents = EXCLUDED.monthly_budget_cents,
    warning_threshold_percent = EXCLUDED.warning_threshold_percent,
    critical_threshold_percent = EXCLUDED.critical_threshold_percent,
    alert_route_key = EXCLUDED.alert_route_key,
    customer_chargeback_supported = EXCLUDED.customer_chargeback_supported,
    notes = EXCLUDED.notes,
    updated_at = CURRENT_TIMESTAMP;
