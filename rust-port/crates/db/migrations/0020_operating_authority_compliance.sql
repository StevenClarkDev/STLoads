CREATE TABLE IF NOT EXISTS operating_authority_decisions (
    id BIGSERIAL PRIMARY KEY,
    model_key TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    scope TEXT NOT NULL,
    decision TEXT NOT NULL,
    regulatory_note TEXT NOT NULL,
    owner TEXT NOT NULL,
    approved_by TEXT NULL,
    approved_at TIMESTAMP NULL,
    next_review_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS compliance_evidence_records (
    id BIGSERIAL PRIMARY KEY,
    evidence_key TEXT NOT NULL,
    label TEXT NOT NULL,
    owner TEXT NOT NULL,
    status TEXT NOT NULL,
    evidence_type TEXT NOT NULL,
    issuer TEXT NULL,
    policy_or_authority_number TEXT NULL,
    coverage_amount_cents BIGINT NULL,
    currency TEXT NULL,
    jurisdiction TEXT NULL,
    document_uri TEXT NULL,
    customer_disclosable BOOLEAN NOT NULL DEFAULT FALSE,
    renewal_required BOOLEAN NOT NULL DEFAULT FALSE,
    effective_at DATE NULL,
    expires_at DATE NULL,
    review_due_at DATE NULL,
    notes TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_expiry
    ON compliance_evidence_records (expires_at, review_due_at)
    WHERE renewal_required = TRUE;

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_customer_package
    ON compliance_evidence_records (customer_disclosable, status, evidence_key);

INSERT INTO operating_authority_decisions (
    model_key, label, scope, decision, regulatory_note, owner, approved_by, approved_at, next_review_at
)
VALUES
    ('software_tms_extension', 'Software-only loadboard and TMS extension', 'in_scope_first_release', 'STLoads provides software workflows, customer-configured loadboard operations, TMS handoffs, documents, audit, and payment orchestration without taking carrier, broker, or freight-forwarder responsibility.', 'Requires customer contract clarity and legal review, but does not intentionally represent STLoads as motor carrier, broker, or freight forwarder.', 'Legal / Product', NULL, NULL, CURRENT_DATE + INTERVAL '90 days'),
    ('carrier_marketplace', 'Carrier marketplace', 'in_scope_first_release', 'STLoads may surface customer/carrier freight opportunities as a software marketplace while the contracting customer remains responsible for its own authority and freight obligations.', 'Marketplace copy must avoid implying STLoads arranges freight as broker of record unless broker authority is approved.', 'Legal / Product', NULL, NULL, CURRENT_DATE + INTERVAL '90 days'),
    ('broker_of_record', 'Broker of record', 'explicitly_out_of_scope', 'STLoads must not act as broker of record in the first enterprise release.', 'FMCSA requires brokers to register, and broker registration for property requires financial security such as BMC-84 or BMC-85.', 'Legal / Operations', NULL, NULL, CURRENT_DATE + INTERVAL '90 days'),
    ('freight_forwarder', 'Freight forwarder', 'explicitly_out_of_scope', 'STLoads must not act as freight forwarder in the first enterprise release.', 'FMCSA requires freight forwarders to register; freight forwarders assume responsibility for transportation.', 'Legal / Operations', NULL, NULL, CURRENT_DATE + INTERVAL '90 days'),
    ('motor_carrier', 'Motor carrier', 'explicitly_out_of_scope', 'STLoads must not operate commercial motor vehicles or hold itself out as a carrier in the first enterprise release.', 'Motor carrier operations require separate safety/commercial registration analysis.', 'Legal / Operations', NULL, NULL, CURRENT_DATE + INTERVAL '90 days'),
    ('payment_facilitator', 'Payment facilitator', 'legal_review_required', 'STLoads may orchestrate Stripe Connect flows only within the approved processor model and must not market itself as a bank, money transmitter, escrow agent, or payment facilitator without finance/legal approval.', 'Payment-facilitator, money-transmission, and escrow claims remain out of scope until finance/legal review is complete.', 'Finance / Legal', NULL, NULL, CURRENT_DATE + INTERVAL '90 days')
ON CONFLICT (model_key) DO UPDATE SET
    label = EXCLUDED.label,
    scope = EXCLUDED.scope,
    decision = EXCLUDED.decision,
    regulatory_note = EXCLUDED.regulatory_note,
    owner = EXCLUDED.owner,
    next_review_at = EXCLUDED.next_review_at,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO compliance_evidence_records (
    evidence_key, label, owner, status, evidence_type, customer_disclosable,
    renewal_required, review_due_at, notes
)
VALUES
    ('operating_model_memo', 'Operating model legal memo', 'Legal', 'required', 'legal_memo', TRUE, FALSE, CURRENT_DATE + INTERVAL '30 days', 'Required before enterprise launch; documents software-only and marketplace boundaries.'),
    ('broker_authority', 'FMCSA broker authority', 'Legal / Operations', 'not_required_current_model', 'operating_authority', TRUE, TRUE, CURRENT_DATE + INTERVAL '90 days', 'Not required while broker-of-record model remains out of scope.'),
    ('freight_forwarder_authority', 'FMCSA freight-forwarder authority', 'Legal / Operations', 'not_required_current_model', 'operating_authority', TRUE, TRUE, CURRENT_DATE + INTERVAL '90 days', 'Not required while freight-forwarder model remains out of scope.'),
    ('cyber_liability', 'Cyber liability insurance', 'Finance / Security', 'required', 'insurance_certificate', TRUE, TRUE, CURRENT_DATE + INTERVAL '30 days', 'Certificate and policy limits required for enterprise evidence package.'),
    ('technology_errors_omissions', 'Technology E&O insurance', 'Finance / Legal', 'required', 'insurance_certificate', TRUE, TRUE, CURRENT_DATE + INTERVAL '30 days', 'Certificate and policy limits required for enterprise evidence package.'),
    ('general_liability', 'Commercial general liability insurance', 'Finance', 'required', 'insurance_certificate', TRUE, TRUE, CURRENT_DATE + INTERVAL '30 days', 'Certificate required for enterprise evidence package.'),
    ('contingent_cargo', 'Contingent cargo / broker liability insurance', 'Finance / Legal', 'customer_specific', 'insurance_certificate', TRUE, TRUE, CURRENT_DATE + INTERVAL '60 days', 'Required only when customer contract or operating model requires it.'),
    ('state_province_registration', 'State/province operating registrations', 'Legal / Operations', 'legal_review_required', 'jurisdiction_registration', TRUE, TRUE, CURRENT_DATE + INTERVAL '60 days', 'Jurisdiction requirements must be reviewed before expanding regulated operations.')
ON CONFLICT DO NOTHING;
