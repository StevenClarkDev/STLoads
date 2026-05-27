# Enterprise Privacy And Data Request Workflow

This document closes `ENT-1704` by defining how STLoads handles customer and data-subject privacy requests for enterprise customers. It complements `docs/LEGAL_AGREEMENT_WORKFLOW.md`, `docs/ENTERPRISE_CUSTOMER_OFFBOARDING.md`, and `docs/ENTERPRISE_DATA_CLASSIFICATION_AND_ENCRYPTION.md`.

## Scope

Covered request types:

- Data export or access requests.
- Deletion and tenant offboarding requests.
- Correction requests for profile, company, carrier, facility, and contact records.
- Restriction or objection requests for nonessential processing.
- Legal hold, preservation, audit, finance, tax, fraud, and dispute exceptions.

Covered data classes:

- Account, identity, role, tenant, and support data.
- Load, offer, booking, dispatch, tracking, appointment, document, and chat data.
- Audit ledger, integration event, payment ledger, invoice, settlement, and subscription data.
- Location, document, compliance, risk, and fraud-review data.

## Intake Channels

Requests may enter through support, customer success, legal, security, contract notice, or a tenant administrator. Every request must be recorded in the customer support or governance tracker with:

- Requester name, company, tenant, email, phone if available, and role.
- Request type, jurisdiction if known, requested deadline, and legal basis if supplied.
- Affected tenant, users, loads, documents, integrations, payments, and date range.
- Whether the request is tied to contract termination, litigation, dispute, fraud, chargeback, insurance, or safety event.

Do not process a request from chat, email, or phone alone without identity and authority verification.

## Identity And Authority Verification

Before export, deletion, or restriction:

- For tenant-level requests, verify the requester is an authorized tenant administrator or contract/legal representative.
- For user-level requests, verify the requester controls the account email or is authorized by the enterprise customer.
- For broker, shipper, carrier, or driver records controlled by a customer tenant, confirm whether STLoads is processor/service provider and route approvals through the customer controller.
- For legal hold, subpoena, insurance claim, safety, fraud, or payment dispute cases, require Legal approval before deletion or restriction.

Failed or ambiguous verification keeps the request open but blocked.

## Request Handling

### Data Export

Exports must include only the approved scope. The default export package may include:

- Tenant/company profile, users, roles, access-review evidence, and support notes.
- Loads, offers, bookings, tender history, dispatch events, tracking events, appointments, and closeout state.
- Documents, document metadata, version history, validation/quarantine status, and retention metadata.
- Financial records owned by the customer scope: invoices, settlements, payment ledger references, subscription invoices, and finance approvals.
- Integration records: external IDs, idempotency keys, webhook delivery summaries, EDI/TMS events, and reconciliation status.
- Audit events relevant to the requester, with secrets, tokens, and unrelated tenant data redacted.

Exports must not include:

- Raw secrets, API keys, password hashes, session tokens, webhook secrets, private keys, or MFA seeds.
- Raw card, bank, or payment credentials.
- Other tenant data.
- Internal vulnerability, fraud-model, security-rule, or privileged employee notes unless Legal approves.

### Deletion

Deletion is allowed only after retention, legal hold, finance, tax, safety, insurance, and dispute checks. The default order is:

1. Confirm request authority and scope.
2. Freeze or disable affected integrations where needed.
3. Export data if contract, law, or customer policy requires it.
4. Remove or anonymize optional profile, preference, support, and contact data.
5. Retain legally required audit, tax, payment, safety, compliance, and dispute records until their retention period expires.
6. Queue object storage and backup deletion according to the approved retention schedule.
7. Record final evidence: requester, approvers, scope, retained categories, deleted categories, timestamps, and exceptions.

Hard deletion of audit ledger, payment ledger, tax, safety, or compliance evidence requires Legal and Security approval.

### Correction

Correction requests should update the canonical governed record, not only a derived view. If a correction affects compliance, invoice, payment, tax, authority, insurance, fraud, or dispute records, preserve an audit event showing the previous value, new value, approver, and reason.

### Restriction Or Objection

For nonessential processing, disable analytics, marketing, enrichment, support visibility, or customer-success processing where applicable. Operational processing required for active loads, payments, compliance, disputes, safety, or contract obligations may continue with Legal approval.

## Data Category Rules

| Category | Export | Delete | Notes |
| --- | --- | --- | --- |
| Account and identity | Yes | Delete/anonymize when no longer required | Preserve access/audit history when required. |
| Load and dispatch | Yes | Retain while contract, safety, tax, or dispute obligations exist | Use tenant boundaries. |
| Location/tracking | Yes | Restrict/delete after retention unless tied to active dispute or delivery proof | Respect consent records. |
| Documents | Yes | Delete/retain by document type and legal hold | Quarantined files follow same legal review. |
| Chat/support notes | Scoped | Redact internal-only security/fraud notes unless approved | Avoid unrelated users/tenants. |
| Audit ledger | Scoped/redacted | Usually retained | Needed for security, compliance, and dispute evidence. |
| Payment/finance | Scoped references | Retain per finance/tax/chargeback obligations | Raw payment credentials are never stored by STLoads. |
| Integration events | Yes | Retain while needed for reconciliation and dispute evidence | Redact tokens and secrets. |

## Timelines

- Acknowledge customer-facing privacy requests within 5 business days.
- Complete standard export, correction, or restriction requests within 30 calendar days unless contract or law requires a shorter period.
- Escalate deletion requests involving finance, compliance, safety, legal hold, or dispute data within 2 business days.
- Record any extension reason and revised due date in the tracker.

## Evidence

Every request must preserve:

- Intake ticket and identity verification evidence.
- Scope approval from customer admin, Legal, Security, Finance, or Product as applicable.
- Export manifest, deletion/anonymization manifest, retained-data exception list, and completion timestamp.
- Customer response and any contract/legal notice.

## Completion Criteria

A request is complete only when the requester receives the approved response, all approved actions are executed or queued, retained categories are documented, and evidence is attached to the governance tracker.
