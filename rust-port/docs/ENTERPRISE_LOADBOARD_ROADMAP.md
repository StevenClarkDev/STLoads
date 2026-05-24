# Enterprise Loadboard Roadmap For STLoads Rust Port

Last reviewed: 2026-05-24

This document turns the current `rust-port` workspace into a step-by-step plan for becoming an enterprise-grade logistics loadboard and operating platform.

The current Rust port is already substantial. It has an Axum backend, PostgreSQL migrations, SQLx persistence, shared DTOs, Leptos frontend pages, role-aware auth, load creation, load board, chat/offers, execution tracking, document storage, Stripe escrow, STLOADS/TMS routes, admin screens, IBM Code Engine deployment assets, and smoke-test scripts.

It is not yet enough to call the product enterprise-grade. Enterprise readiness means the platform can support real shippers, carriers, brokers, freight forwarders, internal operators, finance users, compliance teams, and external TMS partners with strong security, auditability, reliability, scale, and operational ergonomics.

## Current Architecture Snapshot

Workspace layout:

- `crates/backend`: Axum API, route groups, document storage, Stripe, email, realtime, TMS workers, runtime config.
- `crates/db`: PostgreSQL migrations and SQLx persistence layer.
- `crates/domain`: Rust enums and domain contracts for auth, dispatch, marketplace, payments, TMS, tracking, and master data.
- `crates/shared`: DTOs shared between backend and frontend.
- `crates/frontend-leptos`: Leptos browser portal.
- `docs`: migration plans, deployment notes, QA checklists, schema notes, and scoreboards.
- `scripts`: IBM hosted smoke tests, Stripe/SMTP/TMS verification, seed data, and migration helpers.

Major implemented surfaces:

- Auth, registration, OTP, password reset, onboarding, KYC upload, admin account review.
- Session tokens and role/permission contracts.
- User/admin shells in Leptos.
- Load board, load profile, load builder, load documents, load history.
- Dispatch desk boards for quote, tender, facility, closeout, and collections.
- Marketplace conversations, messages, offers, read receipts, and presence.
- Execution page with leg actions, GPS pings, live tracking controls, documents, POD guardrail, map handoffs, and notes.
- Admin users, roles, onboarding reviews, loads, payment shortcuts, STLOADS operations, reconciliation, and master data.
- Stripe Connect, PaymentIntent funding, escrow hold/release, webhook verification, and transfer creation.
- STLOADS/TMS handoff lifecycle, webhooks, retry worker, reconciliation worker, and sync error handling.
- IBM Code Engine backend/frontend deployment assets and smoke validation scripts.

Important caveat:

The repo documentation says production cutover is complete, but many feature rows in `docs/MIGRATION_SCOREBOARD.md` are still marked `partial`. Treat the current app as a serious migrated foundation, not an enterprise-complete logistics network.

## Enterprise Target

The finished platform should be more than a CRUD load board. It should be a trusted operating layer for freight transactions.

Target platform capabilities:

- Verified marketplace of shippers, brokers, freight forwarders, and carriers.
- Fast load posting, matching, quoting, tendering, booking, execution, tracking, documentation, billing, and payout.
- Multi-role portals with strict authorization and least-privilege controls.
- Enterprise shipper tools: private networks, lane guides, preferred carrier lists, contract rates, spot auctions, and reporting.
- Carrier tools: capacity profile, equipment availability, compliance wallet, route preferences, mobile execution, document capture, and payouts.
- Broker/operator tools: desk boards, exception queues, escalation, notes, reconciliation, collections, and finance controls.
- TMS/ERP integrations through secure APIs, webhooks, and event streams.
- Compliance and risk controls for KYC/KYB, FMCSA/DOT/MC, insurance, sanctions, documents, payments, and audit trails.
- Production observability: logs, metrics, traces, alerts, SLOs, runbooks, incident response, and compliance reporting.

## Phase 1: Stabilize The Production Foundation

Goal: remove risks that can undermine trust before adding more marketplace features.

### 1.1 Lock Down Runtime Configuration

Current concern:

- `RuntimeConfig` allows broad defaults.
- CORS falls back to `Any` if no allowed origins are configured.
- `AppState::from_env` continues serving fallback data when `DATABASE_URL` is absent or the database connection fails.
- Startup migrations can run from the app process.

