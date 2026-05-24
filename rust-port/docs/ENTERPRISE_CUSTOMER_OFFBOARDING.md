# Enterprise Customer Offboarding And Data Return

This document defines the required customer-exit process for enterprise tenants. It is the operating contract for `ENT-0008` and must be updated whenever offboarding behavior, retention rules, or tenant access controls change.

## Goal

Enterprise customers must be able to leave STLoads without data loss, unresolved operational risk, unsafe access, or unclear financial/compliance obligations.

Offboarding is not complete until customer success, support, finance, legal, and engineering have all closed their required checks.

## Exit Triggers

- Contract expiration without renewal.
- Customer-requested termination.
- Non-payment termination.
- Mutual migration to another system.
- Legal, compliance, or fraud-driven suspension.
- Tenant consolidation, acquisition, or brand migration.

## Required Owners

- Customer Success owns customer communication, timeline, migration coordination, and final sign-off.
- Support owns open case review, customer-facing status, and final closure confirmation.
- Finance owns final invoices, refunds, credits, disputes, and payment-method closure.
- Legal owns contract obligations, retention terms, data-return terms, deletion rights, and legal holds.
- Engineering owns tenant archival, access disablement, integration shutdown, export generation, and production verification.
- Security owns SSO, SCIM, API credential, webhook, notification, and audit-log review.

## Offboarding Stages

### 1. Intake And Classification

- Record termination reason, effective date, requested data-return date, and contractual notice requirements.
- Identify tenant, parent account, child accounts, brands, dispatch offices, users, carrier/broker relationships, and integrations.
- Classify exit as standard, expedited, legal hold, disputed, or security-sensitive.
- Confirm whether the customer is moving to another platform, winding down operations, or consolidating tenants.

### 2. Operational Freeze Review

- List all open loads, assigned drivers, bids, tenders, invoices, settlements, disputes, claims, documents, messages, tracking sessions, and integrations.
- Decide which workflows can continue until completion and which must be frozen.
- Stop new load creation after the agreed cutoff unless legal/commercial exceptions are approved.
- Keep customer-visible read access only for the agreed support window.

### 3. Financial Closure

- Generate final open AR/AP report.
- Reconcile pending Stripe payments, Connect transfers, refunds, chargebacks, credits, and manual adjustments.
- Confirm final invoice schedule and responsible billing contact.
- Prevent new paid actions after cutoff while preserving evidence for open disputes.

### 4. Data Return Package

The standard export package should include contractually allowed data for:

- Tenant profile, users, roles, offices, equipment, preferences, and configuration.
- Loads, bids, tenders, dispatch events, tracking events, milestones, appointments, and exceptions.
- Carriers, brokers, shippers, facilities, lanes, rates, and operational contacts.
- Invoices, settlements, payment records, credits, refunds, and disputes.
- Uploaded documents, generated PDFs, photos, BOLs, PODs, insurance files, compliance files, and claim documents.
- Audit extracts for authentication, admin actions, data changes, integration events, and sensitive exports.
- Integration configuration, webhook subscriptions, EDI mailbox metadata, API clients, and notification channels where contractually allowed.

Exports must be versioned, checksumed, encrypted in transit, and stored only for the approved delivery window.

### 5. Integration Shutdown

- Disable or rotate API keys.
- Disable outbound webhooks.
- Disable inbound EDI/API ingestion.
- Disable SSO and SCIM provisioning.
- Disable notification channels that could leak future production events.
- Revoke third-party tokens and refresh tokens.
- Confirm no background jobs continue to process terminated tenant data.

### 6. Access Disablement

- Disable customer user login at the agreed time.
- Preserve internal break-glass access only when legally required and audited.
- Remove support impersonation access after the support window closes.
- Confirm terminated tenant cannot access active production data, receive events, or mutate records.

### 7. Retention, Deletion, And Legal Hold

- Apply contract retention terms and regulatory retention requirements.
- Mark tenant data as archived after operational closure.
- Prevent deletion if legal hold, active dispute, chargeback, claim, fraud investigation, or statutory retention applies.
- Document backup-retention behavior and the expected timing for deletion from active systems and backups.
- Record final deletion or retention exception in the audit trail.

### 8. Final Sign-Off

Offboarding is complete only when:

- Customer Success confirms customer communication is complete.
- Support confirms open cases are closed or transferred to post-exit tracking.
- Finance confirms final financial state.
- Legal confirms retention, deletion, and contractual obligations.
- Engineering confirms tenant archival, integration shutdown, and access disablement.
- Security confirms credential and notification shutdown.

## Engineering Requirements

- Tenant lifecycle must support active, suspended, offboarding, archived, legal_hold, and deleted states.
- Runtime jobs, webhooks, notifications, exports, payments, and integrations must check tenant lifecycle state before acting.
- Export generation must be auditable and repeatable.
- Offboarding actions must emit audit events.
- Admin tooling must show offboarding progress and blockers.

## Verification Checklist

- A terminated tenant cannot authenticate.
- A terminated tenant cannot create or update loads.
- A terminated tenant cannot send inbound integration events.
- A terminated tenant cannot receive outbound webhooks, email, SMS, or push notifications.
- Data export contains all approved datasets and excludes disallowed data.
- Open payments, disputes, claims, and legal holds are visible before final closure.
- Retention/deletion status is recorded with owner and timestamp.

## Task Mapping

- `ENT-0008` defines this operating process.
- Later tenant-lifecycle, export, audit, and access-control implementation tasks must reference this document before completion.
