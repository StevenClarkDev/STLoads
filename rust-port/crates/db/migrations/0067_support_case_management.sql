CREATE TABLE IF NOT EXISTS support_cases (
    id BIGSERIAL PRIMARY KEY,
    case_number TEXT NOT NULL UNIQUE,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    reporter_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    affected_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    related_entity_type TEXT NULL,
    related_entity_id TEXT NULL,
    channel TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    owner_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    owner_team TEXT NOT NULL,
    escalation_owner_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    category TEXT NOT NULL,
    customer_impact TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    first_response_due_at TIMESTAMP(6) NOT NULL,
    next_update_due_at TIMESTAMP(6) NOT NULL,
    resolution_due_at TIMESTAMP(6) NOT NULL,
    first_responded_at TIMESTAMP(6) NULL,
    resolved_at TIMESTAMP(6) NULL,
    closed_at TIMESTAMP(6) NULL,
    breach_state TEXT NOT NULL DEFAULT 'ok',
    resolution_reason TEXT NULL,
    root_cause_category TEXT NULL,
    follow_up_action TEXT NULL,
    feedback_score INTEGER NULL CHECK (feedback_score BETWEEN 1 AND 5),
    feedback_comment TEXT NULL,
    feedback_received_at TIMESTAMP(6) NULL,
    created_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (severity IN ('sev1', 'sev2', 'sev3', 'sev4')),
    CHECK (status IN ('new', 'triage', 'waiting_on_customer', 'waiting_on_stloads', 'escalated', 'engineering_review', 'product_review', 'resolved', 'closed', 'reopened')),
    CHECK (breach_state IN ('ok', 'at_risk', 'breached'))
);

CREATE INDEX IF NOT EXISTS idx_support_cases_org_status
    ON support_cases (organization_id, status, severity, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_support_cases_sla
    ON support_cases (organization_id, breach_state, resolution_due_at);

CREATE INDEX IF NOT EXISTS idx_support_cases_related_entity
    ON support_cases (organization_id, related_entity_type, related_entity_id);

CREATE TABLE IF NOT EXISTS support_case_events (
    id BIGSERIAL PRIMARY KEY,
    support_case_id BIGINT NOT NULL REFERENCES support_cases(id) ON DELETE CASCADE,
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    actor_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    visibility TEXT NOT NULL,
    previous_status TEXT NULL,
    new_status TEXT NULL,
    note TEXT NULL,
    customer_update TEXT NULL,
    internal_note TEXT NULL,
    feedback_score INTEGER NULL CHECK (feedback_score BETWEEN 1 AND 5),
    metadata JSONB NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (visibility IN ('internal', 'customer_visible'))
);

CREATE INDEX IF NOT EXISTS idx_support_case_events_case_created
    ON support_case_events (support_case_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_support_case_events_org_visibility
    ON support_case_events (organization_id, visibility, created_at DESC);