Steps:

1. Split `APP_ENV` behavior into `development`, `staging`, and `production`.
2. In production, fail startup if `DATABASE_URL` is missing or invalid.
3. In production, fail startup if `CORS_ALLOWED_ORIGINS` is empty.
4. In production, fail startup if document storage is `local`.
5. In production, fail startup if Stripe, SMTP, object storage, TMS shared secret, and public URLs are placeholders.
6. Move database migrations into a controlled deployment step.
7. Add `/health/live` and `/health/ready` endpoints.
8. Make readiness fail when database, object storage, mail, Stripe, or required workers are not healthy.

Acceptance criteria:

- Production cannot boot with fallback screen data.
- Production cannot boot with permissive CORS.
- Production cannot boot with local document storage.
- Readiness endpoint catches unavailable dependencies before traffic is routed.

### 1.2 Harden Authentication And Session Security

Current concern:

- Session tokens are UUIDs stored and looked up directly in `personal_access_tokens`.
- There is no clear enterprise policy for MFA, device sessions, account lockout, token rotation, or high-risk admin actions.

Steps:

1. Store only hashed access tokens.
2. Use high-entropy token generation with a token prefix for lookup and a secret token body for validation.
3. Add token rotation on login, password change, role change, and privilege change.
4. Add account lockout and rate limiting for login, OTP, password reset, and document routes.
5. Add MFA for admins, finance users, operators, and optionally enterprise customers.
6. Add session/device management screen.
7. Add admin impersonation only if needed, with explicit audit records and strong approvals.
8. Require re-authentication for payments, role changes, document deletion, and account deletion.

Acceptance criteria:

- A leaked database token cannot be used directly.
- Brute-force attempts are throttled.
- Admin and finance actions require stronger authentication.
- Every sensitive auth event writes to an immutable audit trail.

### 1.3 Create A Real Audit/Event Ledger

Current concern:

- The app has `load_history`, `user_history`, TMS event tables, and email outbox, but no single enterprise-grade audit ledger.

Steps:

1. Add `audit_events` table with actor, tenant, entity type, entity id, action, before/after JSON, request id, IP, user agent, and timestamp.
2. Emit audit events from auth, admin, load, document, offer, execution, payment, TMS, and master-data mutations.
3. Add immutable append-only behavior for audit records.
4. Add admin audit search page.
5. Add export for compliance and customer audits.
6. Add correlation IDs across HTTP requests, background jobs, Stripe webhooks, TMS events, and email outbox.

Acceptance criteria:

- Every money, status, document, compliance, and permission change can be reconstructed.
- Operators can answer who changed what, when, from where, and why.

### 1.4 Fix File Security And Document Governance

Current concern:

- Document storage exists and supports IBM COS.
- Local document reads should be hardened against path traversal.
- The product uses mock blockchain/hash metadata in places.
- Document workflow is still broader than simple upload/view.

Steps:

1. Canonicalize local paths and reject reads outside the configured storage root.
2. Add per-document access policy records.
3. Add document categories and required document rules by role, equipment, load type, commodity, and execution status.
4. Add versioning for replacements.
5. Add malware scanning or a scanning hook before documents become available.
6. Add file size limits by document type.
7. Add retention and legal hold rules.
8. Replace mock blockchain proof with real content hash, optional timestamp authority, or remove blockchain wording from user-facing UI.
9. Store MIME sniffing result and reject mismatched extension/content types.
10. Add signed, short-lived download routes if direct object delivery is ever needed.

Acceptance criteria:

- Unauthorized users cannot infer or access private files.
- Replaced documents keep history.
- Required documents are machine-checkable.
- Hash verification reflects actual file content, not mock IDs.

## Phase 2: Finish Core Loadboard Parity

Goal: make the Rust app operationally complete for daily freight work.

### 2.1 Deepen Load Posting

Steps:

1. Finalize multi-leg load creation and edit behavior.
2. Add draft, publish, revise, cancel, archive, and clone flows.
3. Add appointment windows, facility contacts, reference numbers, accessorials, special handling, hazmat, temperature, and commodity constraints.
4. Add contract/private/public visibility per load.
5. Add attachment requirements at posting time.
6. Add validation rules for incomplete or contradictory loads.
7. Add bulk import from CSV/API.
8. Add template lanes for recurring shippers.

Acceptance criteria:

- A shipper can create real enterprise freight without operator cleanup.
- Operators can quickly spot incomplete or risky postings.

### 2.2 Build Carrier Search And Matching

Current concern:

- Carrier preferences exist, but enterprise loadboards need deeper matching.

Steps:

1. Create carrier capacity profiles with equipment, lanes, operating states/provinces, certifications, insurance limits, tracking compliance, and preferred commodities.
2. Add availability calendars.
3. Add geo/radius matching from pickup and delivery.
4. Add lane history and carrier performance scores.
5. Add compliance gating before carriers can book certain loads.
6. Add preferred, blocked, and private-network carrier lists.
7. Add ranking: price, proximity, reliability, compliance, service history, and shipper preference.
8. Add recommendation explanations so operators know why a carrier was suggested.

Acceptance criteria:

- Carriers see relevant loads.
- Shippers/operators can explain why a carrier is eligible or blocked.

### 2.3 Complete Offer, Tender, And Booking Workflows

Steps:

1. Model offer lifecycle as a strict state machine.
2. Add counteroffers, expiration, withdrawal, rejection reason, and re-open logic.
3. Add tender acceptance/decline flow separate from open-market bidding.
4. Add rate confirmation generation.
5. Add booking conflict protection with database transactions.
6. Add idempotency keys for booking and finance actions.
7. Add audit and notifications for every offer transition.
8. Add shipper approval rules for brokers/freight forwarders acting on behalf of customers.

Acceptance criteria:

- Two carriers cannot book the same leg through race conditions.
- Finance and execution cannot start from an ambiguous booking state.

### 2.4 Finish Dispatch Desk Depth

Steps:

1. Define canonical desk queues: quote, tender, facility, in-transit exception, closeout, collections, dispute, reconciliation.
2. Add SLA clocks per desk.
3. Add assignment, owner, priority, due date, and escalation.
4. Add queue filters by customer, region, mode, status, carrier, exception type, and risk.
5. Add internal notes and customer-visible notes as separate concepts.
6. Add bulk actions where safe.
7. Add saved views for operators.
8. Add manager dashboard for backlog, aging, throughput, and exceptions.

Acceptance criteria:

- Operators can run the freight floor from Rust without returning to PHP or ad hoc spreadsheets.

## Phase 3: Enterprise Execution And Tracking

Goal: make execution reliable enough for customer SLAs and payment release.

### 3.1 Make Execution State Machine Strict

Steps:

1. Define leg states and allowed transitions in one domain module.
2. Enforce state transitions in database transactions.
3. Add transition preconditions: booking, escrow, pickup docs, delivery docs, note, tracking freshness, admin override.
4. Add override workflow with reason and audit event.
5. Add customer-visible status projection separate from internal operations status.
6. Add status history with actor and source.
7. Add tests for every allowed and rejected transition.

Acceptance criteria:

- Invalid pickup/delivery/payment sequences are impossible through both UI and API.

### 3.2 Upgrade Tracking To Production-Grade

Steps:

1. Add driver mobile-first tracking UI.
2. Add background location strategy for mobile browsers or native app plan.
3. Add tracking consent and terms.
4. Add freshness rules by load type and customer.
5. Add ETA calculation and delay detection.
6. Add geofence detection for pickup and delivery.
7. Add exception events: late pickup, late delivery, route deviation, stale tracking, detention risk.
8. Add customer tracking share page with limited visibility.
9. Add location data retention policy.

Acceptance criteria:

- Operators and customers can trust tracking freshness.
- Drivers have a simple flow that does not require desktop-style navigation.

### 3.3 Complete Proof Of Delivery And Closeout

Steps:

1. Define required closeout documents by customer, commodity, and mode.
2. Add document checklist on each leg.
3. Add POD validation workflow.
4. Add detention, lumper, accessorial, and claim capture.
5. Add dispute flow before release.
6. Add closeout package export.
7. Add invoice handoff.

Acceptance criteria:

- A delivered load is not financially releasable until proof, exceptions, and billing conditions are satisfied.

## Phase 4: Payments, Finance, And Risk

Goal: make money movement safe, auditable, and scalable.

### 4.1 Harden Stripe And Escrow

Current concern:

- Stripe wiring exists and hosted test validation is documented.
- Enterprise finance needs idempotency, reconciliation, dispute handling, and internal controls.

Steps:

1. Add idempotency keys to PaymentIntent and Transfer creation.
2. Store Stripe event IDs and reject duplicate webhooks.
3. Add payment ledger table separate from `escrows`.
4. Add double-entry-style entries for escrow funded, fee earned, payout held, payout released, refund, dispute, and adjustment.
5. Add manual hold and release approvals.
6. Add payout failure handling.
7. Add refund and dispute workflows.
8. Add finance reconciliation reports by Stripe balance transaction.
9. Add currency/decimal strategy and avoid mixing cents with decimal rates in unclear places.

Acceptance criteria:

- Finance can reconcile every cent between STLoads, Stripe, and bank payout records.
- Duplicate webhooks and repeated button clicks cannot double-release funds.

### 4.2 Add Billing And Invoicing

Steps:

1. Add invoice model for shipper/customer billing.
2. Add carrier settlement model.
3. Add platform fees, broker margin, accessorials, taxes, and adjustments.
4. Add invoice status lifecycle.
5. Add PDF invoice/rate confirmation generation.
6. Add accounting export or integration.
7. Add aging reports and collections queues.

Acceptance criteria:

- The platform can support actual back office accounting, not only escrow button workflows.

## Phase 5: Compliance And Trust Network

Goal: make the marketplace safe enough for enterprise freight.

### 5.1 KYB/KYC And Carrier Compliance

Steps:

1. Split person KYC, company KYB, carrier compliance, broker compliance, and freight-forwarder compliance.
2. Add FMCSA/DOT/MC verification integration or manual verification workflow.
3. Add insurance certificates with expiration tracking.
4. Add W-9/tax document workflow for US payouts.
5. Add sanctions/OFAC screening for companies and beneficial owners.
6. Add compliance status badges and gating rules.
7. Add automated reminders for expiring documents.
8. Add compliance override approvals.

Acceptance criteria:

- A carrier cannot book restricted loads if compliance is expired or missing.
- Admins have clear evidence for every approval.

### 5.2 Fraud And Abuse Controls

Steps:

1. Add risk scores for new accounts, unusual login, new payout account, sudden rate changes, and suspicious document changes.
2. Add hold periods for risky payouts.
3. Add suspicious activity review queue.
4. Add IP/device fingerprinting within privacy/legal limits.
5. Add blocklists for emails, domains, companies, DOT/MC numbers, and payment accounts.
6. Add structured incident notes.

Acceptance criteria:

- High-risk accounts and transactions can be paused before money or freight is lost.

## Phase 6: TMS And External Integrations

Goal: make STLoads a reliable integration partner.

### 6.1 Formalize Public API Contracts

Steps:

1. Publish OpenAPI specs for auth, loads, offers, tracking, documents, webhooks, and TMS handoffs.
2. Add API versioning.
3. Add idempotency for all external writes.
4. Add request signing for partner APIs.
5. Add partner-specific rate limits.
6. Add sandbox/test tenant support.
7. Add webhook retry, dead-letter, replay, and delivery logs.
8. Add integration dashboards for customers.

Acceptance criteria:

- Enterprise customers can integrate without reverse-engineering routes or depending on UI behavior.

### 6.2 Strengthen TMS Sync

Current concern:

- TMS push, queue, requeue, withdraw, close, webhooks, retry, and reconciliation exist.
- Enterprise sync needs stronger conflict, version, and replay controls.

Steps:

1. Add payload schema versions and compatibility tests.
2. Store external event IDs and reject duplicates.
3. Add source-of-truth rules per field.
4. Add conflict resolution workflow when STLoads and TMS disagree.
5. Add replay tools for failed handoffs and webhooks.
6. Add tenant-level integration settings.
7. Add delivery logs with response code, latency, attempt count, and next retry.
8. Add alerting for stale handoffs and repeated sync failures.

Acceptance criteria:

- Operators can diagnose and repair TMS drift without developer database access.

## Phase 7: Enterprise Frontend And UX

Goal: make the product feel like a professional logistics command center.

### 7.1 Design System And Interaction Quality

Current concern:

- Leptos pages exist, but several are large single files with inline styles and mixed concerns.

Steps:

1. Build a shared component library for table, filters, modal, drawer, toast, badge, timeline, file uploader, status pill, map panel, and money controls.
2. Move repeated inline styles into CSS modules or a consistent global design layer.
3. Standardize form validation and error display.
4. Standardize empty, loading, error, and permission-denied states.
5. Add keyboard accessibility and screen-reader checks.
6. Add responsive mobile views for driver/carrier workflows.
7. Split large page modules into smaller components.
8. Add UI regression screenshots for critical screens.

Acceptance criteria:

- Operators can scan dense screens quickly.
- Mobile carrier workflows are usable in the field.
- The UI has consistent behavior across roles.

### 7.2 Role-Specific Dashboards

Steps:

1. Admin dashboard: exceptions, pending reviews, money at risk, stale tracking, TMS drift, failed emails/webhooks.
2. Shipper dashboard: active loads, exceptions, spend, carrier performance, pending documents, invoices.
3. Carrier dashboard: available loads, booked loads, tracking tasks, required documents, payouts.
4. Broker/forwarder dashboard: customer loads, desk queues, margin, compliance, closeout.
5. Finance dashboard: funded escrows, release-ready, payout holds, disputes, reconciliation.

Acceptance criteria:

- Each role lands on the work they actually need to do next.

## Phase 8: Data, Reporting, And Intelligence

Goal: turn operational data into enterprise value.

Steps:

1. Add normalized analytics tables or warehouse export.
2. Define metrics: posted loads, booked loads, acceptance rate, quote-to-book time, tracking compliance, on-time pickup, on-time delivery, document cycle time, margin, payout time, dispute rate.
3. Add report builder or export.
4. Add customer performance dashboards.
5. Add carrier scorecards.
6. Add lane pricing history.
7. Add rate recommendation engine.
8. Add anomaly detection for rates, delays, and fraud.

Acceptance criteria:

- Customers and internal teams can make decisions from the platform, not only transact in it.

## Phase 9: Reliability, Scale, And Operations

Goal: make the platform safe to run continuously.

### 9.1 Observability

Steps:

1. Add structured JSON logs.
2. Add request IDs and trace IDs.
3. Add OpenTelemetry traces for HTTP, SQL, Stripe, COS, SMTP, and TMS calls.
4. Add metrics: request latency, error rate, DB pool usage, queue lag, worker outcomes, email failures, webhook failures, storage errors.
5. Add alerting for P0/P1 conditions.
6. Add dashboards for product, infra, and business operations.

Acceptance criteria:

- Engineers and operators know when the system is unhealthy before customers report it.

### 9.2 Background Jobs And Queues

Current concern:

- TMS and email workers spawn inside the backend process.

Steps:

1. Move long-running workers into separate services or clearly isolated process modes.
2. Add job tables or a queue system with retry, locking, visibility timeout, and dead-letter behavior.
3. Add worker dashboards.
4. Add graceful shutdown.
5. Add backpressure for external dependencies.
6. Add scheduled jobs for compliance expiry, tracking stale checks, payout reconciliation, document retention, and TMS drift.

Acceptance criteria:

- Scaling HTTP traffic does not accidentally multiply unsafe worker activity.

### 9.3 Database Scalability

Steps:

1. Add query plans for load board, chat, tracking, admin queues, TMS reconciliation, and reports.
2. Add indexes for common filters.
3. Add pagination everywhere, with cursor pagination for large datasets.
4. Add read model tables for dashboards if queries become heavy.
5. Add archiving strategy for location pings, messages, audit events, and old handoffs.
6. Add backup/restore drills.
7. Add migration rollback strategy.

Acceptance criteria:

- The app remains fast with millions of messages, tracking points, documents, and audit records.

## Phase 10: Testing And Quality Gates

Goal: stop regressions before production.

Current concern:

- There are route and DB tests, but coverage is not yet broad enough for enterprise freight risk.
- A full `cargo test --workspace` run was attempted during this review and timed out after about 124 seconds, so the test suite needs a documented runtime expectation and likely faster subsets.

Steps:

1. Keep fast unit tests under 60 seconds.
2. Add DB integration tests for each domain state machine.
3. Add API contract tests for every route group.
4. Add browser E2E tests for each role.
5. Add payment tests for duplicate webhooks and duplicate release attempts.
6. Add document access tests for every role.
7. Add TMS idempotency and replay tests.
8. Add load board race-condition tests.
9. Add load/performance tests for board search, chat, tracking writes, and admin queues.
10. Add CI pipeline with formatting, clippy, tests, SQLx checks, security audit, frontend build, and Docker build.

Acceptance criteria:

- Every production deploy passes automated business-critical workflows.
- Manual QA becomes a final confidence pass, not the main safety net.

## Phase 11: Security And Compliance Program

Goal: operate as a trustworthy enterprise vendor.

Steps:

1. Perform threat modeling for auth, payments, documents, TMS, and admin.
2. Add dependency scanning and `cargo audit`.
3. Add secrets scanning.
4. Add security headers and CSP for the frontend.
5. Add CSRF strategy if browser cookies are introduced.
6. Add API rate limiting and bot protection.
7. Add least-privilege service credentials.
8. Encrypt sensitive fields where necessary.
9. Define data classification and retention.
10. Prepare SOC 2-style controls: access reviews, incident response, backups, change management, vendor management, logging, and monitoring.
11. Add privacy policy support for location, documents, and user data.

Acceptance criteria:

- The product can pass enterprise security review and customer procurement checks.

## Phase 12: Multi-Tenancy And Enterprise Accounts

Goal: support real customer organizations, not only individual accounts.

Steps:

1. Add organizations/accounts table.
2. Attach users, loads, documents, payments, and integrations to tenant/org boundaries.
3. Add team membership and role assignments per organization.
4. Add customer-specific settings: required docs, allowed carriers, TMS credentials, rate rules, payment terms.
5. Add enterprise SSO/SAML/OIDC.
6. Add audit and reporting per tenant.
7. Add tenant-level data export.
8. Add hard authorization tests for tenant isolation.

Acceptance criteria:

- One enterprise customer cannot see another customer's loads, documents, users, or events.

## Phase 13: Suggested Implementation Order

This is the recommended sequence from current codebase to enterprise-grade platform.

### Milestone 1: Production Hardening

1. Fail production startup on missing DB, permissive CORS, local storage, and placeholder secrets.
2. Split health into liveness/readiness.
3. Hash session tokens.
4. Add auth/rate limiting for login, OTP, password reset, upload, document view, and finance actions.
5. Add global audit ledger.
6. Harden local file path handling and document type validation.
7. Add CI quality gates.

Exit criteria:

- Security and deploy foundations are safe enough for growth.

### Milestone 2: Operational Parity Closure

1. Finish load-management edge cases.
2. Finish dispatch desk closeout/collections workflows.
3. Finish admin load profile and finance parity.
4. Finish profile/KYC revision workflows.
5. Finish document lifecycle: versioning, delete/replace rules, required docs, audit.
6. Finish acceptance tests for all P0/P1 workflows in the side-by-side QA checklist.

Exit criteria:

- No role needs PHP fallback or developer intervention for normal freight operations.

### Milestone 3: Marketplace Depth

1. Build carrier capacity profiles.
2. Add private networks and preferred/blocked carriers.
3. Add matching/ranking.
4. Add offer expiration, counteroffer, tender, and rate confirmation.
5. Add load templates and recurring lane tools.

Exit criteria:

- The product behaves like a real freight marketplace, not only a dispatch database.

### Milestone 4: Execution And Closeout

1. Centralize execution state machine.
2. Add geofencing and ETA/delay detection.
3. Add mobile-first driver workflow.
4. Add required POD and closeout package generation.
5. Add exception management and customer tracking visibility.

Exit criteria:

- Customers and operators can trust execution visibility and closeout readiness.

### Milestone 5: Finance And Compliance

1. Add payment ledger and Stripe reconciliation.
2. Add invoices, settlements, disputes, refunds, adjustments, and accounting export.
3. Add FMCSA/DOT/MC, insurance, tax, sanctions, and compliance expiry workflows.
4. Add fraud/risk review queues.

Exit criteria:

- Money movement and carrier eligibility are auditable and controlled.

### Milestone 6: Enterprise Integrations

1. Publish OpenAPI specs.
2. Add API versioning and partner auth.
3. Add idempotent external APIs.
4. Add webhook delivery logs and replay.
5. Add TMS tenant settings and conflict resolution.

Exit criteria:

- Enterprise customers can integrate predictably.

### Milestone 7: Scale, Reporting, And Enterprise Administration

1. Add observability dashboards and SLOs.
2. Separate workers from web traffic.
3. Optimize database queries and indexes.
4. Add analytics/reporting.
5. Add organizations, team memberships, tenant isolation, and SSO.
6. Add SOC 2-style control evidence.

Exit criteria:

- The company can sell to larger shippers, brokers, and logistics teams.

## Critical Technical Backlog

These items should be tracked as engineering tickets.

1. Production boot guardrails for DB, CORS, storage, Stripe, SMTP, TMS, and public URLs.
2. Hashed session tokens and token rotation.
3. Rate limiting and account lockout.
4. MFA for admins and finance users.
5. Global audit ledger.
6. Local file path canonicalization.
7. Document versioning, required documents, scanning hook, and real hash verification.
8. Strict domain state machines for offers, booking, execution, escrow, and TMS handoffs.
9. Idempotency keys for booking, payment, TMS, and external APIs.
10. Stripe webhook event de-duplication and payment ledger.
11. Separate worker service/process mode.
12. OpenTelemetry traces, metrics, and alerting.
13. OpenAPI contract generation.
14. Multi-tenant organizations and tenant isolation.
15. CI pipeline with fast test subsets and full integration test lane.

## Critical Product Backlog

These items should be managed as product epics.

1. Carrier network and compliance wallet.
2. Private shipper networks and preferred carriers.
3. Carrier matching and ranking.
4. Tender/counteroffer/rate confirmation workflow.
5. Mobile driver execution workflow.
6. Required closeout/POD package.
7. Billing, invoices, settlements, disputes, and accounting export.
8. Enterprise dashboards by role.
9. TMS/customer integration portal.
10. Reporting and carrier scorecards.

## Team And Process Recommendations

Minimum enterprise delivery team:

- Backend/platform engineer for Rust, SQLx, PostgreSQL, workers, APIs, integrations.
- Frontend engineer for Leptos UX, design system, accessibility, E2E tests.
- Product/operations lead who understands freight workflows.
- QA/automation engineer for role-based workflows, regression, browser tests, and staging validation.
- DevOps/security owner for IBM, observability, secrets, CI/CD, backups, incident response.
- Finance/compliance advisor for escrow, payouts, carrier verification, insurance, tax, and audit requirements.

Working cadence:

1. Convert each milestone into tickets with owner, acceptance criteria, and test plan.
2. Keep `docs/MIGRATION_SCOREBOARD.md` honest: avoid marking `done` while rows still say `partial`.
3. Keep `docs/PHP_RUST_QA_FINDINGS.md` as the defect source of truth until PHP is fully irrelevant.
4. Add automated tests before or with every risky workflow change.
5. Run hosted smoke checks after every staging deploy.
6. Hold weekly operations review with real users until desk, finance, tracking, and admin workflows feel faster than the old system.

## Definition Of Enterprise Ready

The platform is enterprise-ready when all of the following are true:

- Production cannot start in unsafe fallback mode.
- All critical workflows are backed by automated tests and hosted smoke checks.
- Auth, documents, payments, and admin actions are strongly protected and audited.
- Role and tenant isolation are enforced by tests.
- Operators can run load posting, booking, tracking, closeout, finance, TMS sync, and admin support without PHP fallback.
- Shippers and carriers have self-service workflows that reduce operator burden.
- Compliance status is enforceable, visible, and auditable.
- Payments are idempotent, reconcilable, and protected against duplicate release.
- TMS/API integrations are versioned, idempotent, observable, and replayable.
- The system has logs, metrics, traces, alerts, backups, runbooks, and incident response.
- Enterprise customers can receive security, compliance, uptime, and audit answers without custom developer investigation.

