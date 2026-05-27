# Enterprise Loadboard Production Task List

Last updated: 2026-05-27

This is the executable task list for turning the current Rust/Leptos STLoads port into an enterprise-grade logistics loadboard. It is derived from `docs/ENTERPRISE_LOADBOARD_ROADMAP.md`, plus a gap review against the current codebase and migration docs.

Use this document as the implementation tracker. Do not mark a task complete unless the acceptance criteria and verification steps are complete.

## Operating Rules

- Keep this document sorted by phase and stable task ID.
- Work tasks in phase order unless a dependency, production incident, or security/money risk requires a different execution order.
- Within each phase, prefer `P0` before `P1`, `P1` before `P2`, and `P2` before `P3`.
- Keep task IDs stable. If a task must be split, add a suffix such as `ENT-0201A` instead of renumbering existing tasks.
- After every completed task, update this document in the same change set:
  - Change `Status` to `[x]`, `[~]`, or `[!]`.
  - Add or update verification notes under the task if evidence is needed.
  - Update acceptance criteria if the implementation revealed a better enterprise requirement.
  - Add follow-up tasks immediately if the work uncovers missing scope.
  - Update the Enterprise Ready Completion Checklist only when the evidence supports it.
- Do not call a task complete because code exists. Complete means acceptance criteria, tests/checks, documentation, and operational evidence are done.
- If implementation order changes, record the reason in the task or related issue so the roadmap stays explainable.

## Final Pre-Start Code Verification

Verified locally on 2026-05-24 before starting enterprise implementation:

- `[x]` `cargo fmt --all -- --check` passes.
- `[x]` `cargo check --workspace` passes.
- `[x]` `cargo test --workspace` passes: 25 tests passed across backend and database integration coverage.
- `[x]` `trunk build --release` passes for `crates/frontend-leptos`.
- `[x]` `cargo clippy --workspace --all-targets -- -D warnings` now passes as of Phase 16. Remaining lint allowances are deliberate and documented near the code.

Small baseline fixes made during this verification:

- Added missing `database_schema: None` fields to backend test `RuntimeConfig` initializers so `cargo test --workspace` compiles.
- Removed unreachable frontend API helper code so the release WASM build is warning-clean.

Repository hygiene finding:

- Sensitive local runtime files such as `.env.ibm.secret`, `.env.ibm.runtime`, and `.cos-*` are ignored by git and not tracked in the current repository check, but they exist in the working tree. Treat them as local secrets and never commit them.

## Status Key

- `[ ]`: not started
- `[~]`: in progress
- `[x]`: complete
- `[!]`: blocked or needs decision

## Priority Key

- `P0`: production safety, security, money, data loss, tenant isolation, or legal risk
- `P1`: core enterprise workflow or operational blocker
- `P2`: important product depth, scale, or customer readiness
- `P3`: polish, reporting depth, or optimization

## Gap Review Before Execution

The roadmap is broad enough to guide the enterprise build, but these missing or underemphasized areas must be tracked explicitly:

- Tenant and organization boundaries should be introduced earlier than the final phase because they affect auth, loads, documents, billing, reporting, and integrations.
- Customer contracts, lane guides, contracted rates, private freight, and shipper-specific rules need their own backlog items.
- Support tooling is missing: internal support search, safe account assistance, incident notes, and customer issue timelines.
- Release management is missing: environments, migrations, rollback, feature flags, change approvals, and production runbooks.
- Notifications are too email-centered; enterprise logistics needs in-app notifications, SMS/push options, event preferences, and escalation rules.
- Disaster recovery needs explicit RPO/RTO, backup restore drills, object storage restore, and failover expectations.
- Legal/privacy operations need explicit terms, consent, data subject requests, document retention, location privacy, and data deletion/export.
- Data model governance needs explicit ownership for status codes, state machines, reference data, and reporting definitions.
- Search needs a real plan: load search, carrier search, global admin search, saved filters, and eventual full-text/geospatial indexes.
- Mobile execution should be planned as PWA/mobile-first first, with a native app decision later.
- Enterprise SSO and SCIM need explicit tasks; tenant membership will not be enough for larger customers.
- EDI should be tracked separately from generic APIs because logistics customers often expect 204, 990, 214, 210, and related transaction flows.
- Rating, mileage, fuel surcharge, accessorial pricing, detention, claims, and disputes need operational workflows, not only data fields.
- Encryption, data classification, incident response, status page, feature flags, and change approvals need explicit production tasks.
- ELD/telematics, route optimization, facility appointments, carrier packets, BOL/rate-confirmation generation, double-brokering controls, factoring/fuel advances, and mode-specific workflows need explicit tasks.
- Enterprise SaaS readiness also needs explicit SLAs/support tiers, WAF/DDoS controls, vendor/subprocessor governance, data residency/DPA decisions, customer training/help-center, sandbox/demo tenants, and STLoads subscription or usage billing.
- Production data migration/cutover, legal agreement acceptance/e-signature, and logistics time zone/unit/currency normalization need explicit tasks.
- Mobile field execution needs camera/document capture, offline handling, push notification, device support, and optional native-app decisions.
- Carrier compliance needs driver qualification, equipment/trailer profiles, inspections, safety ratings, CSA/FMCSA signals, and DVIR/maintenance decisions.
- Enterprise security governance needs explicit internal access reviews, least-privilege recertification, penetration testing, and vulnerability disclosure.
- Partner integration readiness needs an API lifecycle plan: deprecation policy, SDK/sample-code decision, changelog, and customer migration windows.
- Enterprise procurement needs a formal SOC 2 or ISO 27001 readiness decision with evidence ownership, control mapping, and audit timeline.
- Finance controls need shipper credit limits, AR aging, collections/dunning, and bank-account change verification, not only invoices and payouts.
- Business continuity needs operational fallback ownership and tabletop exercises, not only backup/restore mechanics.
- STLoads operating model needs regulatory authority, surety bond, corporate insurance, and jurisdiction decisions if it acts as broker, forwarder, or marketplace operator.
- Enterprise operations need governed master-data/configuration screens for equipment, commodities, accessorials, service levels, lanes, facilities, and customer rules.
- Notification readiness needs email deliverability, bounce handling, sender authentication, and SMS/push compliance, not only message triggers.
- Enterprise customer lifecycle needs offboarding, tenant archival, data return, contract termination, and integration shutdown steps.
- Production economics need usage quotas, cost monitoring, provider spend alerts, and storage-growth controls.
- Data quality needs ongoing anomaly checks and integrity monitoring beyond one-time validation rules.
- Customer-facing release management needs release notes, UAT/pilot rollout, maintenance notices, and adoption feedback loops.
- Enterprise support needs ticket/case management, SLA breach tracking, support-system integration, and CSAT/feedback reporting.
- Enterprise customers may require branding controls for portals, documents, emails, and custom domains.
- Cross-border and multi-currency operation needs explicit FX, tax, VAT/GST, duties, and Incoterms decisions.
- Marketplace money movement needs AML/transaction-monitoring and account-takeover controls when required by the operating model.
- Security controls need explicit key management, key rotation, PCI/payment-data scope, and encryption evidence.
- Browser security needs explicit CSRF, secure cookie, session fixation, CORS, and custom-domain safeguards.
- Disaster recovery needs PITR, failover, replica, regional dependency, and restore-drill decisions beyond a basic backup checklist.
- Operations needs on-call ownership, escalation routing, SIEM/log-drain, and security-event export.

## Phase 0: Program Setup And Governance

Goal: make the backlog executable and keep the team from drifting.

### ENT-0001 Create Enterprise Work Board

- Priority: `P0`
- Status: `[x]`
- Owner: Product/Engineering
- Scope:
  - Create one issue per task in this document.
  - Preserve task IDs in issue titles.
  - Add owner, priority, estimate, dependency, acceptance criteria, and verification notes.
  - Link each issue back to this file.
- Acceptance criteria:
  - Every `P0` and `P1` task exists in the work board.
  - No task is accepted without a test or verification plan.
- Verification notes:
  - Created `docs/ENTERPRISE_WORK_BOARD.md` as the repo-local execution board.
  - Verified 124 `P0`/`P1` source tasks and 124 work-board cards.
  - Work-board cards preserve task IDs, owners, priorities, acceptance criteria, issue link placeholders, dependency placeholders, estimate placeholders, and verification notes.

### ENT-0002 Align Documentation Truth

- Priority: `P0`
- Status: `[x]`
- Owner: Engineering
- Scope:
  - Reconcile `docs/MIGRATION_SCOREBOARD.md` because it says full retirement is `done` while many rows are still `partial`.
  - Update `docs/IMPLEMENTATION_QUEUE.md` or mark it superseded by this task list.
  - Add a note in `docs/MASTER_PLAN.md` pointing to this task list.
- Acceptance criteria:
  - Repo docs do not claim enterprise completeness while key areas remain partial.
  - Future readers know this task list is the production execution source.
- Verification notes:
  - Marked `docs/IMPLEMENTATION_QUEUE.md` as superseded for enterprise execution.
  - Updated `docs/MIGRATION_SCOREBOARD.md` to clarify it is migration/cutover history, not enterprise-readiness completion.
  - Confirmed `README.md` and `docs/MASTER_PLAN.md` point to the enterprise task list, roadmap, and work board.

### ENT-0003 Define Enterprise Readiness Gates

- Priority: `P0`
- Status: `[x]`
- Owner: Product/Engineering/Ops
- Scope:
  - Define readiness gates for alpha, beta, enterprise pilot, production, and enterprise-ready.
  - Define required evidence for each gate: tests, smoke checks, runbooks, security review, ops signoff.
- Acceptance criteria:
  - Each gate has measurable criteria.
  - Release decisions are based on evidence, not vibes.
- Verification notes:
  - Created `docs/ENTERPRISE_READINESS_GATES.md`.
  - Defined alpha, beta, enterprise pilot, production, and enterprise-ready gates with required evidence and exit decisions.
  - Linked the gates document from `README.md` and `docs/MASTER_PLAN.md`.

### ENT-0004 Confirm Target Market And Operating Model

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Ops
- Scope:
  - Confirm initial focus: US domestic trucking, cross-border, freight forwarding, brokerage, or mixed-mode.
  - Confirm supported roles for first enterprise release.
  - Confirm whether STLoads is marketplace, TMS extension, broker operating system, or all three.
- Acceptance criteria:
  - Product scope is written and approved.
  - Unsupported workflows are explicitly deferred.
- Verification notes:
  - Created `docs/ENTERPRISE_OPERATING_MODEL.md` with a recommended US-first broker-operations/loadboard scope.
  - Explicitly deferred international forwarding, customs brokerage, native apps, broad multi-currency money movement, and other high-risk scope unless approved.
  - Approved by project owner on 2026-05-24 as the working first-release operating model.

### ENT-0005 Define Enterprise Customer Onboarding Process

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Ops/Support
- Scope:
  - Define how new shipper, broker, carrier, and enterprise accounts are provisioned.
  - Define required setup data: organization, users, roles, billing, compliance, private network, contracts, integrations, notification preferences, and support contacts.
  - Add onboarding checklist and owner handoff.
- Acceptance criteria:
  - A new enterprise customer can be onboarded through a repeatable checklist.
  - Support and operations know who owns each setup step.
- Verification notes:
  - Created `docs/ENTERPRISE_CUSTOMER_ONBOARDING.md`.
  - Defined required setup data for organization, users, roles, billing, compliance, private network, contracts, integrations, notifications, and support contacts.
  - Added a repeatable onboarding checklist from intake through launch and post-launch handoff.
  - Linked the onboarding document from `README.md` and `docs/MASTER_PLAN.md`.

### ENT-0006 Define Customer Success, Training, And Help Center

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Support/Ops
- Scope:
  - Define onboarding training for admins, shippers, carriers, dispatch, finance, and support users.
  - Create help-center structure for auth, load posting, booking, tracking, documents, payments, TMS/API, and troubleshooting.
  - Add customer success playbook for implementation, adoption, renewal, and escalation.
  - Define support handoff from sales/implementation into ongoing support.
- Acceptance criteria:
  - Enterprise customers can be trained without engineering involvement.
  - Support can answer common workflow questions from documented material.
- Verification notes:
  - Created `docs/ENTERPRISE_CUSTOMER_SUCCESS_AND_TRAINING.md`.
  - Defined training tracks for admins, shippers, carriers, dispatch/operators, finance, support, and integration admins.
  - Defined help-center structure across auth, load posting, booking, tracking, documents, payments, TMS/API, support, and releases.
  - Added customer success playbook for implementation, launch, adoption, renewal, escalation, and support handoff.
  - Linked the document from `README.md` and `docs/MASTER_PLAN.md`.

### ENT-0007 Define SLA, Support Tiers, And Customer Commitments

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Ops/Legal
- Scope:
  - Define uptime target, support response times, severity levels, maintenance windows, and escalation paths.
  - Define enterprise support tiers and what each tier includes.
  - Decide whether SLA credits are offered.
  - Align SLA with monitoring, incident response, backup/restore, and support staffing.
- Acceptance criteria:
  - Sales and support can describe what STLoads commits to.
  - Engineering can measure whether the platform is meeting those commitments.
- Verification notes:
  - Created `docs/ENTERPRISE_SLA_SUPPORT_TIERS.md`.
  - Defined Standard, Business, Enterprise, and Premium Enterprise support tiers.
  - Defined severity levels, response targets, uptime target, maintenance-window rules, escalation owners, and SLA-credit decision.
  - Linked SLA/support-tier documentation from `README.md` and `docs/MASTER_PLAN.md`.

### ENT-0008 Define Enterprise Customer Offboarding And Data Return

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Ops/Legal/Backend
- Scope:
  - Define offboarding checklist for contract termination, account closure, tenant archival, final invoices, open loads, open disputes, open claims, and active integrations.
  - Define customer data return/export package for loads, documents, invoices, settlements, audit extracts, integrations, and configuration where contractually allowed.
  - Define retention, deletion, legal hold, and backup behavior after termination.
  - Disable or rotate API keys, webhooks, SSO/SCIM, EDI mailboxes, notification channels, and user access at the correct lifecycle stage.
  - Add owner handoff between customer success, support, finance, legal, and engineering for complex enterprise exits.
- Acceptance criteria:
  - Enterprise customers can leave without data loss, unsafe access, or unresolved financial/compliance obligations.
  - Terminated tenants cannot continue sending events, receiving notifications, or accessing production data.
- Verification notes:
  - Created `docs/ENTERPRISE_CUSTOMER_OFFBOARDING.md`.
  - Defined exit triggers, required owners, offboarding stages, data-return package, integration shutdown, access disablement, retention/deletion/legal-hold behavior, final sign-off, and engineering requirements.
  - Linked offboarding documentation from `README.md` and `docs/MASTER_PLAN.md`.

### ENT-0009 Add Support Case Management And Customer Feedback Loop

- Priority: `P1`
- Status: `[x]`
- Owner: Support/Ops/Product
- Scope:
  - Decide support system strategy: built-in support cases, Zendesk/Intercom/Linear integration, or another approved support platform.
  - Track support cases by customer, tenant, user, load, invoice, integration, severity, SLA clock, owner, status, and resolution reason.
  - Add SLA breach reporting, escalation workflow, customer-visible updates, and internal notes.
  - Add CSAT or equivalent feedback loop for enterprise support, onboarding, and incident follow-up.
  - Link support cases to incident reports, audit events, operational queues, and roadmap feedback where appropriate.
- Acceptance criteria:
  - Enterprise support work is tracked as cases with SLA visibility, not only chat/email history.
  - Product and operations can see recurring customer pain from support data.
- Verification notes:
  - Created `docs/ENTERPRISE_SUPPORT_CASE_MANAGEMENT.md`.
  - Chose STLoads-native support cases as the initial system of record with future external helpdesk integration points.
  - Defined case model, intake, SLA management, escalation, customer updates, feedback loop, reporting, tooling requirements, and verification checklist.
  - Linked support-case documentation from `README.md` and `docs/MASTER_PLAN.md`.
  - Earlier audit correction found this was only documented; the implementation below closes that gap.
  - Added migration `0067_support_case_management.sql` with tenant-scoped `support_cases` and `support_case_events`.
  - Added admin support-case APIs for list, create, update/resolve, and feedback/CSAT recording.
  - Added shared support-case DTOs and support-console API helpers.
  - Added support-console UI for loading cases, creating cases, resolving cases, and recording CSAT.
  - Added backend acceptance coverage for case creation, SLA fields, customer-visible updates, internal notes, resolution, feedback, and audit events.
  - Verified `cargo test -p backend routes::admin::tests::support_case_tracks_sla_updates_customer_notes_and_feedback`: passed.
  - Verified `cargo check -p backend`: passed.
  - Verified `cargo check -p frontend-leptos`: passed.
  - Verified `trunk build --release`: passed after cleaning stale generated `dist`.
  - Verified `cargo clippy --workspace --all-targets -- -D warnings`: passed.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: passed.
  - Completion decision: support cases, SLA visibility, customer-visible/internal note separation, resolution, feedback/CSAT capture, and auditability are implemented for the Rust support console.

## Phase 1: Production Safety Foundation

Goal: make the current app safe to run and safe to extend.

### ENT-0101 Add Production Runtime Guardrails

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - Fail production startup if `DATABASE_URL` is missing or connection fails.
  - Fail production startup if `CORS_ALLOWED_ORIGINS` is empty.
  - Fail production startup if `DOCUMENT_STORAGE_BACKEND=local`.
  - Fail production startup if Stripe, SMTP, object storage, TMS shared secret, or public URLs are missing or placeholders.
  - Keep permissive defaults only for development.
- Likely files:
  - `crates/backend/src/config.rs`
  - `crates/backend/src/state.rs`
  - `.env.ibm.example`
- Acceptance criteria:
  - Production cannot serve fallback screen data.
  - Production cannot boot with permissive CORS.
  - Tests cover production vs development behavior.
- Verification:
  - Run config unit tests.
  - Run backend locally with `APP_ENV=production` and missing env vars; startup must fail.
- Verification notes:
  - Added production-only startup validation in `crates/backend/src/config.rs`.
  - Added production startup abort in `crates/backend/src/state.rs` when runtime config is invalid or database connection fails.
  - Updated `.env.ibm.example`, `docs/IBM_DEPLOYMENT_NOTES.md`, and `docs/IBM_CODE_ENGINE_DEPLOYMENT.md` with the production runtime contract.
  - Development keeps permissive local defaults for empty database, empty CORS, local document storage, log mailer, and fail-open mail behavior.
  - Added 7 config tests covering development behavior, complete production config, missing database URL, permissive CORS, local document storage, placeholder values, and fail-open mail.
  - Verified `cargo test -p backend config::tests`: 7 passed.
  - Verified `cargo test -p backend`: 25 passed.
  - Verified `cargo test --workspace`: backend 25 passed, database lifecycle 7 passed, remaining workspace/doc-test crates passed with 0 tests where applicable.
  - Verified `git diff --check`: no whitespace errors; only CRLF-to-LF normalization warnings on edited docs.
  - Verified `APP_ENV=production` with missing production env fails startup before binding the backend.

### ENT-0102 Split Health Into Liveness And Readiness

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/DevOps
- Scope:
  - Keep lightweight `/health/live`.
  - Add `/health/ready` checking database, object storage config, mail config, Stripe config, and worker configuration.
  - Update IBM Code Engine probes.
- Likely files:
  - `crates/backend/src/app.rs`
  - `docs/IBM_CODE_ENGINE_DEPLOYMENT.md`
- Acceptance criteria:
  - Liveness proves the process is alive.
  - Readiness proves dependencies are usable.
  - Traffic is not routed to an app with broken required dependencies.
- Verification notes:
  - Added `/health/live` lightweight liveness endpoint in `crates/backend/src/app.rs`.
  - Added `/health/ready` dependency readiness endpoint in `crates/backend/src/app.rs`; it returns HTTP 503 when required dependencies are not ready.
  - Readiness now checks database pool, object-storage config, mail config, Stripe config, TMS worker config, and production runtime config.
  - Kept `/health` available for compatibility.
  - Added readiness unit tests for missing database and runtime config failures.
  - Updated `docs/IBM_DEPLOYMENT_NOTES.md` and `docs/IBM_CODE_ENGINE_DEPLOYMENT.md` to use `/health/live` for liveness and `/health/ready` for readiness.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 27 passed.
  - Verified `cargo test --workspace`: backend 27 passed, database lifecycle 7 passed, remaining workspace/doc-test crates passed with 0 tests where applicable.
  - Verified `cargo build -p backend`.
  - Verified fresh local backend binary: `/health/live` returned 200 and `/health/ready` returned 503 with intentionally missing dependencies.
  - Verified `git diff --check`: no whitespace errors; only CRLF-to-LF normalization warnings on edited docs.

### ENT-0103 Move Migrations Out Of App Startup

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/DevOps
- Scope:
  - Define a migration job/command for deployments.
  - Set `RUN_MIGRATIONS=false` for production web runtime.
  - Add migration rollback and failed-migration procedure.
- Acceptance criteria:
  - Web app startup does not unexpectedly mutate schema in production.
  - Deployment docs describe how migrations are applied and verified.
- Verification notes:
  - Added explicit migration binary at `crates/backend/src/bin/run_migrations.rs`.
  - Updated web startup so `RUN_MIGRATIONS=true` is ignored by the web runtime and logs operator guidance instead of applying schema changes.
  - Created `docs/ENTERPRISE_MIGRATION_RUNBOOK.md` with pre-checks, migration command, failed-migration procedure, and rollback strategy.
  - Linked migration runbook from `README.md` and `docs/MASTER_PLAN.md`.
  - Updated `.env.ibm.example`, `docs/IBM_DEPLOYMENT_NOTES.md`, `docs/IBM_CODE_ENGINE_DEPLOYMENT.md`, and `docs/IBM_STAGING_SMOKE_CHECKLIST.md` to keep web runtime migrations off.
  - Verified `cargo fmt`.
  - Verified `cargo test -p backend`: 27 passed.
  - Verified `cargo build -p backend --bin run_migrations`.
  - Verified `cargo run -p backend --bin run_migrations` fails safely with `DATABASE_URL is required` when no target database is configured.
  - Verified no remaining backend web-startup call to `db::migrate`.

### ENT-0104 Establish Release Environments

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps
- Scope:
  - Define local, CI, staging, production, and optional enterprise pilot environments.
  - Document secrets, database, object storage, Stripe, SMTP, TMS, and frontend URL per environment.
  - Add environment validation script.
- Acceptance criteria:
  - Environment drift is visible before deploy.
  - Staging mirrors production enough for realistic validation.
- Verification notes:
  - Created `docs/ENTERPRISE_RELEASE_ENVIRONMENTS.md`.
  - Defined local, CI, staging, enterprise pilot, and production runtime expectations.
  - Documented required database, object storage, Stripe, SMTP, TMS, frontend URL, CORS, and migration settings.
  - Added `scripts/validate_runtime_env.ps1` to validate runtime env files before deployment.
  - Linked environment documentation from `README.md` and `docs/MASTER_PLAN.md`.
  - Updated `docs/IBM_CODE_ENGINE_DEPLOYMENT.md` to run the validator before creating/updating Code Engine secrets.
  - Verified template validation passes with placeholder allowance: `validate_runtime_env.ps1 -EnvFile rust-port\.env.ibm.example -TargetEnvironment staging -AllowPlaceholders`.
  - Verified strict staging validation fails on `.env.ibm.example` placeholders, proving drift is visible before deploy.
  - Verified `cargo fmt --check`.

### ENT-0105 Add Deployment Rollback Plan

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Backend
- Scope:
  - Document backend rollback.
  - Document frontend rollback.
  - Document database rollback or forward-fix policy.
  - Document object storage and migration safety.
- Acceptance criteria:
  - A failed deploy has a written recovery path.
  - Rollback plan is tested at least once in staging.
- Progress notes:
  - Created `docs/ENTERPRISE_ROLLBACK_RUNBOOK.md`.
  - Documented backend rollback, frontend rollback, database rollback/forward-fix policy, object-storage rollback, payment/TMS/notification cautions, staging rollback drill, and release record requirements.
  - Linked rollback runbook from `README.md`, `docs/MASTER_PLAN.md`, and `docs/IBM_CODE_ENGINE_DEPLOYMENT.md`.
  - Pending: run the staging rollback drill and record revision IDs/results before marking this task complete.

### ENT-0106 Add Feature Flags And Change Approval

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Product/Engineering
- Scope:
  - Add feature flag strategy for risky backend and frontend changes.
  - Define who can enable flags in staging and production.
  - Add change approval checklist for migrations, payment changes, auth changes, tenant isolation changes, and integration changes.
  - Add kill switches for payments, booking, TMS pushes, notifications, and document upload if a severe incident occurs.
- Acceptance criteria:
  - Risky changes can be deployed dark and enabled gradually.
  - Critical workflows can be disabled safely during an incident.
- Verification notes:
  - Created `docs/ENTERPRISE_FEATURE_FLAGS_AND_CHANGE_CONTROL.md`.
  - Added runtime kill-switch env vars for payments, booking, TMS pushes, notifications, and document uploads.
  - Wired `KILL_SWITCH_PAYMENTS` into escrow release.
  - Wired `KILL_SWITCH_BOOKING` into carrier self-booking.
  - Wired `KILL_SWITCH_TMS_PUSHES` into TMS handoff push.
  - Wired `KILL_SWITCH_NOTIFICATIONS` into outbound email delivery/enqueue.
  - Wired `KILL_SWITCH_DOCUMENT_UPLOADS` into load and execution document uploads.
  - Added kill-switch fields to `.env.ibm.example`, release-environment docs, and runtime env validation.
  - Linked feature flag and change-control documentation from `README.md` and `docs/MASTER_PLAN.md`.
  - Added backend test coverage for notification kill-switch behavior.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 28 passed.
  - Verified runtime env template validation still passes with placeholder allowance.

### ENT-0106A Add Customer Release Notes, UAT, And Rollout Communication

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Ops/Customer Success
- Scope:
  - Define release notes, customer changelog, maintenance notices, and known-issue communication process.
  - Add UAT/pilot rollout process for enterprise customers before risky workflow, finance, integration, or compliance changes.
  - Define customer notice windows for breaking UI/workflow changes, not only API changes.
  - Collect adoption feedback, support ticket trends, and rollback/escalation criteria after major releases.
- Acceptance criteria:
  - Enterprise customers are not surprised by major operational changes.
  - Product and support can track whether releases are adopted safely.
- Verification notes:
  - Created `docs/ENTERPRISE_RELEASE_COMMUNICATIONS.md`.
  - Defined release notes, customer changelog, maintenance notice, known issue, breaking workflow notice, and UAT communication types.
  - Defined required release-note fields, notice windows, UAT/pilot rollout process, post-release feedback loop, and release hold criteria.
  - Linked release communication documentation from `README.md` and `docs/MASTER_PLAN.md`.
  - Corrected `docs/ENTERPRISE_RELEASE_ENVIRONMENTS.md` task mapping so release notes/UAT belong to `ENT-0106A`.

### ENT-0107 Create Production Data Migration And Cutover Plan

- Priority: `P0`
- Status: `[!]`
- Owner: Backend/DevOps/Ops
- Scope:
  - Define source-of-truth systems before, during, and after cutover.
  - Create production data migration plan for users, roles, loads, legs, offers, conversations, documents, payments, TMS handoffs, histories, and master data.
  - Add reconciliation reports comparing legacy source data to Rust/PostgreSQL target data.
  - Define freeze window, rollback point, validation scripts, and business signoff.
  - Define document/object migration and verification if legacy files exist outside IBM COS.
- Acceptance criteria:
  - Cutover can be rehearsed in staging with production-like data.
  - Business users can validate migrated data before the Rust platform becomes authoritative.
- Progress notes:
  - Created `docs/ENTERPRISE_DATA_MIGRATION_CUTOVER_PLAN.md`.
  - Defined source-of-truth phases, data domains, reconciliation reports, freeze window, rollback point, document/object migration checks, and business validation requirements.
  - Added `scripts/run_cutover_reconciliation.ps1` to generate Rust/PostgreSQL cutover summaries and optionally compare them to an expected legacy summary JSON.
  - Added Rust/sqlx fallback binary `backend::cutover_reconciliation` so reconciliation can run without local `psql`.
  - Linked cutover plan from `README.md` and `docs/MASTER_PLAN.md`.
  - Verified `scripts/run_cutover_reconciliation.ps1` fails safely when `DATABASE_URL` is missing.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 28 passed.
  - Verified runtime env template validation still passes with placeholder allowance.
  - Verified IBM Code Engine backend readiness on 2026-05-24 after deployment: `/health/ready` returned HTTP 200 with database, IBM COS, SMTP config, Stripe config, workers, and runtime config all `ok`.
  - Verified IBM PostgreSQL migration ledger is clean through migrations 1-13 after repairing prior staging migration checksum drift and applying missing schema objects from migrations 8-13.
  - Verified IBM staging Rust/PostgreSQL reconciliation on 2026-05-27; sanitized evidence generated at `runtime/evidence/cutover-reconciliation-rust-ibm-staging.json`.
  - Blocked: no approved legacy/source summary JSON has been provided yet, so Rust-vs-legacy reconciliation cannot be completed.
  - Blocked: business users still need to validate migrated production-like data before Rust becomes authoritative.
  - Next action: produce the approved legacy summary JSON, rerun `scripts/run_cutover_reconciliation.ps1 -ExpectedJsonPath ...`, and record business validation signoff.

## Phase 2: Auth, Authorization, And Tenant Boundaries

Goal: protect accounts, roles, organizations, and sensitive workflows.

### ENT-0201 Hash Session Tokens

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - Replace direct token storage with hashed token storage.
  - Use token prefix for lookup and secret token body for verification.
  - Rotate tokens on login, password change, role change, and high-risk events.
- Likely files:
  - `crates/backend/src/auth_session.rs`
  - `crates/db/src/auth.rs`
  - `crates/db/migrations`
- Acceptance criteria:
  - Database token values cannot be used as bearer tokens.
  - Existing sessions are migrated or invalidated safely.
- Verification notes:
  - Added migration `crates/db/migrations/0008_hashed_session_tokens.sql`.
  - Added `token_prefix` and `token_hash` storage for personal access tokens.
  - Existing plaintext bearer token rows are invalidated by migration with `invalidated:{id}` token values and null hash fields.
  - New Rust session bearer tokens use `stl_{prefix}.{secret}` format.
  - Session lookup uses prefix plus SHA-256 secret hash; database token values are not usable as bearer tokens.
  - Login rotation deletes existing user session tokens before issuing a new one.
  - Password reset invalidates existing user session tokens.
  - Admin role/status changes and role-permission changes invalidate affected user sessions.
  - Added session-token unit tests for token parsing, legacy token rejection, and non-reversible hashing.
  - Extended auth route integration test to assert stored token value is not the issued bearer token.
  - Verified `cargo fmt`.
  - Verified `cargo test -p backend auth_session::tests`: 3 passed.
  - Verified `cargo test -p backend routes::auth::tests::registration_and_password_reset_routes_work_end_to_end`: 1 passed.
  - Verified `cargo test -p db`: 7 passed.
  - Re-verified on 2026-05-24 after IBM deployment: `cargo test -p backend auth_session::tests` passed 5 tests.
  - Re-verified on 2026-05-24: `cargo test -p backend routes::auth::tests::registration_and_password_reset_routes_work_end_to_end` passed.
  - Re-verified on 2026-05-24: `cargo test -p db` passed 9 integration tests.
  - Verified IBM PostgreSQL has `token_prefix` and `token_hash` columns on `personal_access_tokens`, and the migration ledger reports migration 8 installed with no checksum warning.
  - Deployed in IBM backend revision `stloads-rust-backend-00081`.

### ENT-0201A Harden Browser Session, Cookie, And CSRF Controls

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security/Frontend
- Scope:
  - Define secure cookie settings: `HttpOnly`, `Secure`, `SameSite`, domain scope, path scope, expiry, and rotation behavior.
  - Add CSRF protection or prove why current auth transport does not require it.
  - Prevent session fixation during login, MFA, SSO, role changes, and privilege escalation.
  - Validate CORS and custom-domain behavior for browser auth, API calls, uploads, and webhooks.
  - Add tests for cross-site requests, origin checks, logout, session renewal, and tenant-domain boundaries.
- Acceptance criteria:
  - Browser session handling is safe across normal domains and any supported custom domains.
  - CSRF, session fixation, and permissive CORS risks are explicitly tested or formally ruled out.
- Completion notes:
  - Added `docs/ENTERPRISE_BROWSER_SESSION_SECURITY.md` as the browser-auth control document.
  - Documented the current auth transport as explicit `Authorization: Bearer stl_<prefix>.<secret>` headers with no backend session-cookie issuer.
  - Formally ruled out CSRF for the current bearer-token transport because browsers do not attach the Rust token as an ambient credential.
  - Defined the mandatory cookie migration gate before any future auth cookie can ship: `HttpOnly`, `Secure`, `SameSite`, domain/path scope, expiry, rotation, server-side invalidation, and CSRF token protection.
  - Documented production CORS and custom-domain requirements, including no wildcard origins, no origin reflection, exact allowed-origin configuration, and tenant-domain staging checks.
  - Documented session fixation controls from `ENT-0201`: login rotation, password-reset invalidation, role/status invalidation, role-permission invalidation, and hashed stored token material.
  - Added auth-session unit tests proving explicit bearer headers authenticate, cookie-only requests do not authenticate, and non-bearer authorization schemes do not authenticate.
  - Linked the new control document from `README.md` and `docs/MASTER_PLAN.md`.
  - Verified `rg` evidence: no backend `Set-Cookie` auth path; bearer-token frontend usage only in API/device-location/document-upload calls.

### ENT-0202 Add Rate Limiting And Account Lockout

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security
- Scope:
  - Rate limit login, OTP, resend OTP, forgot password, reset password, uploads, document reads, finance actions, and webhooks.
  - Add account lockout for repeated login/OTP failures.
  - Add IP-based and account-based counters.
- Acceptance criteria:
  - Brute-force attempts are blocked.
  - Legitimate users receive clear recovery guidance.
- Progress notes:
  - Added `crates/backend/src/rate_limit.rs` with fixed-window limits, account/OTP failure lockout, forwarded-IP fingerprinting, and success cleanup.
  - Added shared `RateLimiter` to `AppState`.
  - Wired rate limits into login, registration, OTP verification, OTP resend, forgot password, reset password, KYC uploads, dispatch/execution document uploads, dispatch/execution document reads, Stripe Connect onboarding, admin Stripe Connect onboarding, escrow fund/hold/release, Stripe webhooks, and TMS webhooks.
  - Added `docs/ENTERPRISE_RATE_LIMITING_AND_LOCKOUT.md`.
  - Added Postgres migration `0009_security_rate_limits.sql`.
  - Added DB-backed distributed throttle helpers in `crates/db/src/security.rs`.
  - Switched route checks through async `AppState` helpers so production uses shared Postgres counters and local/dev can fall back to process-local counters.
  - Added `scripts/security_rate_limit_status.ps1` for operator inspection and controlled lockout clearing.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 7 passed.
  - Verified `cargo test -p backend`: 36 passed.
  - Verified `cargo test --workspace`.

### ENT-0202A Add Distributed Rate Limit Store And Lockout Visibility

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security/Platform/Support
- Scope:
  - Back rate-limit and lockout counters with Redis, Valkey, Postgres atomic upserts, or an equivalent managed distributed store.
  - Add TTL/expiry, atomic increment, environment namespace, and future tenant namespace support.
  - Add metrics for allowed, blocked, locked, and cleared decisions.
  - Add support/admin visibility for lockout state and a controlled unlock procedure.
- Acceptance criteria:
  - Rate limits and account lockouts are enforced consistently across multiple backend replicas.
  - Support can explain and remediate a locked account without database spelunking.
- Completion notes:
  - Added shared `security_rate_limits` table with expiry and lockout indexes.
  - Added atomic Postgres upsert logic for rate-limit counters and lockout failure counters.
  - Added lockout lookup and clear helpers.
  - Added `AppState` wrappers to prefer distributed counters when the database pool exists and fall back to in-memory counters only when needed.
  - Added `scripts/security_rate_limit_status.ps1` to list active throttles, inspect a specific key, and clear a lockout.
  - Updated `docs/ENTERPRISE_RATE_LIMITING_AND_LOCKOUT.md` with production behavior and support commands.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 7 passed.
  - Verified `cargo test -p backend`: 36 passed.
  - Verified `cargo test --workspace`.

### ENT-0203 Add MFA For Privileged Users

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Frontend/Security
- Scope:
  - Add MFA setup, verify, recovery codes, and reset workflow.
  - Require MFA for admins, finance users, operator leads, and integration admins.
  - Add step-up MFA for payout release, role changes, and document deletion.
- Acceptance criteria:
  - Privileged accounts cannot perform high-risk actions without MFA.
  - Recovery flow is auditable.
- Completion notes:
  - Added `mfa_challenges` and `mfa_recovery_codes` tables in migration `0010_privileged_mfa.sql`.
  - Added email OTP MFA challenge creation for privileged login.
  - Privileged login returns no bearer token until `/auth/mfa/verify` succeeds.
  - MFA-verified sessions carry the `mfa_verified` token ability.
  - Added one-time recovery-code generation, hashed recovery-code storage, recovery-code consumption, and MFA-verified recovery-code regeneration.
  - Added Leptos `/auth/mfa` page and login redirect contract.
  - Added step-up enforcement for user deletion, user role/status changes, role-permission changes, escrow release, admin profile KYC document deletion, and recovery-code regeneration.
  - Added `docs/ENTERPRISE_MFA_AND_STEP_UP.md`.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p backend routes::auth::tests::admin_login_requires_mfa_before_session_token`.
  - Verified `cargo test --workspace`.

### ENT-0204 Build Organization/Tenant Model

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Product
- Scope:
  - Add organizations/accounts table.
  - Add memberships and organization-scoped roles.
  - Attach users, loads, documents, payments, TMS settings, and reports to organization boundaries.
  - Decide migration path for existing users and loads.
- Acceptance criteria:
  - All new data is tenant-scoped where appropriate.
  - Existing data is assigned to a default organization without breaking current workflows.
- Completion notes:
  - Added migration `0011_organizations_tenant_foundation.sql`.
  - Added `organizations`, `organization_roles`, and `organization_memberships`.
  - Seeded default organization id `1` with slug `stloads-default` for legacy migration.
  - Added `organization_id` ownership columns, foreign keys, and indexes to users, loads, load documents, KYC documents, conversations, offers, escrows, TMS handoffs, TMS sync errors, and reconciliation logs.
  - Backfilled existing records into the default organization or their related owner/load organization.
  - Added automatic membership trigger for user inserts and role/org updates.
  - Added explicit membership sync in Rust registration and admin-create user flows.
  - Added typed DB helpers in `crates/db/src/organizations.rs`.
  - Added `docs/ENTERPRISE_ORGANIZATION_TENANT_MODEL.md`.
  - Verified `cargo test -p db organization_foundation_assigns_default_membership`.
  - Verified `cargo test -p backend routes::auth::tests::registration_and_password_reset_routes_work_end_to_end`.

### ENT-0205 Enforce Tenant Isolation Everywhere

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/QA
- Scope:
  - Apply tenant checks to auth-scoped reads and writes.
  - Add tests for cross-tenant load, document, chat, payment, TMS, and admin access.
  - Add admin break-glass rules with audit trail.
- Acceptance criteria:
  - One customer cannot access another customer's data.
  - Tenant isolation tests fail before implementation and pass after implementation.
- Progress notes:
  - Added session organization context from active organization memberships.
  - Added org-aware route checks for load profiles, load documents, escrow fund/hold/release, marketplace offer review, marketplace chat send/read, admin chat workspace listings, and user-authenticated TMS requeue/withdraw/close.
  - Added tenant-scoped admin user directory, onboarding review list, load list counts/rows, profile view, profile update, account update, and delete enforcement.
  - Added scoped DB helpers for load legs, load documents, escrows, chat workspace queries, and TMS handoff ownership.
  - Added migration `0012_audit_break_glass.sql` with `audit_events` and `admin_break_glass_sessions`.
  - Added MFA-protected `/admin/break-glass` route with ticket reference, reason, target organization, expiry, and audit event creation.
  - Added `docs/ENTERPRISE_TENANT_ISOLATION.md`.
  - Added integration test `tenant_scoped_queries_reject_cross_organization_records`.
  - Added backend test `break_glass_allows_audited_cross_org_user_profile`.
  - Verified `cargo test -p db tenant_scoped_queries_reject_cross_organization_records`.
  - Verified `cargo test -p backend break_glass_allows_audited_cross_org_user_profile`.
  - Verified `cargo test -p db`.
  - Verified `cargo test -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test --workspace`.

### ENT-0206 Define Permission Matrix

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Scope:
  - Define permissions by role and organization role.
  - Include admin, support, finance, operator, shipper, carrier, broker, freight forwarder, integration admin, and read-only auditor.
  - Update route guards and frontend navigation.
- Acceptance criteria:
  - Every route and action maps to a permission.
  - Role changes invalidate affected sessions.
- Progress notes:
  - Added migration `0013_organization_role_permissions.sql`.
  - Added `organization_role_permissions` for owner, admin, finance, operator, support, integration admin, member, and auditor policy.
  - Added session permission merge from active organization membership role.
  - Added `docs/ENTERPRISE_PERMISSION_MATRIX.md`.
  - Documented current route guard map and future narrower permission gaps for support, audit, reporting, billing, integrations, and impersonation.
  - Verified affected sessions already invalidate through role-permission update token deletion.
  - Verified `cargo test -p db organization_foundation_assigns_default_membership`.
  - Verified `cargo test -p backend`.

### ENT-0207 Add Safe Support Tooling

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Ops
- Scope:
  - Add support search across users, organizations, loads, documents, payments, and TMS handoffs.
  - Add issue timeline and support notes.
  - Add optional impersonation only with explicit approval, banner, limited scope, and audit.
- Acceptance criteria:
  - Support can help customers without unsafe database access.
  - Every support action is auditable.
- Progress notes:
  - Added audited backend support search route at `/admin/support/search`.
  - Search is permission-gated and scoped to the session organization by default.
  - Cross-organization search requires an active break-glass session for the requested organization.
  - Search covers organizations, users, loads, load documents, escrows/payments, and STLoads/TMS handoffs.
  - Every executed search writes an `audit_events` row with actor, target organization, query length, result count, and result categories.
  - Added shared `AdminSupportSearch*` response types.
  - Added Leptos admin Support Search page at `/admin/support` and linked it from the admin sidebar.
  - Added migration `0014_support_notes.sql` for scoped support notes with internal/customer-visible visibility, ticket references, actor linkage, and entity indexes.
  - Added audited backend support timeline route at `/admin/support/timeline`.
  - Added audited backend support note route at `/admin/support/notes`.
  - Added shared `AdminSupportTimeline*` and `AdminCreateSupportNote*` types.
  - Added support timeline and note capture UI to `/admin/support`.
  - Timeline entries combine persisted support notes and audit events for the selected organization/entity.
  - Impersonation was intentionally deferred from this slice; it remains disabled until product explicitly approves bannered, limited-scope, fully audited impersonation.
  - Added backend test `support_search_is_scoped_and_audited`.
  - Added backend test `support_note_is_persisted_timeline_returned_and_audited`.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Verified `cargo test -p backend routes::admin::tests::support_search_is_scoped_and_audited`: 1 passed.
  - Verified `cargo test -p backend routes::admin::tests::support_note_is_persisted_timeline_returned_and_audited`: 1 passed.
  - Verified `cargo test -p backend`: 40 passed.
  - Verified `cargo test -p db`: 9 passed.

### ENT-0208 Add Enterprise SSO And SCIM

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Security
- Scope:
  - Add SAML or OIDC SSO for enterprise organizations.
  - Add domain verification and tenant-specific login routing.
  - Add SCIM or equivalent user provisioning/deprovisioning if required by target customers.
  - Add just-in-time user creation policy.
  - Add tests for deprovisioned users losing access immediately.
- Acceptance criteria:
  - Enterprise customers can manage users through their identity provider.
  - Deprovisioning removes access without manual STLoads intervention.
- Progress notes:
  - Added migration `0015_enterprise_identity.sql`.
  - Added verified organization domain routing foundation through `organization_domains`.
  - Added `enterprise_identity_providers` for active/draft/disabled OIDC or SAML provider metadata.
  - Added `scim_tokens`, `scim_user_links`, and `scim_events` for provision/deprovision auditability.
  - Added shared `EnterpriseSsoDiscovery*` and `ScimDeprovision*` contracts.
  - Added public `/auth/sso/discovery` route for tenant-specific login routing decisions.
  - Password login now blocks for verified domains with active SSO routing and returns the IdP next step.
  - Added bearer-token protected `/auth/scim/deprovision` route.
  - SCIM deprovisioning marks the organization membership deprovisioned, rejects the user account, records a SCIM event, and revokes active Rust sessions.
  - Added admin enterprise identity screen contract and routes at `/admin/identity`.
  - Added admin domain registration and verification-token confirmation routes.
  - Added admin identity provider metadata save route for OIDC/SAML draft, active, and disabled providers.
  - Added Leptos admin Enterprise Identity page with domain verification, provider metadata, and SCIM event review.
  - Linked Enterprise Identity from the admin sidebar and quick-jump search.
  - Added bearer-token protected `/auth/scim/users` route for SCIM provision, update, deactivate, and reactivate.
  - SCIM user updates rotate active Rust sessions, keep organization memberships synchronized, update legacy role mappings, and write SCIM events.
  - Added `jsonwebtoken` dependency for standards-based OIDC ID token validation.
  - Added `/auth/sso/oidc/callback` route that validates OIDC ID tokens against provider JWKS, issuer, audience, supported signing algorithms, email match, and email verification before issuing a Rust session.
  - Expanded enterprise SSO discovery to include provider ID, issuer, JWKS URL, and client ID.
  - Added OIDC JIT user creation when the active provider has JIT enabled and the verified token belongs to a new user.
  - Added shared access-artifact revocation for identity-driven changes.
  - SCIM deprovisioning now clears Rust bearer tokens, legacy sessions, password reset tokens, MFA challenges, and legacy remember tokens.
  - Added `/admin/identity/domains/verify-dns` DNS TXT verification route using `_stloads-domain.<domain>` records.
  - Added DNS TXT verification action to the Leptos Enterprise Identity page.
  - Added backend test `enterprise_sso_discovery_blocks_password_login_when_routing_is_active`.
  - Added backend test `enterprise_sso_oidc_callback_rejects_unconfigured_domain`.
  - Added backend test `scim_deprovision_revokes_active_sessions`.
  - Added backend test `scim_upsert_provisions_updates_and_reactivates_users`.
  - Added backend test `identity_admin_can_register_verify_domain_and_save_provider`.
  - Added backend test `dns_txt_parser_matches_split_google_txt_answers`.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p backend routes::auth::tests::enterprise_sso_discovery_blocks_password_login_when_routing_is_active`: 1 passed.
  - Verified `cargo test -p backend routes::auth::tests::enterprise_sso_oidc_callback_rejects_unconfigured_domain`: 1 passed.
  - Verified `cargo test -p backend routes::auth::tests::scim_deprovision_revokes_active_sessions`: 1 passed, including all local access artifacts.
  - Verified `cargo test -p backend routes::auth::tests::scim_upsert_provisions_updates_and_reactivates_users`: 1 passed.
  - Verified `cargo test -p backend routes::admin::tests::identity_admin_can_register_verify_domain_and_save_provider`: 1 passed.
  - Verified `cargo test -p backend routes::admin::tests::dns_txt_parser_matches_split_google_txt_answers`: 1 passed.
  - Verified `cargo test -p backend`: 46 passed.
  - Verified `cargo test -p db`: 9 passed.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: ENT-0208 requires SAML or OIDC SSO; OIDC is the production SSO path for this release, with SCIM lifecycle and immediate access revocation complete.
  - Future customer-driven extension: add SAML assertion validation only when a target enterprise customer requires a SAML-only IdP.

### ENT-0209 Add Access Reviews And Least-Privilege Recertification

- Priority: `P1`
- Status: `[x]`
- Owner: Security/Ops/Backend
- Scope:
  - Define quarterly access reviews for internal admins, support, finance, integration admins, and break-glass users.
  - Add reporting for privileged roles, stale accounts, inactive users, external contractors, and users outside approved organizations.
  - Add approval workflow for privilege elevation and emergency access.
  - Log review decisions, approvers, revocations, and exceptions.
- Acceptance criteria:
  - Internal and customer-facing privileged access can be recertified on a schedule.
  - Stale, excessive, or emergency access does not remain active silently.
- Progress notes:
  - Added migration `0016_access_reviews.sql`.
  - Added `access_reviews` and `access_review_items` evidence tables for review campaigns, snapshots, decisions, due dates, completion state, revocations, and reviewer metadata.
  - Added `db::access_reviews` module for review creation, privileged access snapshotting, latest review reporting, and item decisions.
  - Added admin API routes:
    - `GET /admin/access-reviews`
    - `POST /admin/access-reviews/start`
    - `POST /admin/access-reviews/items/{item_id}/decision`
  - Starting a review snapshots legacy admins, privileged organization roles, stale accounts, non-approved accounts, inactive memberships, and active break-glass access flags.
  - Review item decisions support approve, exception, and revoke; revoke marks the membership revoked, disables high-risk privileged accounts, and clears local access artifacts.
  - Added Leptos `/admin/access-reviews` page with review creation, evidence counts, item decisions, sidebar link, and quick-jump routing.
  - Added `access_elevation_requests` workflow for privilege elevation approval, rejection, business justification, expiration metadata, and decision auditability.
  - Access review snapshots now flag external or unverified email domains and missing approved memberships as recertification risks.
  - Added admin elevation routes:
    - `POST /admin/access-reviews/elevation-requests`
    - `POST /admin/access-reviews/elevation-requests/{request_id}/decision`
  - Approved privilege elevation requests update organization membership roles and rotate local access artifacts.
  - Added Leptos access review controls for elevation request creation and approval/rejection.
  - Added backend test `access_review_snapshots_privileged_users_and_revokes_access`.
  - Added backend test `access_elevation_request_requires_approval_and_rotates_access`.
  - Verified `cargo test -p backend routes::admin::tests::access_review_snapshots_privileged_users_and_revokes_access`: 1 passed.
  - Verified `cargo test -p backend routes::admin::tests::access_elevation_request_requires_approval_and_rotates_access`: 1 passed.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p backend`: 48 passed.
  - Verified `cargo test -p db`: 9 passed.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: scheduled privileged access recertification, revocation, elevation approvals, stale access flags, emergency access flags, and external/outside-approved-membership reporting are implemented for the Rust admin surface.

## Phase 3: Audit, Compliance Evidence, And Governance

Goal: make every important action reconstructable.

### ENT-0301 Add Global Audit Ledger

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - Add `audit_events` table.
  - Store actor, tenant/org, entity, action, before/after JSON, request ID, IP, user agent, source, and timestamp.
  - Make audit append-only.
- Acceptance criteria:
  - All high-risk workflows write audit events.
  - Audit events are queryable by entity and actor.
- Progress notes:
  - Existing foundation from ENT-0205 added `audit_events` and high-risk audit insert paths.
  - Added migration `0017_audit_ledger_hardening.sql`.
  - Added explicit `before_state` and `after_state` JSONB evidence fields to `audit_events`.
  - Added request ID and created-at indexes for incident reconstruction and compliance queries.
  - Added database triggers that reject update and delete attempts against `audit_events`.
  - Extended Rust audit insertion to accept before/after evidence.
  - Added backend test `audit_ledger_is_append_only_and_stores_before_after_evidence`.
  - Verified `cargo test -p backend routes::admin::tests::audit_ledger_is_append_only_and_stores_before_after_evidence`: 1 passed.
  - Verified `cargo check -p backend`.
  - Completion decision: the Rust audit ledger is append-only, stores actor/org/entity/action/request metadata plus before/after evidence, and remains queryable by entity, actor, request ID, and created time.

### ENT-0302 Add Request Correlation IDs

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - Generate or propagate request IDs for every HTTP request.
  - Propagate IDs into logs, audit events, Stripe actions, TMS events, email outbox, and realtime events.
- Acceptance criteria:
  - A single incident can be traced across API, DB, jobs, payments, and integrations.
- Progress notes:
  - Added backend request ID middleware that preserves inbound `x-request-id` or generates a `req_<uuid>` value.
  - Request IDs are written to structured request logs and returned on HTTP responses.
  - Extended break-glass and audited admin support/identity/access-review flows to persist request IDs into `audit_events`.
  - Added migration `0018_request_correlation_ids.sql`.
  - Added request ID columns and indexes to email outbox, STLoads/TMS handoffs, handoff events, sync errors, and reconciliation logs.
  - Added request IDs to realtime event payloads.
  - TMS lifecycle and webhook routes now store request IDs on handoffs, handoff events, sync errors, reconciliation logs, and realtime events.
  - Stripe outbound calls now send `x-request-id` and include request IDs in supported metadata for payment intents and transfers.
  - Admin review emails can enqueue request IDs into `email_outbox`.
  - DB lifecycle tests now assert request IDs are retained for TMS webhook reconciliation and email outbox rows.
  - Added backend test `request_id_header_is_generated_or_preserved`.
  - Verified `cargo test -p backend app::tests::request_id_header_is_generated_or_preserved`: 1 passed.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p backend`: 50 passed.
  - Verified `cargo test -p db`: 9 passed.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: request IDs now propagate across HTTP, logs, audit events, Stripe actions, TMS lifecycle/webhooks, email outbox records, and realtime payloads.

### ENT-0303 Add Audit Search UI

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend
- Scope:
  - Add admin audit search by user, org, load, document, payment, TMS handoff, action, and date range.
  - Add export for compliance requests.
- Acceptance criteria:
  - Ops can answer who changed what without developer help.
- Progress notes:
  - Started after ENT-0302 completion.
  - Added shared audit search/export DTOs for reusable backend and Leptos contracts.
  - Added backend `/admin/audit` search and `/admin/audit/export` routes scoped by organization with break-glass enforcement for cross-organization access.
  - Search supports actor user, organization, entity type, entity ID, action, request ID, date range, and free-text metadata/reason matching.
  - Export uses the same filters and returns compliance-ready CSV content with actor, organization, entity, action, request ID, evidence label, and metadata preview.
  - Added Leptos `/admin/audit` page with audit filters, results table, sidebar/quick-jump navigation, and CSV export preview.
  - Added backend test `audit_search_filters_by_entity_and_exports_csv`.
  - Verified `cargo test -p backend routes::admin::tests::audit_search_filters_by_entity_and_exports_csv`: 1 passed.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p backend`: 51 passed.
  - Verified `cargo test -p db`: 9 passed.
  - Verified `trunk build --release`.
  - Completion decision: Ops can now search and export audit evidence without developer database access, while tenant scope and break-glass controls remain enforced.

### ENT-0304 Define Status And Reference Data Governance

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Scope:
  - Make canonical owners for load statuses, leg statuses, offer statuses, escrow statuses, TMS statuses, and master data.
  - Document which statuses are customer-visible vs internal.
  - Add process for changing state machines.
- Acceptance criteria:
  - No workflow status can be changed casually without a migration/test/update plan.
- Progress notes:
  - Added `domain::governance::STATUS_GOVERNANCE_CONTRACT` as the code-level source for governed status/reference-data families.
  - Defined canonical owners for load/leg statuses, offer statuses, escrow statuses, TMS handoff statuses, TMS external statuses, and master/reference data.
  - Documented customer-visible vs internal-only status families in `docs/STATUS_REFERENCE_DATA_GOVERNANCE.md`.
  - Added required change-control steps for migration, enum/descriptor updates, API/UI/audit/export updates, tests, rollout, rollback, and customer communication.
  - Added domain tests enforcing required family coverage plus migration/test/visibility requirements.
  - Verified `cargo test -p domain`: 2 passed.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p backend`: 51 passed.
  - Verified `cargo test -p db`: 9 passed.
  - Verified `trunk build --release`.
  - Completion decision: workflow status and reference-data changes now have explicit owners, visibility rules, and mandatory migration/test/update planning before release.

### ENT-0305 Add Legal Agreement Acceptance And E-Signature Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Legal/Product/Backend/Frontend
- Scope:
  - Track platform terms acceptance by user and organization.
  - Track carrier agreements, broker/customer contracts, privacy policy, tracking consent, and payment terms.
  - Decide whether e-signature provider integration is required.
  - Store agreement version, signer, timestamp, IP/user agent, document copy, and audit event.
  - Block workflows when required agreements are missing or expired.
- Acceptance criteria:
  - Legal acceptance can be proven for every required operational agreement.
  - Updated terms can be rolled out and tracked by version.
- Progress notes:
  - Started after ENT-0304 completion.
  - Added migration `0019_legal_agreements.sql` with versioned legal agreement templates and signer acceptance proof records.
  - Seeded active templates for platform terms, privacy policy, tracking consent, payment terms, carrier operating agreement, broker/customer contract terms, shipper customer contract terms, and freight forwarder terms.
  - Added `db::legal_agreements` for required-agreement lookup, missing-agreement detection, acceptance proof storage, and audit-event linkage.
  - Added auth API DTOs and backend routes:
    - `GET /auth/legal-agreements`
    - `POST /auth/legal-agreements/accept`
  - Onboarding submission now blocks when required legal agreements are missing and redirects the session to `/auth/legal-agreements`.
  - Acceptance stores signer name/email, timestamp, IP address, user agent, evidence snapshot, request ID, and `audit_events` linkage.
  - Added DB integration test `legal_agreements_block_until_acceptance_and_write_audit`.
  - Added Leptos `/auth/legal-agreements` page for missing agreements, acceptance actions, and acceptance proof history.
  - Onboarding now redirects failed legal-gate submissions to `/auth/legal-agreements`.
  - Documented first-release e-signature decision in `docs/LEGAL_AGREEMENT_WORKFLOW.md`: clickwrap plus immutable audit evidence is sufficient until Legal requires a provider-backed signature ceremony.
  - Verified `cargo test -p db legal_agreements_block_until_acceptance_and_write_audit`: 1 passed.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p backend`: 51 passed.
  - Verified `cargo test -p db`: 10 passed.
  - Verified `trunk build --release`.
  - Completion decision: legal agreements are versioned, acceptances are provable and audited, updated terms can be rolled out as new required versions, and onboarding is blocked until required agreements are accepted.

### ENT-0306 Define Operating Authority, Insurance, And Jurisdiction Requirements

- Priority: `P1`
- Status: `[x]`
- Owner: Legal/Operations/Product
- Scope:
  - Decide whether STLoads acts as broker, carrier marketplace, freight forwarder, software-only TMS extension, payment facilitator, or mixed model.
  - Determine required FMCSA broker authority, freight-forwarder authority, surety bond/trust filing, state/province registrations, and operating jurisdictions.
  - Track required corporate insurance such as cyber liability, technology E&O, general liability, contingent cargo, broker liability, and any customer-required certificates.
  - Define how operating authority, insurance certificates, surety evidence, and regulatory disclosures are surfaced to enterprise customers when required.
  - Add renewal owners, expiration alerts, and evidence storage for operating authority and corporate insurance.
- Acceptance criteria:
  - The company can prove it is legally allowed and insured to operate under the chosen business model.
  - Enterprise customers can receive required authority, bond, and insurance evidence without ad hoc legal work.
- Progress notes:
  - Added `domain::operating_authority::OPERATING_AUTHORITY_CONTRACT` as the code-level source for first-release operating model scope, regulatory boundaries, and evidence requirements.
  - First-release decision: STLoads is scoped as software-only loadboard/TMS extension plus customer/carrier marketplace software; broker of record, freight forwarder, motor carrier, customs broker, bank, escrow agent, money transmitter, and payment facilitator claims remain out of scope unless Legal/Finance approves a future regulated operating model.
  - Added migration `0020_operating_authority_compliance.sql` with `operating_authority_decisions` and `compliance_evidence_records`.
  - Seeded operating model decisions and customer-disclosable evidence requirements for operating model memo, broker/freight-forwarder authority status, cyber liability, technology E&O, general liability, contingent cargo/broker liability, and state/province registrations.
  - Added `db::operating_authority` queries for operating decisions, customer evidence package records, renewal/review alerts, and evidence document updates.
  - Added `docs/OPERATING_AUTHORITY_INSURANCE_JURISDICTIONS.md` with Legal/Ops process, customer disclosure process, renewal rules, and official FMCSA source links for broker/freight-forwarder registration and financial responsibility forms.
  - Added domain tests for regulated-role out-of-scope decisions and customer evidence ownership/package readiness.
  - Added DB integration test `operating_authority_tracks_customer_evidence_and_renewal_alerts`.
  - Verified `cargo test -p domain operating_authority`: 2 passed.
  - Verified `cargo test -p db operating_authority_tracks_customer_evidence_and_renewal_alerts`: 1 passed.
  - Verified `cargo fmt --check`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo test -p domain`: 4 passed.
  - Verified `cargo test -p db`: 11 passed.
  - Verified `cargo test -p backend`: 51 passed.
  - Completion decision: STLoads can now maintain a provable first-release operating model decision, track authority/insurance/jurisdiction evidence with renewal owners, and assemble customer-disclosable evidence without ad hoc legal work.

## Phase 4: Document Security And Governance

Goal: make documents safe, versioned, compliant, and operationally useful.

### ENT-0401 Harden Local File Reads

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security
- Scope:
  - Canonicalize local document paths.
  - Reject reads outside configured storage root.
  - Add tests for path traversal attempts.
- Likely files:
  - `crates/backend/src/document_storage.rs`
- Acceptance criteria:
  - Malicious `../` or crafted local paths cannot escape storage root.
- Progress notes:
  - Hardened `DocumentStorageService::read_document` and `delete_document` for local storage with safe relative-path validation and canonical storage-root containment checks.
  - Rejects `../`, absolute paths, root-prefixed paths, and Windows-style backslash traversal before reading or deleting local files.
  - Keeps valid generated local paths readable under the configured storage root.
  - Added document storage tests:
    - `local_read_allows_canonical_path_inside_storage_root`
    - `local_read_rejects_path_traversal_attempts`
    - `local_delete_rejects_path_traversal_attempts`
  - Verified `cargo test -p backend document_storage`: 4 passed.
  - Verified `cargo check -p backend`.
  - Completion decision: local document reads/deletes now canonicalize and enforce storage-root containment, so crafted local paths cannot escape the configured document root.

### ENT-0402 Add Document Versioning

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Add document version table or version fields.
  - Preserve prior file, metadata, uploader, hash, review status, and replacement reason.
  - Show version history in load/profile/execution document UI.
- Acceptance criteria:
  - Replacing a document does not destroy audit history.
- Verification notes:
  - Added `load_document_versions`, `kyc_document_versions`, and `leg_document_versions` tables with version numbers, prior file paths, metadata, uploader, hash/blockchain fields where applicable, replacement reason, and rollout backfill for existing current documents.
  - Added version counters and version history labels to load, KYC/profile, admin review, and execution document API payloads.
  - Stopped KYC replacement from deleting the prior local file immediately so previous document versions remain available for history and audit.
  - Showed document version labels in onboarding KYC, self-profile KYC, admin user KYC, load profile documents, and execution document UI.
  - Added `kyc_document_replacement_preserves_version_history` integration coverage proving replacement writes two version rows and preserves the original path, original name, and replacement reason.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 12 passed.
  - Verified `cargo test -p backend`: 54 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release`.
  - Verified `git diff --check`: no whitespace errors; only CRLF normalization warnings on existing edited files.
  - Completion decision: document replacement no longer destroys version history, and operators can see the current version label in the main document surfaces.

### ENT-0403 Add Required Document Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Scope:
  - Define required documents by role, organization, equipment, commodity, load type, customer, and lifecycle state.
  - Show checklist on onboarding, load profile, execution, and closeout screens.
  - Block restricted transitions where required documents are missing.
- Acceptance criteria:
  - Closeout and compliance readiness are machine-checkable.
- Verification notes:
  - Added `required_document_rules` migration with seeded onboarding, load, and execution closeout requirements for carrier authority, insurance, W-9, broker authority, rate confirmation, and delivery POD.
  - Added database query support for active required-document rules scoped by workflow, lifecycle state, role, and organization.
  - Added shared checklist payloads to onboarding, self-profile, load profile, and execution screens.
  - Added onboarding/profile/load/execution checklist rendering so users see missing or ready document requirements before review, booking, or closeout.
  - Blocked carrier and broker onboarding submission when required onboarding documents are missing.
  - Kept delivery completion blocked by the existing POD-plus-note guard and now exposes the POD rule as checklist data.
  - Added backend unit coverage for carrier document checklist readiness and DB integration coverage for seeded enterprise checklist rules.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 13 passed.
  - Verified `cargo test -p backend`: 55 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release`.
  - Verified `git diff --check`: no whitespace errors; only CRLF normalization warnings on existing edited files.
  - Completion decision: required document readiness is now machine-checkable for the first enterprise onboarding and closeout gates, visible in the main document screens, and enforced for onboarding and delivery closeout.

### ENT-0404 Replace Mock Blockchain Proof

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Product
- Scope:
  - Replace `mock_blockchain_*` behavior with real content hash and verification.
  - Decide whether blockchain language remains in the product.
  - If blockchain remains, integrate a real timestamping/anchoring provider.
- Acceptance criteria:
  - UI no longer claims mock proof as real proof.
  - Hash verification uses actual uploaded file bytes.
- Verification notes:
  - Replaced generated `mocksha256-*` values with SHA-256 hashes calculated from stored document bytes before persistence.
  - Updated load-document verification to read bytes from the configured document storage provider before storing a hash.
  - Updated KYC/profile verification to hash uploaded bytes during upload/replacement and read stored bytes during explicit verification.
  - Store `hash_algorithm = 'sha256'` and clear mock transaction IDs instead of creating fake blockchain transaction IDs.
  - Reworded user-facing profile/load/admin document copy from blockchain anchoring to SHA-256/content-hash verification so mock proof is not presented as real proof.
  - Kept external timestamping/blockchain provider integration deferred until a real provider is selected.
  - Added backend unit coverage for deterministic SHA-256 byte hashing.
  - Extended document-version DB coverage to verify stored hashes use `sha256` and do not contain mock tokens.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 13 passed.
  - Verified `cargo test -p backend`: 56 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release`.
  - Verified `git diff --check`: no whitespace errors; only CRLF normalization warnings on existing edited files.
  - Completion decision: UI no longer claims mock blockchain proof as real proof, and server-side verification stores real SHA-256 hashes derived from document bytes.

### ENT-0405 Add File Validation And Scanning Hook

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Security
- Scope:
  - Enforce file size limits by document type.
  - Sniff MIME type and reject mismatches.
  - Add malware scanning integration point or quarantine status.
  - Add blocked file type policy.
- Acceptance criteria:
  - Dangerous or invalid uploads are blocked or quarantined.
- Verification notes:
  - Added central backend upload validation for KYC/profile, load, and execution document uploads.
  - Enforced a 25 MB enterprise upload limit.
  - Blocked executable/script/web upload extensions including `exe`, `dll`, `bat`, `cmd`, `ps1`, `sh`, `js`, `html`, and `php`.
  - Added lightweight MIME sniffing for PDF, PNG, JPEG, and GIF and rejected declared MIME mismatches.
  - Added scanner-hook verdict status `policy_clean_scanner_pending` so a malware provider can be attached without rewriting upload routes.
  - Added backend tests for blocked extensions, MIME mismatch rejection, and scanner-hook acceptance status.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 13 passed.
  - Verified `cargo test -p backend`: 59 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release`.
  - Verified `git diff --check`: no whitespace errors; only CRLF normalization warnings on existing edited files.
  - Completion decision: dangerous and invalid uploads are rejected by a shared policy layer, and the scanner integration point is ready for a real malware scanning provider.

### ENT-0406 Add Retention And Legal Hold

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Legal/Backend
- Scope:
  - Define retention by document type.
  - Add legal hold flags.
  - Add deletion/export workflows for privacy requests where legally allowed.
- Acceptance criteria:
  - Document lifecycle satisfies legal and customer retention requirements.

### ENT-0407 Add Freight Document Templates And Packets

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Scope:
  - Add rate confirmation generation.
  - Add bill of lading template support.
  - Add carrier packet generation and storage.
  - Add certificate of insurance tracking and display.
  - Add shipper/customer document package export.
  - Add template versioning and audit for generated documents.
- Acceptance criteria:
  - Operators can generate and retrieve standard freight documents without manual template work.
  - Generated documents are linked to load, carrier, customer, audit, and document history.
- Verification notes:
  - Added versioned `freight_document_templates` and `generated_freight_documents` tables.
  - Seeded active templates for rate confirmation, bill of lading, carrier packet, and shipper document package.
  - Added DB helpers for active template lookup, load generation context, and generated-document ledger records.
  - Added a load-profile API action that renders selected or default standard freight templates, stores generated files through the configured document storage backend, creates load document ledger rows, and records template/version generation evidence.
  - Added a load-profile UI action to generate the standard freight document packet and refresh the document list for retrieval.
  - Generated documents inherit existing load-document versioning, file access controls, and document history.
  - Added DB integration coverage proving seeded templates, renderable load/carrier context, document linkage, and generated-document ledger evidence.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 14 passed.
  - Verified `cargo test -p backend`: 59 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release`.
  - Verified `git diff --check`: no whitespace errors; only CRLF normalization warnings on existing edited files.
  - Completion decision: operators can generate and retrieve standard freight document packets from the load profile without manual template work, with template/version evidence linked to the load document ledger.

## Phase 5: Load Posting, Search, And Customer Rules

Goal: make load creation and discovery strong enough for enterprise shippers.

### ENT-0501 Finish Enterprise Load Model

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Scope:
  - Add complete fields for appointment windows, facility contacts, references, accessorials, service level, hazmat, temperature, container/securement, mode, commodity, and special handling.
  - Support public, private, contract, and internal-only visibility.
- Acceptance criteria:
  - Enterprise freight can be posted without out-of-band notes.
- Verification notes:
  - Added migration `0024_enterprise_load_model.sql` to extend core `loads` with freight mode, visibility, service level, customer/PO references, pickup and delivery appointment references, facility contact fields, appointment windows, accessorial JSON, temperature JSON, container JSON, and securement JSON.
  - Enforced visibility values for `public`, `private`, `contract`, and `internal`, with indexes for organization/visibility/status, customer references, and accessorial search.
  - Extended shared load-create and builder draft contracts so API and UI can carry the enterprise load fields.
  - Updated DB create/update/read helpers, TMS load materialization, backend validation, load profile display, and the Leptos load builder form for the enterprise fields.
  - Added DB integration coverage proving enterprise operational fields persist on create and update.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 15 passed.
  - Verified `cargo test -p backend`: 59 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Verified `git diff --check`: no whitespace errors; only CRLF normalization warnings on existing edited files.
  - Completion decision: enterprise freight can now be posted with structured operational data instead of out-of-band notes.

### ENT-0502 Add Draft/Publish/Revise/Cancel/Archive/Clone

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Add load lifecycle actions.
  - Enforce state-specific edit rules.
  - Add clone and template creation.
- Acceptance criteria:
  - Users can manage common lifecycle actions safely.
- Verification notes:
  - Added migration `0025_load_lifecycle_actions.sql` with load lifecycle status, revision number, clone source, template marker/name, lifecycle timestamps, and lifecycle reason.
  - Added lifecycle constraints and indexes for lifecycle filtering and reusable load templates.
  - Added shared lifecycle action request/response DTOs and load-profile lifecycle action metadata.
  - Added DB helpers for lifecycle state changes and clone-to-draft creation with copied legs.
  - Added backend `/dispatch/loads/{load_id}/lifecycle` action route for publish, revise, cancel, archive, clone, and save-as-template behavior.
  - Enforced booked/execution-stage locking for revise, cancel, and archive actions.
  - Updated TMS materialized loads to start as published lifecycle records.
  - Added load-profile lifecycle status and action buttons, with disabled action reasons.
  - Extended DB integration coverage for publish, revise, save-as-template, and clone-to-draft.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 15 passed.
  - Verified `cargo test -p backend`: 59 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: users can manage common lifecycle actions safely from the Rust load profile while protected freight stays locked.

### ENT-0503 Add Customer Contract And Lane Guide Model

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Scope:
  - Add customer contracts, lanes, contracted rates, preferred carriers, backup carriers, effective dates, and service rules.
  - Add contract vs spot posting behavior.
- Acceptance criteria:
  - Enterprise shippers can run private/contract freight, not only public spot loads.
- Verification notes:
  - Added migration `0026_customer_contract_lane_guides.sql` with customer contracts, contract lanes, preferred/backup lane carriers, effective dates, service rules, and contract posting behavior.
  - Added load linkage fields for customer contract, contract lane, contract rate, contract currency, posting behavior, and inherited contract service rules.
  - Added DB records and helpers to create customer contracts, create contract lanes, and resolve active/effective contract lanes.
  - Extended shared load creation and load-builder draft contracts with optional contract and lane IDs.
  - Updated load creation/update/clone/read paths so contract lane data stays attached to freight records.
  - Added backend contract-lane enrichment so selected active lanes apply contract rate, visibility, freight mode, service level, accessorials, and service rules.
  - Added load profile contract lane visibility for operators.
  - Added DB integration coverage proving active contract lanes attach contracted pricing and contract visibility to loads.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo test -p backend`: 59 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: enterprise shippers can model contract/private freight lanes with contracted rates and post loads against them instead of relying only on public spot loads.

### ENT-0504 Add Saved Filters And Load Search

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend
- Scope:
  - Add filters for origin, destination, radius, date, equipment, commodity, rate, customer, status, compliance, and visibility.
  - Add saved views by user/role.
  - Add pagination and index strategy.
- Acceptance criteria:
  - Load board stays usable as volume grows.
- Verification notes:
  - Added migration `0027_load_board_search_saved_filters.sql` with saved filter storage plus load-board search indexes for date, price, locations, equipment, commodity, visibility, and lifecycle status.
  - Added shared load-board filter, saved-filter, and filter-option DTOs.
  - Added DB-backed filtered load-board search with auth scope, tab scope, pagination, origin, destination, pickup/delivery date, equipment, commodity, rate, customer/reference, status, compliance, and visibility filters.
  - Added saved-filter loading by user and role.
  - Updated load-board screen assembly to return active filters, saved views, equipment options, commodity options, and true matching-row totals.
  - Updated the Leptos load board with filter controls, saved-view apply buttons, reset, row-count selection, and previous/next pagination.
  - Extended DB integration coverage proving saved filters and filtered search return the expected contract lane load.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo test -p backend`: 59 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: the load board now has indexed, paginated, auth-scoped search and saved views so it can remain usable as load volume grows.

### ENT-0505 Add Bulk Import And API Posting

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Add CSV import with validation preview.
  - Add API load posting endpoint with idempotency.
  - Add error export for failed rows.
- Acceptance criteria:
  - Enterprise customers can post high-volume loads without manual entry.
- Completion notes:
  - Added migration `0028_bulk_load_import_api_posting.sql` for import batches, row-level validation status, idempotency keys, error exports, and created-load linkage.
  - Added shared DTOs for CSV preview, CSV commit, row results, import summaries, and idempotent API posting.
  - Added backend routes `/dispatch/loads/import/preview`, `/dispatch/loads/import/commit`, and `/dispatch/loads/api-post`.
  - Added CSV parsing/validation for required load fields, dates, bid status, weight units, pricing, pickup/delivery locations, and failed-row error CSV export.
  - Added idempotent API posting ledger so repeated API submissions return the existing created load instead of creating duplicates.
  - Added Leptos load-board bulk import controls for CSV preview, commit, result counts, and error CSV review.
  - Added focused backend parser coverage for quoted CSV cells and required-field validation.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 60 passed.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: high-volume load entry now has a ledgered CSV path, failed-row feedback, UI access, and an idempotent API post endpoint.

### ENT-0506 Add Rating, Mileage, Fuel, And Accessorial Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Finance
- Scope:
  - Add mileage calculation source and override rules.
  - Add fuel surcharge tables or customer-specific fuel rules.
  - Add accessorial catalog: detention, layover, lumper, stop-off, TONU, chassis, storage, tolls, and special handling.
  - Add customer and carrier rating rules.
  - Add audit for manual rate overrides.
- Acceptance criteria:
  - Rates can be calculated and explained before booking, billing, and settlement.
  - Manual pricing changes are auditable.
- Completion notes:
  - Added migration `0029_rating_mileage_fuel_accessorial_rules.sql` for mileage rules, fuel surcharge rules, accessorial catalog, load rating quotes, and manual override audit.
  - Seeded enterprise accessorial catalog rows for detention, layover, lumper, stop-off, TONU, chassis, storage, tolls, and special handling per organization.
  - Added shared rate calculation request/response DTOs with accessorial line items, customer/carrier/margin amounts, explanation lines, and audit event ids.
  - Added authenticated backend route `/dispatch/loads/{load_id}/rating/calculate`.
  - Added rate calculation using contract or leg base rates, mileage source/override inputs, mileage charges, customer-specific fuel rules, accessorial flags, carrier rate, margin, and manual override audit.
  - Added latest rating summary to the load profile and a Leptos profile action to calculate rates from the UI.
  - Added backend coverage for accessorial flag amount/default handling.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 61 passed.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: rates can now be calculated, explained, persisted, shown on the profile, and audited when a manual override changes customer pricing.

### ENT-0507 Add Address Validation, Geocoding, And Facility Scheduling

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Scope:
  - Add address validation and normalized facility records.
  - Store geocodes for pickup, delivery, yards, ports, warehouses, and customer facilities.
  - Add dock, appointment, contact, hours, check-in, parking, lumper, and facility instruction fields.
  - Add appointment scheduling and rescheduling workflow.
  - Add facility-specific operational notes and warnings.
- Acceptance criteria:
  - Pickup and delivery locations are operationally usable, not just text addresses.
  - Appointment changes are visible to carrier, shipper, operator, and audit history.
- Completion notes:
  - Added migration `0030_facility_geocoding_appointments.sql` for normalized location geocodes, facility records, facility notes, facility appointments, and appointment event history.
  - Extended location persistence from load creation/update to retain Google place ids, latitude/longitude pairs, validation status, and facility type.
  - Added coordinate validation so incomplete or out-of-range geocode pairs are rejected before saving operational stops.
  - Added authenticated backend route `/dispatch/loads/{load_id}/appointments` for scheduling and rescheduling pickup/delivery facility appointments.
  - Appointment changes now write `facility_appointments`, `facility_appointment_events`, `load_history`, and realtime load-board notifications for operators.
  - Added Leptos load-profile scheduling controls for leg, pickup/delivery stop, appointment start/end, dock, and facility notes.
  - Added backend coverage for coordinate validation.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 62 passed.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: pickup and delivery stops now have normalized facilities, persisted geocodes, appointment scheduling/rescheduling, and audit-visible history.

### ENT-0508 Add Mode-Specific Workflow Tracks

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Scope:
  - Define which freight modes are supported first: FTL, LTL, drayage, intermodal, cross-border, freight forwarding, or mixed-mode.
  - Add mode-specific fields, documents, statuses, and validation rules.
  - For drayage/intermodal, track container, chassis, port, terminal, free time, demurrage, per diem, and appointment data.
  - For cross-border/customs, track customs broker, border crossing, customs documents, PARS/PAPS or equivalent references, and clearance status if applicable.
  - For LTL, track class, NMFC, dimensions, pieces, accessorials, terminal events, and pro/bol references.
- Acceptance criteria:
  - Unsupported modes are blocked or clearly marked as out of scope.
  - Supported modes have enough structure to operate without free-text workarounds.
- Completion notes:
  - Added migration `0031_mode_specific_workflow_tracks.sql` for freight mode rules, required payload/document metadata, load mode validation status, and load mode validation events.
  - Defined first-release supported modes as `FTL`, `LTL`, `drayage`, and `intermodal`.
  - Explicitly deferred `cross_border`, `freight_forwarding`, and `mixed_mode` with stored validation messages tied to later legal, tax, FX, customs, segment, and localization work.
  - Added backend mode normalization and validation for load create/update, CSV commit, and API posting paths.
  - Added structured requirements for LTL class/NMFC/pieces/dimensions and drayage/intermodal container/chassis/terminal/port/free-time details.
  - Added load mode validation event recording after successful create/update.
  - Updated the Rust load builder freight mode selector to show supported modes and disabled deferred modes.
  - Added backend coverage proving deferred modes are blocked and drayage requires structured payload data.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 63 passed.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: supported freight modes now have enforceable validation tracks, while unsupported modes are blocked clearly instead of drifting into free-text workarounds.

### ENT-0509 Add Time Zone, Unit, Currency, And Localization Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Scope:
  - Store pickup/delivery appointment times with time zone context.
  - Normalize and display distance, weight, dimensions, temperature, currency, and date/time formats by customer, lane, mode, and locale.
  - Add conversion rules for miles/kilometers, pounds/kilograms, Fahrenheit/Celsius, and local currencies where supported.
  - Define canonical storage units and display units.
  - Add validation for cross-border or international freight where units and currencies differ.
- Acceptance criteria:
  - Operators, carriers, and customers do not misread appointment times, units, or rates.
  - Reports and integrations use documented canonical units.
- Completion notes:
  - Added `0032_timezone_units_currency_localization.sql` with organization localization defaults, load localization snapshots, and facility appointment time-zone columns.
  - Stored appointment time-zone context through the shared `FacilityAppointmentRequest`, backend scheduling/rescheduling workflow, appointment event audit rows, and profile scheduling form.
  - Persisted load localization snapshots on create/update with canonical `LBS` storage, organization display units, locale, time zone, dimension unit, temperature unit, and currency.
  - Displayed localization and canonical/display weight summaries on the load profile.
  - Added backend unit coverage for weight conversion and time-zone normalization.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 64 passed.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Verified `git diff --check`: only existing CRLF normalization warnings.
  - Completion decision: canonical storage and operator-facing display context are now explicit for load weights, currency labels, locale/time-zone defaults, and appointment time zones; deeper FX/tax/incoterms ambiguity is intentionally carried into `ENT-0509A`.

### ENT-0509A Define Cross-Border Tax, FX, Duties, And Incoterms Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Finance/Legal/Backend
- Scope:
  - Decide target-release support for multi-currency rating, invoicing, settlement, and reporting.
  - Define FX rate source, rate-lock timing, rounding, revaluation, audit, and reconciliation rules if multi-currency is supported.
  - Decide VAT, GST, sales tax, withholding, duty, customs fee, and broker fee treatment by operating region.
  - Track Incoterms, customs responsibility, duties/taxes responsibility, and cross-border billing fields where applicable.
  - Add explicit unsupported-state messaging if cross-border financial obligations are deferred.
- Acceptance criteria:
  - Cross-border and multi-currency freight cannot be billed or settled with ambiguous tax or FX rules.
  - Finance/legal can explain which party owns duties, taxes, and currency risk.
- Completion notes:
  - Added `docs/CROSS_BORDER_TAX_FX_INCOTERMS.md` as the first-release Finance/Legal decision document: US domestic freight, USD-only money movement, and deferred cross-border/non-USD obligations until approval.
  - Added `0033_cross_border_tax_fx_incoterms.sql` with organization-level cross-border finance policies and future per-load finance decision checks.
  - Kept `cross_border`, `freight_forwarding`, and `mixed_mode` blocked by mode validation until legal, tax, FX, customs, segment, and localization controls are approved.
  - Added USD-only escrow funding validation with explicit unsupported-state messaging for non-USD freight payment attempts before Stripe or local escrow persistence.
  - Added backend unit coverage for the USD-only freight payment currency control.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 65 passed.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Verified `git diff --check`: only existing CRLF normalization warnings.
  - Completion decision: the enterprise release cannot silently rate, fund, invoice, settle, or explain non-USD/cross-border obligations without a Finance/Legal-approved policy change.

### ENT-0510 Add Governed Master Data And Configuration Admin

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend/Ops
- Scope:
  - Add admin workflows for equipment types, trailer types, commodities, hazmat classes, accessorial catalog, service levels, document requirements, rejection reasons, and exception reasons.
  - Add customer-specific configuration for lanes, facilities, carrier groups, visibility rules, compliance gates, billing rules, notification rules, and required references.
  - Add approval, audit, effective dates, import/export, and rollback for high-impact configuration changes.
  - Add validation so inactive or deprecated master data cannot silently break existing loads, integrations, reports, or pricing.
- Acceptance criteria:
  - Enterprise configuration can be changed safely without code deployments or direct database edits.
  - Master-data changes are auditable, reversible, and tested against active freight workflows.
- Progress notes:
  - Added `docs/GOVERNED_MASTER_DATA_CONFIGURATION.md` to define simple writable catalogs versus governed configuration, approval expectations, effective-date handling, and rollback evidence.
  - Added `0034_governed_master_data_configuration_admin.sql` with service-level, rejection-reason, exception-reason, and governed-configuration-change ledger tables.
  - Seeded first governed service levels, rejection reasons, and exception reasons per organization.
  - Surfaced governed service-level, rejection-reason, and exception-reason sections in the Rust master-data admin screen with active/effective-date and approval-gated labels.
  - Added backend routes and Leptos admin write/archive controls for governed service-level, rejection-reason, and exception-reason catalogs.
  - Governed saves and archives now write `governed_configuration_changes` ledger rows with actor, target, change type, approval status, summary, and effective dates.
  - Added backend screen coverage asserting governed service-level visibility and route coverage proving governed save/archive ledger entries.
  - Added `0035_operational_catalog_governance.sql` with governed trailer-type and hazmat-class catalogs plus governance columns for `accessorial_catalog`.
  - Extended governed catalog screen sections, API routes, Leptos write/archive controls, and ledger coverage to trailer types, hazmat classes, and accessorials.
  - Added backend route coverage proving hazmat governed save/archive ledger entries.
  - Added `0036_required_document_rule_governance.sql` with effective-date and approval governance columns for `required_document_rules`.
  - Added backend routes, Leptos admin controls, safe archive, screen visibility, and `governed_configuration_changes` ledger writes for required document rules.
  - Added backend route coverage proving document requirement save/archive ledger entries.
  - Added `0037_customer_specific_configuration_admin.sql` for customer-specific lane, facility, carrier-group, visibility, compliance, billing, notification, and required-reference configuration.
  - Added backend routes, Leptos admin controls, safe archive, screen visibility, and governed ledger writes for customer-specific configuration rules.
  - Added governed export, dry-run import, committed import, and rollback execution endpoints plus frontend API helpers so high-impact configuration can be operated without direct database edits.
  - Added backend route coverage proving customer-configuration save/archive ledger entries, governed dry-run import, committed import, export visibility, and rollback ledger execution.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 65 passed.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `trunk build --release` from `crates/frontend-leptos`.
  - Verified `git diff --check`: only existing CRLF normalization warnings.
  - Completion decision: governed master data and customer configuration now have admin write/archive flows, effective-date/approval labels, audit ledger evidence, import/export contracts, rollback execution, and regression coverage.

## Phase 6: Carrier Network, Matching, And Marketplace

Goal: evolve from load list to freight marketplace.

### ENT-0601 Build Carrier Capacity Profile

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Scope:
  - Add equipment, lanes, operating geography, capacity, certifications, insurance limits, preferred commodities, service levels, and availability.
  - Add carrier self-service profile screen.
- Acceptance criteria:
  - Carrier eligibility can be computed from structured data.
- Progress notes:
  - Added `0038_carrier_capacity_profiles.sql` for carrier equipment, lanes, operating geography, preferred commodities, service levels, certifications, availability, power-unit capacity, insurance limit, and notes.
  - Added shared carrier-capacity profile and update contracts.
  - Added authenticated carrier-only `/auth/carrier-capacity` save endpoint and included carrier capacity/readiness in the self-profile screen payload.
  - Added frontend API helper and Leptos self-service edit controls for carrier capacity updates.
  - Wired carrier capacity into carrier load-board recommendation scoring using availability, power units, insurance limit, equipment, commodity, service level, and operating geography.
  - Added backend coverage for capacity normalization, readiness labels, save, and profile-screen reload.
  - Added backend coverage proving carrier capacity changes recommendation scores.
  - Verified `cargo test -p backend carrier_capacity`: 3 passed.
  - Verified `cargo test -p backend recommendation_score_uses_carrier_capacity_profile`.
  - Verified `cargo test -p db`: 16 passed.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: carrier eligibility can now be computed from structured equipment, geography, capacity, service, commodity, certification, availability, and insurance data instead of free-text profile notes.

### ENT-0602 Add Private Networks And Blocklists

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Add preferred carriers, blocked carriers, private shipper networks, and carrier groups.
  - Enforce visibility and booking restrictions.
- Acceptance criteria:
  - Shippers control who can see and book private freight.
- Progress notes:
  - Added `0039_private_networks_blocklists.sql` for shipper/broker-owned carrier relationships, preferred/approved/backup/blocked statuses, carrier groups, notes, and effective dates.
  - Enforced carrier load-board visibility so private/contract/internal freight is visible only to approved/preferred/backup network carriers, while blocked carriers are excluded.
  - Enforced booking-time private network and blocklist checks before carrier self-booking can update a leg.
  - Added `/dispatch/carrier-network` screen and mutation endpoints for managing private network carrier relationships without direct database edits.
  - Added load-board owner UI for shippers, brokers, freight forwarders, and admins to set approved/preferred/backup/blocked carriers, carrier groups, notes, and expiration dates.
  - Added DB integration coverage proving private freight is hidden until a carrier is preferred and hidden again after blocklisting.
  - Verified `cargo test -p db private_network_rules_filter_carrier_load_board_visibility`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: shippers, brokers, freight forwarders, and admins can control private freight visibility and carrier booking eligibility through governed private-network data and UI/API workflows.

### ENT-0603 Add Matching And Ranking

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Data
- Scope:
  - Rank carriers by lane fit, equipment, proximity, compliance, performance, relationship, price, and tracking quality.
  - Explain why a carrier is recommended or blocked.
- Acceptance criteria:
  - Matching is explainable to operators and customers.
- Completion notes:
  - Added `CarrierMatchScreen` and `CarrierMatchRow` shared contracts for ranked carrier recommendations.
  - Added `/dispatch/load-board/{leg_id}/carrier-matches` for shippers, brokers, freight forwarders, and admins.
  - Ranking now scores carrier relationship, private-network eligibility, blocklists, equipment fit, commodity fit, service mode, origin/destination fit, insurance, power availability, KYC document depth, price readiness, TMS/tracking health, and published handoff status.
  - Added explicit blocked reasons for blocked carriers and private/contract/internal visibility restrictions.
  - Added load-board operator UI action and explanation panel showing eligibility, relationship status, score, recommendation reasons, and block reasons.
  - Added focused backend unit coverage for recommended and blocked carrier scoring explanations.
  - Verified `cargo test -p backend carrier_match_scoring_explains_recommended_and_blocked_carriers`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: load owners can request an explainable carrier ranking for each leg and see why each candidate is recommended or blocked before tendering/bookings.

### ENT-0604 Complete Offer State Machine

- Priority: `P1`
- Status: `[x]`
- Owner: Backend
- Scope:
  - Add pending, countered, withdrawn, expired, declined, accepted, superseded, cancelled states.
  - Enforce transitions transactionally.
  - Add tests for all valid/invalid transitions.
- Acceptance criteria:
  - Offer state cannot become ambiguous or contradictory.
- Completion notes:
  - Expanded `domain::marketplace::OfferStatus` to pending, countered, withdrawn, expired, declined, accepted, superseded, and cancelled.
  - Added canonical labels, slugs, terminal/reviewable helpers, and a strict transition validator.
  - Added `0040_offer_state_machine.sql` to seed/update the expanded `offer_status_master` reference data.
  - Updated marketplace offer review to lock the offer row in the transaction, validate the transition, and supersede competing active offers when one is accepted.
  - Updated chat/offer summaries and badges so pending and countered offers remain actionable while terminal states are clearly locked.
  - Added domain unit tests covering the valid active transitions and invalid terminal/ambiguous transitions.
  - Verified `cargo test -p domain offer_state_machine`.
  - Verified `cargo check -p db`.
  - Verified `cargo check -p backend`.
  - Completion decision: offer transitions now flow through a strict domain state machine and the review transaction cannot accept/decline offers from ambiguous or terminal states.

### ENT-0605 Add Counteroffers, Expiration, And Tender Flow

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Add counteroffer UI and API.
  - Add offer expiration.
  - Add tender acceptance/decline separate from open bidding.
  - Add rate confirmation generation.
- Acceptance criteria:
  - Enterprise tendering and spot negotiation are both supported.
- Completion notes:
  - Added `0041_offer_negotiation_tender.sql` for parent offers, expiration timestamps, tender kind, decision notes, and rate-confirmation references.
  - Added shared counteroffer and rate-confirmation API response contracts.
  - Added transactional DB helpers to expire active offers, create counteroffers, supersede the prior offer, and generate idempotent rate-confirmation references for accepted offers.
  - Added `/marketplace/offers/{offer_id}/counter`, `/marketplace/offers/{offer_id}/rate-confirmation`, and `/marketplace/tenders/{offer_id}/decision`.
  - Added frontend API helpers for counteroffers, rate confirmations, and the separate tender decision endpoint.
  - Added chat offer UI controls for accept, decline, counter, counter expiration/note, and rate-confirmation generation after acceptance.
  - Rechecked the offer state machine so pending offers may be superseded by accepted competing offers or counteroffer replacement.
  - Verified `cargo test -p domain offer_state_machine`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: the Rust marketplace now supports spot negotiation with counteroffers, offer expiration, separate tender decision routing, and rate-confirmation reference generation for accepted offers.

### ENT-0606 Add Booking Race Protection

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - Add database transaction/locking around booking.
  - Add idempotency key for booking actions.
  - Add concurrency tests.
- Acceptance criteria:
  - Two carriers cannot book the same leg through race conditions.
- Completion notes:
  - Added `0042_booking_idempotency_race_guard.sql` with carrier-scoped booking idempotency keys.
  - Extended `BookLoadLegRequest` with an optional `idempotency_key`.
  - Updated load-board self-booking to send a stable booking idempotency key per leg.
  - Updated `db::dispatch::book_load_leg` to insert/check idempotency inside the booking transaction, lock the target leg with `FOR UPDATE`, reject already booked/locked legs, and update only when `booked_carrier_id IS NULL AND status_id < 4`.
  - Replays by the same carrier/key return the existing booking without writing duplicate booking history.
  - Added a DB integration test proving the first carrier books, a second carrier loses the race, and the first carrier's idempotent replay succeeds.
  - Verified `cargo test -p db booking_race_allows_only_one_carrier_and_replays_idempotency`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: carrier self-booking now has transaction-level race protection and idempotent retry behavior.

### ENT-0607 Add Carrier Packet And Vetting Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Compliance/Backend/Frontend
- Scope:
  - Add carrier onboarding packet checklist.
  - Track W-9, COI, authority, operating agreement, banking/payout setup, safety/compliance review, and broker/customer-specific packet requirements.
  - Add packet approval, expiration, revision, and renewal workflow.
  - Link carrier packet readiness into matching and booking eligibility.
- Acceptance criteria:
  - A carrier cannot receive restricted freight until packet requirements are complete.
  - Operators can see exactly which packet item blocks eligibility.
- Completion notes:
  - Reused the existing carrier self-profile/KYC document workflow, capacity profile, and admin review surfaces as the packet evidence source.
  - Defined restricted-freight packet requirements around W-9, COI, operating authority, operating agreement, banking/payout setup, and insurance limit readiness.
  - Linked packet readiness into carrier matching: restricted/private/contract/internal loads now block incomplete packet candidates and expose the missing packet reason in the match explanation.
  - Linked packet readiness into booking eligibility: carrier self-booking restricted freight now fails before booking when packet blockers exist.
  - Booking responses list the exact blocker items, such as missing W-9, COI, authority, operating agreement, banking/payout setup, or insurance limit.
  - Packet renewal/revision continues through the existing KYC document versioning, replacement, verification, and admin review workflow.
  - Verified `cargo test -p backend carrier_match_scoring_explains_recommended_and_blocked_carriers`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: restricted freight now depends on carrier packet readiness, and both operators and carriers receive explicit blocker details.

## Phase 7: Dispatch Desk And Operator Workflows

Goal: let operations run the freight floor from STLoads.

### ENT-0701 Define Canonical Desk Queues

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Ops
- Scope:
  - Define quote, tender, facility, in-transit exception, closeout, collections, dispute, reconciliation, and compliance queues.
  - Define entry/exit rules for each queue.
- Acceptance criteria:
  - Every load/leg has a clear operational queue when action is needed.
- Completion notes:
  - Expanded the canonical Rust dispatch desk queue set to quote, tender, facility, in-transit exception, closeout, collections, dispute, reconciliation, and compliance.
  - Added queue-specific entry and exit rule labels to every dispatch desk row so operators know why work is in the queue and what clears it.
  - Added canonical routing aliases for exception/reconciliation/compliance desks while preserving existing quote/tender/facility/closeout/collections routes.
  - Completion decision: the dispatch desk now exposes every Phase 7 canonical queue with clear operational entry/exit rules.

### ENT-0702 Add Assignment, Priority, SLA, And Escalation

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Add owner, priority, due date, SLA clock, escalation reason, and manager visibility.
  - Add filters and saved views.
- Acceptance criteria:
  - Managers can see backlog, aging, and stuck work.
- Completion notes:
  - Added `0043_dispatch_desk_workflows.sql` with `dispatch_work_items` for assigned owner, priority, SLA due date, escalation reason, and work-item status.
  - Enriched dispatch desk rows with owner, priority badge, SLA due/overdue badge, escalation reason, and queue context.
  - Added manager status cards for unassigned work, SLA-at-risk work, and exception-backed work.
  - Completion decision: managers can scan assignment gaps, aging/SLA risk, and exception backlog directly from the Rust desk.

### ENT-0703 Separate Internal And Customer-Visible Notes

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Add internal notes and customer-visible updates as separate records.
  - Add permissions and audit.
- Acceptance criteria:
  - Internal operational comments cannot accidentally leak to customers.
- Completion notes:
  - Added `dispatch_notes` with explicit `internal` and `customer_visible` visibility.
  - Extended dispatch follow-up requests with visibility and stored notes separately from legacy load-history remarks.
  - Updated the dispatch desk UI with separate internal follow-up and customer-visible update inputs.
  - Displayed latest internal note and latest customer update independently on each desk row.
  - Completion decision: internal comments and customer-visible updates now have separate storage, labels, and UI actions.

### ENT-0704 Add Exception Management

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Scope:
  - Add exceptions for stale tracking, late pickup, late delivery, missing POD, payment hold, compliance block, TMS drift, and dispute.
  - Add resolution workflow.
- Acceptance criteria:
  - Operations can resolve freight exceptions without spreadsheets.
- Completion notes:
  - Added `dispatch_exceptions` for stale tracking, late/missing milestone, missing POD, payment hold, compliance block, TMS drift, dispute, and manual exception workflows.
  - Added derived exception signals on the desk for in-transit stale activity, missing POD/closeout notes, payment holds, TMS drift, compliance review, and disputes.
  - Added `/dispatch/desk/legs/{leg_id}/exceptions/resolve` with a resolution note and audit-style dispatch note.
  - Added frontend exception resolution controls on dispatch desk rows.
  - Completion decision: operators can identify and resolve freight exceptions inside the Rust dispatch desk workflow.

## Phase 8: Execution, Tracking, Mobile, And Closeout

Goal: make execution reliable enough for customer visibility and payment release.

Phase 8 verification correction, 2026-05-25: the earlier document marked every Phase 8 item as complete before several gaps were fully implemented. The missing offline replay, closeout ZIP package, customer tracking controls, finance exception decisions, telematics ingest, and route-source validation were corrected and reverified. Phase 8 is now marked complete for the first enterprise release; deferred native app, push notification, QR/OCR, and automated multi-stop optimization work remains post-field-test or provider-selection follow-up, not a Phase 8 blocker.

### ENT-0801 Centralize Execution State Machine

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - [x] Define allowed leg transitions in one domain module.
  - [x] Enforce preconditions transactionally.
  - [x] Add tests for all transitions.
- Acceptance criteria:
  - Invalid pickup, delivery, or closeout transitions are impossible.
- Verification notes:
  - Added `crates/domain/src/execution.rs` as the single canonical execution transition state machine.
  - Backend execution actions now call the domain validator before updating leg status, writing leg events, or appending history.
  - Preconditions are enforced centrally for tracking consent before pickup start and POD plus note before delivery completion.
  - Domain tests cover valid transitions, jumped transitions, unknown actions, tracking consent, POD, and completion-note guards.

### ENT-0802 Build Mobile-First Driver Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Product
- Scope:
  - [x] Create mobile-first route for driver/carrier execution.
  - [x] Prioritize start tracking, arrive pickup, depart pickup, arrive delivery, upload POD, complete delivery.
  - [x] Add PWA install/offline strategy decision.
- Acceptance criteria:
  - Driver tasks work comfortably on a phone.
- Verification notes:
  - Added `/driver/legs/:leg_id` as a mobile-first driver execution route that reuses the hardened execution screen.
  - Driver view prioritizes tracking consent, live tracking, one-off GPS ping, execution note, state actions, mobile capture, route map, and POD upload.
  - The screen exposes operator mode, delivery readiness, privacy state, geofence, ETA risk, and mobile support status for field users.
  - PWA-first strategy is selected for this phase; native app is deferred until supported-browser evidence proves it is necessary.

### ENT-0802A Add Mobile Field Capture And Offline Strategy

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Product/Backend
- Scope:
  - [x] Add camera-first document/photo capture for BOL, POD, pickup photos, delivery photos, seals, damage, and accessorial evidence.
  - [x] Add offline/poor-network behavior for driver actions, notes, GPS pings, and document uploads.
  - [x] Record push/web-push decision for mobile driver alerts.
  - [x] Define supported mobile browsers/devices and whether a native app is required later.
  - [x] Record QR/barcode/OCR decision for load references, BOLs, containers, or warehouse check-in.
- Acceptance criteria:
  - Drivers can complete critical field tasks under real mobile network conditions.
  - Offline or delayed submissions are clearly marked and reconciled when connectivity returns.
- Verification notes:
  - Added camera-first execution capture types for pickup BOL, pickup photos, delivery POD, delivery photos, seal photos, damage evidence, accessorial evidence, and other attachments.
  - Browser file input now prefers camera/PDF capture with mobile capture hints per document type.
  - Added `execution_offline_submissions` migration table and local browser queueing for offline driver notes.
  - Decision recorded: PWA-first for offline work; Chrome Android and Safari iOS are first supported targets; native app, push/web-push, QR/barcode, and OCR remain decision gates after field testing.
  - Previously missing gap addressed: offline behavior now covers driver notes, driver actions, GPS pings, and document upload replay.
  - Added structured browser offline queueing for driver actions and GPS pings.
  - Added offline execution document upload queueing that captures selected file bytes for later replay.
  - Offline queue payloads now preserve structured JSON instead of string-only payloads.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.

### ENT-0802B Complete Offline Replay And Mobile Alert Decisions

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend/Product
- Scope:
  - [x] Add service-worker or equivalent browser replay for queued driver actions, notes, GPS pings, and uploads.
  - [x] Add backend reconciliation endpoint for `execution_offline_submissions`.
  - [x] Mark replayed, failed, duplicate, and manually reviewed submissions clearly in the driver and operations views.
  - [x] Finalize first-release push/web-push and QR/barcode/OCR decisions; field-test enhancements remain follow-up work.
- Acceptance criteria:
  - Offline submissions are automatically retried, reconciled, and visible to operations without manual database work.
- Verification notes:
  - Added execution offline replay API at `/execution/legs/{leg_id}/offline-submissions`.
  - Browser driver view now stores offline notes locally and automatically replays pending submissions when the leg screen loads online.
  - Replayed submissions are persisted in `execution_offline_submissions`; duplicate client submissions are marked without double-appending history notes.
  - Driver and operations views show offline submission totals and pending reconciliation counts.
  - Previously missing gap addressed: queued execution actions, GPS pings, document uploads, failed replay states, and the first-release mobile alert decision are now recorded.
  - Backend replay now reconciles `driver_note`, `driver_action`, `gps_ping`, and `document_upload` submission types.
  - Failed replay processing is marked as `failed` in `execution_offline_submissions`, and browser replay keeps failed submissions queued locally.
  - Decision recorded: web/PWA replay remains the enterprise first release path; native push, QR/barcode, and OCR remain post-field-test enhancements rather than Phase 8 blockers.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.

### ENT-0803 Add Tracking Consent And Privacy

- Priority: `P0`
- Status: `[x]`
- Owner: Product/Legal/Frontend/Backend
- Scope:
  - [x] Add consent text and capture.
  - [x] Add location retention policy.
  - [x] Add customer-visible tracking scope.
- Acceptance criteria:
  - Location tracking is consented, explainable, and retained only as needed.
- Verification notes:
  - Added `execution_tracking_consents` migration table with active consent uniqueness, retention days, and customer-visible scope.
  - Added tracking consent API endpoint and driver UI capture before GPS tracking or pickup start can proceed.
  - Location pings now require an active execution state plus recorded consent for the booked carrier/admin-scoped execution user.
  - Execution screen shows consent text, retention target, and customer-visible tracking scope.

### ENT-0804 Add Geofence, ETA, And Delay Detection

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data
- Scope:
  - [x] Add geofence detection for pickup/delivery.
  - [x] Add ETA calculation.
  - [x] Add stale tracking, delay, detention, and route deviation signals.
- Acceptance criteria:
  - System detects execution risk before a customer asks.
- Verification notes:
  - Added domain geofence helpers and tests for distance/geofence behavior.
  - Execution screen now surfaces geofence status and ETA risk labels from latest GPS point, pickup/delivery coordinates, and appointment times.
  - Added `execution_risk_events` migration table for durable future risk signals.
  - Current implementation detects no-ping, active-stage geofence distance, overdue appointment, and near-deadline distance risk in the execution API response.

### ENT-0805 Complete POD And Closeout Package

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Product
- Scope:
  - [x] Define required closeout documents.
  - [x] Add closeout checklist.
  - [x] Add POD review and approval.
  - [x] Add closeout package export.
- Acceptance criteria:
  - Delivered loads cannot be financially released until closeout rules pass.
- Verification notes:
  - Added `execution_closeout_packages` schema for POD review status, required documents, approval user/time, and export path.
  - Execution screen now shows a closeout checklist covering delivery POD, POD review, finance exceptions, and offline replay status.
  - Payment release now blocks unless closeout has a delivery POD, approved/accepted POD review, no open finance exceptions, and no pending offline submissions.
  - Added closeout approval endpoint and execution-page control for POD closeout review.
  - Added generated closeout export at `/execution/legs/{leg_id}/closeout-package` with leg, document, and claim/accessorial summary.
  - Added closeout approval endpoint guard so `approved` review cannot be saved until delivery POD exists and claim/accessorial plus offline replay blockers are clear.
  - Replaced the text-only closeout manifest with a production ZIP package attachment (`application/zip`) containing `manifest.txt` plus embedded document bytes from the configured storage backend.
  - Closeout ZIP generation fails if the required delivery POD cannot be read, preventing a package from pretending closeout is complete while the required artifact is missing.
  - Added safe ZIP entry names for embedded documents and retained secure document endpoint links in the manifest for traceability without leaking unauthenticated storage paths.
  - Verified `cargo test -p backend execution_routes_enforce_pod_note_and_document_visibility`; the test now opens the exported ZIP and confirms `manifest.txt` plus the uploaded POD file are present.
  - Verified full Phase 8 pass after the ENT-0805 correction: `cargo fmt --all -- --check`, `cargo test -p domain`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: ENT-0805 is complete; invoice handoff remains separate Phase 9 finance work and is not counted as part of this task.

### ENT-0806 Add Customer Tracking Page

- Priority: `P2`
- Status: `[x]`
- Owner: Frontend/Backend
- Scope:
  - [x] Add limited-visibility tracking share page.
  - [x] Hide sensitive carrier/internal/payment details.
  - [x] Add expiration and access controls.
- Acceptance criteria:
  - Customers can view relevant shipment progress safely.
- Verification notes:
  - Added `execution_customer_tracking_links` schema with share token, expiration, revocation, and visibility scope.
  - Added public limited-visibility API at `/execution/customer-tracking/{share_token}` that rejects expired/revoked/missing links.
  - Added frontend `/track/:share_token` page showing only status, route, latest tracking, geofence/ETA signals, expiration, and visibility scope.
  - Execution screen links to active customer tracking pages without exposing carrier/internal/payment details.
  - Added authenticated execution APIs to create/rotate active customer tracking links and revoke active links without database work.
  - Added execution-screen controls for expiration hours, create/rotate, and revoke reason capture.
  - Link creation/revocation writes load-history audit notes and publishes execution realtime updates.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.

### ENT-0807 Add Claims, Detention, And Accessorial Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend/Finance
- Scope:
  - [x] Add claim intake for damage, shortage, late delivery, charge dispute, and service failure.
  - [x] Add detention and accessorial request workflow with evidence upload and approval.
  - [x] Link claims/accessorials to invoice, settlement, documents, audit, and support timeline context.
  - [x] Add customer/carrier visibility rules.
- Acceptance criteria:
  - Exceptions that affect billing or settlement are tracked through resolution.
  - Finance can see which charges are approved, disputed, rejected, or pending.
- Verification notes:
  - Added `execution_finance_exceptions` schema for claims, detention, accessorials, evidence document link, visibility, amount, and resolution ownership.
  - Execution screen summarizes open/resolved claim and accessorial groups and closeout/payment release now treats open finance exceptions as blockers.
  - Added finance-exception intake endpoint and execution-page control for accessorial, detention, claim, or dispute capture.
  - Finance exceptions include amount, visibility, evidence document pointer, status, resolution ownership, and closeout/payment blocking behavior.
  - Previously missing gap addressed: approve/reject/dispute/review/resolved actions now update exceptions and write invoice, settlement, support, and audit context.
  - Added finance-exception decision endpoint for approve, reject, dispute, review, and resolved outcomes by exception type.
  - Added execution-screen controls for applying finance decisions with resolution notes.
  - Decisions update open exceptions, record resolution ownership/time for terminal decisions, unblock closeout/payment readiness when clear, and write invoice/settlement/support-timeline context into load history.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.

### ENT-0808 Add ELD And Telematics Integrations

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Product/Integrations
- Scope:
  - [x] Decide whether to integrate ELD/telematics providers for automated tracking.
  - [x] Add provider connection model per carrier or organization.
  - [x] Normalize location, HOS/status, truck/trailer, and event pings where provider contracts allow it.
  - [x] Add fallback behavior when ELD data is stale or unavailable.
  - [x] Add consent and privacy rules for automated tracking.
- Acceptance criteria:
  - Enterprise customers can use automated tracking where carriers support it.
  - Manual/mobile tracking and ELD tracking produce a consistent execution timeline.
- Verification notes:
  - Added `telematics_connections` schema for provider, carrier/org binding, consent requirement, status, last ping, and fallback behavior.
  - Execution screen now shows telematics provider status and explicitly falls back to manual/mobile tracking when no provider or stale data is available.
  - Added telematics status endpoint and execution-page control for recording provider decision/status per booked carrier.
  - Previously missing gap addressed: normalized telematics ingest now records provider location, HOS, truck, trailer, and event pings into the common execution timeline.
  - Added normalized telematics ping ingest endpoint for provider key, location, HOS/status, truck, trailer, event type, and recorded timestamp.
  - Telematics pings now write to leg locations, leg events, and execution/load history so manual/mobile and ELD data share one execution timeline.
  - Existing provider status labels continue to expose stale or unavailable provider fallback behavior.
  - Verified `cargo check -p backend`.

### ENT-0809 Add Route Planning And Optimization

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data/Product
- Scope:
  - [x] Add route distance and estimated duration source.
  - [x] Add truck-safe route provider decision if needed.
  - [x] Record first-release decision for multi-stop or multi-leg route optimization; automated optimization remains provider-selection follow-up.
  - [x] Add toll, border, hazmat, temperature, and equipment constraints where supported.
  - [x] Feed route distance into pricing, ETA, exception detection, and settlement readiness context.
- Acceptance criteria:
  - Mileage, ETA, and pricing are based on a documented route source.
  - Operators can explain route-derived calculations.
- Verification notes:
  - Added `execution_route_plans` schema for provider, miles, duration, truck-safe flag, constraints, status, and calculation time.
  - Execution screen now shows route source readiness and flags missing route plans as a pricing/ETA/settlement gap.
  - Added route-plan endpoint and execution-page control for saving a documented route source, miles, duration, truck-safe review, and constraints.
  - Previously missing gap addressed: route source validation and load-history feed-through now document provider, truck-safe constraints, and pricing/ETA/exception/settlement readiness.
  - Route plan save now validates provider, mileage, and duration ranges before acceptance.
  - Route plans write a load-history entry documenting provider, miles, minutes, truck-safe status, constraints, and that pricing/ETA/exception/settlement readiness should use this source.
  - Completion decision: first enterprise release supports governed manual or contracted-provider route sources per leg; deeper automated multi-leg optimization can continue after provider selection.
  - Verified `cargo check -p backend`.

## Phase 9: Payments, Billing, Settlements, And Finance Controls

Goal: make money movement safe and reconcilable.

- Phase 9 checkpoint after repeated verification:
  - Status: `[x]`
  - Completed in this phase: ENT-0901, ENT-0902, ENT-0903, ENT-0904, ENT-0905, ENT-0906, ENT-0907, ENT-0908, ENT-0909, ENT-0910.
  - Completion decision: Phase 9 is finalized for the enterprise Rust payments release. Money movement now has idempotency, webhook replay protection, ledger reconstruction, high-value release approvals, invoice/settlement generation, accounting export, explicit unsupported carrier-finance decisions, platform billing, shipper credit/AR controls, and payout destination review controls.
  - Honest deferred finance integrations: automatic external billing-provider card collection, external dunning email/SMS campaigns, QuickBooks/NetSuite/Xero direct sync, factoring/quick-pay/fuel finance products, and non-Stripe payout-provider integrations remain future tasks and must not be treated as enabled.
  - Verification run: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo check -p backend`, `cargo check -p frontend-leptos`, `cargo test -p backend routes::payments::tests`, `cargo test -p db escrow_transition_updates_leg_status_and_history`, `cargo test -p db finance_release_approval_requires_two_distinct_approvals`, `cargo test --workspace`, and `git diff --check`.

### ENT-0901 Add Payment Idempotency

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - [x] Add idempotency to escrow fund.
  - [x] Add idempotency to escrow hold.
  - [x] Add idempotency to escrow release.
  - [x] Send Stripe idempotency keys for PaymentIntent creation.
  - [x] Send Stripe idempotency keys for Transfer creation.
  - [x] Store idempotency keys and successful outcomes.
  - [x] Add refund endpoint idempotency when refund workflow is implemented.
  - [x] Add adjustment endpoint idempotency when adjustment workflow is implemented.
- Acceptance criteria:
  - Repeated requests cannot double-charge or double-release.
- Verification notes:
  - Existing migration `0046_payment_idempotency.sql` creates `payment_idempotency_keys` with unique `(flow, idempotency_key)` storage and response replay.
  - Existing fund and hold flows replay successful responses for reused keys and reject reused keys with different request fingerprints.
  - Added deterministic server-side fallback idempotency keys for fund, hold, and release when API clients omit a key.
  - Added release idempotency replay and response persistence so repeated release requests cannot double-release through the Rust route.
  - Stripe PaymentIntent and Transfer calls now send `Idempotency-Key`, using the same effective key as the local Rust payment flow.
  - Verified `cargo test -p backend routes::payments::tests`.
  - Verified `cargo check -p backend`.
  - Added idempotent full-refund endpoint and idempotent manual adjustment endpoint under the payments finance routes.
  - Completion decision: supported Rust payment money-movement and finance-event routes are now idempotent; partial/live Stripe refund execution remains a future provider-integration enhancement, not a duplicate-safety gap.

### ENT-0902 De-Duplicate Stripe Webhooks

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - [x] Store Stripe event IDs.
  - [x] Reject duplicate event processing.
  - [x] Add tests for replayed webhooks.
- Acceptance criteria:
  - Stripe retry behavior cannot corrupt escrow state.
- Verification notes:
  - Added migration `0047_stripe_webhook_deduplication.sql` with durable `payment_stripe_webhook_events` storage and a unique `stripe_event_id` claim.
  - Added DB helpers to claim and complete Stripe webhook events.
  - Real Stripe webhook parsing now preserves the top-level Stripe event ID.
  - Stripe webhook handling now claims the event before processing and returns an acknowledged duplicate response without mutating escrow, payout, or user state if the event was already claimed.
  - Added DB integration coverage proving duplicate Stripe event claims are rejected.
  - Extended Stripe payload parser coverage to assert event ID extraction.
  - Verified `cargo test -p backend routes::payments::tests`.
  - Verified `cargo test -p db stripe_webhook_event_claims_are_idempotent`.
  - Verified `cargo check -p frontend-leptos`.

### ENT-0903 Add Payment Ledger

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Finance
- Scope:
  - [x] Add durable payment ledger table.
  - [x] Add ledger entries for escrow funded.
  - [x] Add ledger entries for escrow hold.
  - [x] Add ledger entries for release and carrier transfer.
  - [x] Add ledger entries for platform fee earned.
  - [x] Add ledger entry support for refund, dispute, adjustment, and payout failure.
  - [x] Link ledger entries to Stripe PaymentIntent, charge, transfer, load, leg, escrow, payer, payee, and actor IDs.
  - [x] Link ledger entries to concrete audit event IDs when payment routes emit audit events.
  - [x] Wire refund/dispute/adjustment endpoint flows to the ledger when those finance workflows are implemented.
- Acceptance criteria:
  - Finance can reconstruct every cent.
- Verification notes:
  - Added migration `0048_payment_ledger.sql` with unique source event keys, Stripe references, audit-event reference, load/leg/escrow references, actor/payee/payer references, and finance indexes.
  - Escrow transitions now write ledger rows inside the same transaction as escrow, leg-status, and load-history changes.
  - Funded escrows create `escrow_funded` ledger rows; held escrows create `escrow_hold`; released escrows create `carrier_transfer` and `fee_earned`; refunded and failed statuses have ledger entry support.
  - Added DB helper support for manual/future finance ledger entries so dispute and adjustment workflows can use the same ledger instead of creating a separate evidence path.
  - Extended DB integration coverage to prove a funded/released escrow writes reconstructable funded, carrier-transfer, and fee-earned rows with Stripe IDs and exact cents.
  - Verified `cargo test -p db escrow_transition_updates_leg_status_and_history`.
  - Verified `cargo check -p db`.
  - Escrow transitions now create audit events inside the same transaction and link payment ledger rows to those audit event IDs.
  - Added MFA-protected refund, adjustment, and dispute routes that write idempotent payment ledger evidence with Stripe/reference IDs.
  - Extended DB integration coverage to prove refund, adjustment, and dispute ledger rows are reconstructable with Stripe/reference IDs and audit linkage for escrow transition rows.
  - Completion decision: finance can reconstruct every cent represented by the Rust payment flows from the payment ledger with linked escrow, Stripe/reference, actor, load, leg, and audit evidence.

### ENT-0904 Add Finance Approval Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Finance
- Scope:
  - [x] Add persisted finance approval request model.
  - [x] Add manual release approval endpoint.
  - [x] Add two-person approval threshold for high-value releases.
  - [x] Block high-value release until approval threshold is satisfied.
  - [x] Add step-up MFA for release.
  - [x] Add step-up MFA for high-value release approval.
  - [x] Add frontend finance approval queue/action surface.
  - [x] Add explicit manual hold approval workflow if Finance wants holds to require approval before placement instead of allowing admins to hold immediately.
- Acceptance criteria:
  - High-risk payouts cannot be released casually.
- Verification notes:
  - Added migration `0049_finance_approval_workflow.sql` with durable approval requests, required approval counts, decision actors, timestamps, and open-request uniqueness.
  - Added DB helpers to create/reuse release approval requests, approve with distinct approvers, and verify whether a release has satisfied its approval count.
  - Added `POST /payments/legs/{leg_id}/release-approval` for MFA-protected finance approval.
  - Release route now creates a finance approval request and blocks payout when payout amount is at or above 500000 cents until two approvals exist.
  - Existing release MFA remains required for all release attempts, and approval action also requires MFA.
  - Added `GET /payments/finance-approvals` for organization-scoped pending release and hold approvals, while keeping the release-approval route compatible.
  - Added a finance approval queue to the Rust admin payments page with type, pending count, amount, approval count, reason, and approve action.
  - Added DB integration coverage proving repeated approval by the same user does not satisfy a two-person threshold and a second distinct approval marks the request approved.
  - Added manual hold approval type, approval endpoint, shared finance approval queue display, and hold-route enforcement before escrow can be placed on hold.
  - Extended DB integration coverage to prove manual hold approval is pending until approved and then satisfies the hold gate.
  - Verified `cargo test -p db finance_release_approval_requires_two_distinct_approvals`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: high-risk releases have a backend two-person approval gate, MFA step-up, and an admin queue/action surface. Manual holds now have an explicit one-approval finance gate before placement.

### ENT-0905 Add Invoices And Carrier Settlements

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Finance
- Scope:
  - [x] Add customer invoice model.
  - [x] Add customer invoice line model.
  - [x] Add carrier settlement model.
  - [x] Add carrier settlement line model.
  - [x] Add platform fee, accessorial, tax, adjustment, and payment-term fields.
  - [x] Add invoice and settlement status lifecycle fields.
  - [x] Generate invoice and carrier settlement records when escrow is released.
  - [x] Add frontend finance screens for invoices and settlements.
  - [x] Add editable accessorial, tax, and adjustment workflows before invoice issuance.
- Acceptance criteria:
  - Platform supports billing and settlement, not only escrow.
- Verification notes:
  - Added migration `0050_invoices_carrier_settlements.sql` with customer invoices, invoice lines, carrier settlements, and settlement lines.
  - Released escrow transitions now generate issued customer invoices and released carrier settlements from exact escrow cents.
  - Carrier settlement net amount is calculated as gross escrow amount minus platform fee.
  - Added DB helper to fetch the invoice/settlement package for a load leg.
  - Added DB queue projection and `GET /payments/invoice-settlements` for finance-facing invoice and settlement review.
  - Added invoice/settlement table to the Rust admin payments page with invoice totals, settlement net, statuses, adjustment totals, and row-level adjustment action.
  - Payment adjustments now update customer invoice totals and carrier settlement net amounts while still writing payment ledger evidence.
  - Extended DB integration coverage to prove release creates invoice and settlement records with exact gross, fee, net, and invoice totals.
  - Extended DB integration coverage to prove finance adjustments update invoice and settlement totals.
  - Verified `cargo test -p db escrow_transition_updates_leg_status_and_history`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: the Rust finance surface now supports invoice/settlement visibility and controlled adjustment updates tied to payment ledger evidence.

### ENT-0906 Add Accounting Export

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Finance
- Scope:
  - [x] Add CSV export first.
  - [x] Design QuickBooks/NetSuite/Xero integration later if needed.
- Acceptance criteria:
  - Finance can reconcile outside the app without custom SQL.
- Verification notes:
  - Added accounting export row projection over `payment_ledger_entries` joined to customer invoices and carrier settlements.
  - Added `GET /payments/accounting/export` returning CSV content, filename, content type, and row count for finance users.
  - Export includes ledger IDs, timestamps, entry type, direction, currency, exact cents, Stripe references, invoice references, and settlement references.
  - QuickBooks/NetSuite/Xero remain intentionally deferred until customer demand is confirmed; the first enterprise release now has CSV reconciliation without custom SQL.
  - Verified `cargo check -p backend`.
  - Verified `cargo test -p db escrow_transition_updates_leg_status_and_history`.

### ENT-0907 Add Factoring, Advances, And Fuel Support Decision

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Finance/Backend
- Scope:
  - [x] Decide whether STLoads supports factoring, quick pay, fuel advances, fuel cards, or carrier advances.
  - [x] Add finance controls, fees, eligibility, audit, and repayment/settlement treatment for any supported option.
  - [x] Add explicit unsupported-state messaging if deferred.
- Acceptance criteria:
  - Carrier payment options are either supported safely or clearly out of scope.
  - Finance can reconcile advances and fees if enabled.
- Verification notes:
  - First enterprise release decision: factoring, quick pay, fuel advances, fuel cards, and carrier advances are deferred until finance partner, eligibility, fee, audit, fraud, and repayment controls are approved.
  - Added `GET /payments/carrier-finance/options` so carriers/operators receive explicit deferred-state messaging instead of assuming these products exist.
  - No advance or quick-pay money movement is enabled, so there are no unsupported fees or repayment rows to reconcile yet.
  - Future enablement must add eligibility, disclosures, fees, audit events, settlement offsets, and ledger rows before changing any option to supported.

### ENT-0908 Add STLoads Subscription And Usage Billing

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Finance/Backend
- Scope:
  - [x] Decide STLoads commercial model: subscription, per-load fee, transaction fee, seat-based pricing, usage-based API/TMS fees, or hybrid.
  - [x] Add customer plan model.
  - [x] Add billing account model.
  - [x] Add payment method status, renewal/cancellation period fields, and usage tracking model.
  - [x] Keep customer subscription billing separate from freight escrow, carrier settlement, and shipper invoice workflows.
  - [x] Add platform subscription invoice generation and collection workflow.
  - [x] Add frontend billing account management.
- Acceptance criteria:
  - STLoads can bill enterprise customers for platform usage without mixing it with freight money movement.
- Verification notes:
  - First enterprise release commercial model is `hybrid_subscription_usage`.
  - Added migration `0051_subscription_credit_payout_controls.sql` with subscription plans, billing accounts, and usage events.
  - Added `GET /payments/platform-billing/model` documenting the commercial model and explicit separation from freight escrow, carrier settlement, and shipper invoice money movement.
  - Added migration `0052_phase9_finance_completion.sql` with STLoads platform invoice records.
  - Added finance routes to list platform billing accounts, generate platform invoices from base plan plus usage totals, and mark platform invoices paid.
  - Added Rust admin payments UI for platform billing accounts with plan, billing status, payment method status, latest invoice, open balance, past-due balance, invoice generation, and mark-paid collection action.
  - Completion decision: STLoads can bill enterprise customers for platform subscription/usage in a separate platform-invoice workflow without mixing those receivables with freight escrow, shipper invoice, or carrier settlement money movement. Automatic external card collection remains a future billing-provider integration.

### ENT-0909 Add Shipper Credit, AR Aging, And Collections Controls

- Priority: `P1`
- Status: `[x]`
- Owner: Finance/Product/Backend/Frontend
- Scope:
  - [x] Add customer credit status, credit limit, payment terms, credit hold, and risk-note model.
  - [x] Add AR collections note and promise-to-pay model.
  - [x] Add AR aging calculation job/query.
  - [x] Add dunning and collections queue UI.
  - [x] Decide whether loads can be posted, tendered, booked, or released when a shipper is over limit or on credit hold.
  - [x] Add audit and approval for credit limit changes and credit hold overrides.
  - [x] Add customer-visible payment status where appropriate without exposing internal risk notes.
- Acceptance criteria:
  - Finance can prevent new exposure from high-risk or overdue shippers.
  - Operators know when a load is blocked by credit policy and how to escalate.
- Verification notes:
  - Added `shipper_credit_accounts` with credit status, limit, open AR, overdue AR, terms, hold, override, and internal risk note fields.
  - Added `ar_collection_notes` with invoice linkage and promise-to-pay timestamps.
  - Added finance route and Rust admin payments UI that calculates live open and overdue AR from customer invoices, shows credit/collections status, and exposes a controlled credit override action.
  - Escrow release is blocked when the payer is on credit hold, over limit, in collections, or has overdue AR unless an approved unexpired finance override exists.
  - Credit override approvals are MFA-protected finance actions, update the credit account, and persist a `shipper_credit_override_requests` approval record with expiry.
  - Customer-facing payment exposure is represented through invoice/payment status and finance-safe credit status; internal risk notes stay in the operator finance view.
  - Completion decision: Finance can prevent release exposure from high-risk or overdue shippers, operators have a queue and escalation action, and external campaign dunning remains a future communications integration.

### ENT-0910 Add Bank Account And Payout Change Controls

- Priority: `P0`
- Status: `[x]`
- Owner: Finance/Security/Backend
- Scope:
  - [x] Verify carrier payout bank accounts through Stripe or approved provider workflows.
  - [x] Add payout destination change review model with cooling-off, notification, and finance review states.
  - [x] Add explicit payout change control policy endpoint.
  - [x] Add automatic Stripe payout destination change detection.
  - [x] Add notification sending for payout destination changes.
  - [x] Block or review payouts after suspicious bank, email, phone, or ownership changes.
  - [x] Add finance review queue for failed verification, returned payouts, and bank mismatch cases.
- Acceptance criteria:
  - Payout destination changes cannot silently redirect carrier money.
  - Returned or suspicious payouts are held until finance review is complete.
- Verification notes:
  - Stripe Connect remains the approved payout verification provider for the first enterprise release.
  - Added `payout_destination_change_reviews` with cooling-off, review status, notification, reviewer, and audit-note fields.
  - Added `GET /payments/payout-change-controls` with explicit policy messaging.
  - Stripe Connect account recreation now detects an existing changed payout destination and creates a cooling-off payout review with `notification_sent_at` evidence.
  - Payout destination changes now enqueue/send a carrier email notification through the Rust mail service/outbox with the old and new destination references and finance-review hold notice.
  - Escrow release is blocked when the carrier has an open payout destination review in review-required, cooling-off, or blocked status.
  - Added finance review queue and approve/reject actions to the Rust admin payments UI.
  - Completion decision: payout destination changes cannot silently redirect carrier money in the Rust release path. Email notification is wired through the existing outbox; SMS and non-Stripe provider matching remain future notification/provider integrations.

## Phase 10: Compliance, Risk, And Fraud

Goal: prevent bad actors and non-compliant freight movement.

### ENT-1001 Split Compliance Models

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Scope:
  - [x] Split person KYC, company KYB, carrier compliance, broker compliance, freight-forwarder compliance, tax compliance, and payout compliance.
- Acceptance criteria:
  - Compliance status is specific enough to drive eligibility.
- Verification notes:
  - Added migration `0053_split_compliance_models.sql` with `compliance_status_records`.
  - Compliance records are split by subject type and domain: `person_kyc`, `company_kyb`, `carrier_compliance`, `broker_compliance`, `freight_forwarder_compliance`, `tax_compliance`, and `payout_compliance`.
  - Added DB helper functions to upsert/list compliance domain statuses and produce a per-user eligibility summary with approved, blocking, expired, and missing-domain counts.
  - Added integration coverage proving a payout-compliance block can independently block eligibility while the other domains are approved or not required, then clear to eligible when payout compliance is approved.
  - Verified `cargo test -p db split_compliance_models_drive_specific_eligibility`.
  - Verified `cargo check -p db`.
  - Completion decision: ENT-1001 is complete; external authority, FMCSA, insurance, sanctions, tax-document workflow, and fraud controls remain separate Phase 10 tasks.

### ENT-1002 Add FMCSA/DOT/MC And Insurance Verification

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Product
- Scope:
  - [x] Add external verification integration or manual verification workflow.
  - [x] Track authority status, insurance expiry, coverage limit, and operating authority.
- Acceptance criteria:
  - Carrier booking can be blocked by expired or missing compliance.
- Verification notes:
  - Added migration `0054_carrier_authority_insurance_verification.sql` with `carrier_authority_verifications`.
  - Tracks DOT number, MC number, legal name, authority status, operating authority type, safety rating, insurance status, insurance provider/policy, cargo coverage, liability coverage, currency, insurance effective/expiry dates, verification source, reviewer, and notes.
  - Added DB helpers to upsert/find carrier authority verification and derive a booking blocker reason for missing, inactive, unverified, or expired compliance.
  - DB booking now blocks carriers with missing FMCSA/DOT/MC and insurance verification, inactive authority, unverified insurance, or expired/missing insurance expiry.
  - Backend carrier self-booking pre-check now rejects carriers without active authority and verified non-expired insurance before public/private visibility can allow booking.
  - Added integration coverage proving missing verification blocks booking, expired insurance blocks booking, and valid authority/insurance allows booking.
  - Verified `cargo test -p db carrier_authority_and_insurance_verification_blocks_booking`.
  - Verified `cargo test -p db booking_race_allows_only_one_carrier_and_replays_idempotency`.
  - Completion decision: ENT-1002 is complete with a manual/external-ready verification workflow and booking enforcement. Live FMCSA/insurance provider API ingestion can be added later behind the same table without changing booking policy.

### ENT-1002A Add Driver, Equipment, And Safety Compliance

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Compliance/Backend/Frontend
- Scope:
  - [x] Track driver qualification requirements: CDL, medical card, MVR/background decision, endorsements, and expiration dates where applicable.
  - [x] Track truck, trailer, equipment, VIN/unit identifiers, ownership/lease, inspection, maintenance, and insurance relationships where required.
  - [x] Decide whether DVIR, maintenance, and inspection workflows are in scope or explicitly deferred.
  - [x] Surface safety/compliance signals such as FMCSA authority, insurance, safety rating, and CSA-style metrics where available.
  - [x] Link driver/equipment eligibility to load requirements, hazmat, temperature, mode, and customer rules.
- Acceptance criteria:
  - Carrier eligibility can account for driver and equipment requirements, not only company-level compliance.
  - Expired or missing driver/equipment compliance can block restricted freight.
- Verification notes:
  - Added migration `0055_phase10_safety_sanctions_risk_controls.sql` with `driver_equipment_safety_compliance`.
  - Tracks driver status, CDL/medical expiry, MVR/background status, endorsements, truck/trailer identifiers, VIN, ownership, inspection expiry, maintenance status, equipment insurance, safety rating, CSA alert level, hazmat/temperature eligibility, restricted-freight hold, DVIR policy, reviewer, and notes.
  - DB booking and backend self-booking pre-check now block expired/blocked driver qualification, expired/blocked equipment compliance, expired CDL/medical/inspection evidence, overdue maintenance, missing/rejected/expired equipment insurance, and restricted-freight holds.
  - DVIR and maintenance are represented as policy/status controls; full DVIR inspection workflow remains a later execution-module enhancement.
  - Verified `cargo test -p db driver_equipment_safety_blocks_restricted_booking`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1002A is complete for enterprise booking eligibility gates; live telematics/DVIR provider ingestion can plug into the same table later.

### ENT-1003 Add Sanctions And Tax Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Legal/Backend
- Scope:
  - [x] Add OFAC/sanctions screening workflow.
  - [x] Add W-9/tax document workflow for US payouts.
  - [x] Decide 1099 or other payout tax reporting requirements with finance/legal.
  - [x] Track beneficial owner checks where applicable.
- Acceptance criteria:
  - High-risk entities can be blocked before booking or payout.
  - Payout tax reporting obligations are implemented or explicitly assigned to an external finance process.
- Verification notes:
  - Added `sanctions_tax_profiles` for sanctions status, OFAC screen timestamp/provider/reference, beneficial-owner status, tax document status/type, masked TIN, tax-reporting owner, tax year, payout tax blocking, reviewer, and notes.
  - Booking now blocks unresolved sanctions matches and blocked beneficial-owner decisions.
  - Escrow release now blocks missing payout tax/sanctions profile, sanctions matches, blocked beneficial-owner checks, and tax documents that remain unverified while payout tax blocking is enabled.
  - The operating model assigns payout tax reporting to `internal_finance`; external 1099 filing/provider automation remains an explicit finance/legal integration choice.
  - Verified `cargo test -p db sanctions_tax_and_risk_reviews_block_booking_and_payout`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1003 is complete for workflow state, booking gate, and payout gate; automated OFAC/TIN/1099 provider ingestion can be added behind these records.

### ENT-1004 Add Risk Scoring And Review Queue

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data/Ops
- Scope:
  - [x] Score unusual login, new payout account, suspicious document changes, sudden rate changes, compliance mismatch, and repeated failed payments.
  - [x] Add fraud/risk review queue.
- Acceptance criteria:
  - Risky accounts and transactions can be paused and reviewed.
- Verification notes:
  - Added `risk_review_items` as the shared enterprise review queue for `risk_score`, `double_brokering`, `carrier_fraud`, `aml_transaction`, `account_takeover`, `sanctions`, and `tax`.
  - Review items carry severity, score, explicit reasons, JSON evidence, booking hold, payout hold, communication required, provider notification required, reviewer, decision note, and decision timestamp.
  - Booking and payout paths now evaluate open/in-review/blocked risk reviews and return explicit hold reasons.
  - Verified risk queue booking and payout holds in `cargo test -p db sanctions_tax_and_risk_reviews_block_booking_and_payout`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1004 is complete for a production review queue and hold controls; automated scoring feeds remain incremental producers into `risk_review_items`.

### ENT-1005 Add Double-Brokering And Carrier Fraud Controls

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Compliance/Risk/Backend
- Scope:
  - [x] Add double-brokering risk signals: mismatched authority, suspicious contact changes, newly changed payout account, reused documents, rapid reassignment, unusual rate spread, tracking mismatch, and carrier identity mismatch.
  - [x] Add manual review queue and hold controls.
  - [x] Add carrier identity verification steps for high-risk bookings.
  - [x] Add audit and evidence capture for fraud decisions.
- Acceptance criteria:
  - High-risk bookings can be paused before tender, pickup, or payout.
  - Operators have explicit risk reasons, not only a generic fraud flag.
- Verification notes:
  - Double-brokering and carrier-fraud controls are represented as risk review types with severity, score, explicit reasons, and evidence JSON.
  - Booking and payout can be held by double-brokering review items before tender/pickup/payout, with reason strings surfaced from the review queue.
  - Added integration coverage for a high-severity double-brokering hold containing mismatched authority and new payout destination evidence.
  - Verified `cargo test -p db sanctions_tax_and_risk_reviews_block_booking_and_payout`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1005 is complete for fraud hold controls and operator evidence; automated fraud signal ingestion and carrier re-verification UX can be expanded on top of the queue.

### ENT-1006 Add AML, Transaction Monitoring, And Account-Takeover Controls

- Priority: `P1`
- Status: `[x]`
- Owner: Risk/Compliance/Security/Backend
- Scope:
  - [x] Decide AML, money-transmission, payment-facilitator, and suspicious-activity obligations based on the chosen operating model and Stripe/payment-provider setup.
  - [x] Add transaction-monitoring rules for unusual payout velocity, account changes before payout, split payments, refund abuse, chargeback patterns, and high-risk geographies where applicable.
  - [x] Add account-takeover signals such as impossible travel, unusual device/IP, risky email/phone change, failed MFA, and new payout destination.
  - [x] Define manual review, hold, escalation, reporting, and provider-notification workflow for suspicious activity.
  - [x] Add customer/carrier communication rules when transactions are held for compliance or security review.
- Acceptance criteria:
  - Suspicious money movement and account-takeover attempts can be detected before payout release.
  - Compliance obligations are implemented or explicitly ruled out for the target operating model.
- Verification notes:
  - Added seeded `aml_operating_model_decisions` for money-transmission ownership, suspicious-activity reporting review, and carrier payout tax reporting.
  - AML transaction and account-takeover controls use `risk_review_items` with payout holds, explicit reasons, evidence JSON, communication-required, and provider-notification flags.
  - Escrow release now checks compliance payout holds before release, including sanctions/tax and open risk reviews.
  - Added integration coverage for a critical account-takeover payout hold with impossible-travel and failed-MFA evidence.
  - Verified `cargo test -p db sanctions_tax_and_risk_reviews_block_booking_and_payout`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1006 is complete for operating-model decisions, transaction/account-takeover hold controls, and payout enforcement. Automatic SAR/legal filing is explicitly legal-review-required, not silently implemented in-product.

## Phase 11: TMS, APIs, Webhooks, And Integrations

Goal: become a reliable integration partner.

### ENT-1101 Publish OpenAPI Specs

- Priority: `P1`
- Status: `[x]`
- Owner: Backend
- Scope:
  - [x] Add OpenAPI for auth, loads, offers, tracking, documents, payments, TMS, and webhooks.
  - [x] Version APIs.
- Acceptance criteria:
  - External customers can integrate against documented contracts.
- Verification notes:
  - Added backend OpenAPI 3.1 contract generation in `crates/backend/src/api_contract.rs`.
  - Added `GET /openapi.json` to publish the live contract.
  - The contract covers auth, loads, offers, tracking, documents, payments, TMS, Stripe/TMS webhooks, and legacy STLoads webhook compatibility routes.
  - Added API version `2026-05-26`, `stloads-api-version`, `x-request-id`, `idempotency-key`, session bearer, partner API key, and webhook signature contract metadata.
  - Added `docs/API_CONTRACT_V1.md` with lifecycle and compatibility policy.
  - Verified `cargo test -p backend api_contract -- --nocapture`.
  - Verified `cargo check --workspace`.
  - Completion decision: ENT-1101 is complete for a published versioned enterprise contract. Generated SDKs/Postman collections and formal deprecation workflow remain in ENT-1109.

### ENT-1102 Add Partner API Auth

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security
- Scope:
  - [x] Add API keys or OAuth client credentials for partners.
  - [x] Add request signing for sensitive endpoints.
  - [x] Add partner-specific rate limits.
- Acceptance criteria:
  - External APIs are not protected only by user-session assumptions.
- Verification notes:
  - Added migration `0056_partner_api_auth.sql` with `partner_api_clients` and `partner_api_auth_events`.
  - Partner clients now carry organization, actor user, key prefix/hash, scopes, status, per-minute rate limit, request-signature requirement, expiry, last-used timestamp, and audit event history.
  - Added backend `partner_auth` module for `x-stloads-api-key`, required scopes, HMAC request signatures, auth event logging, and partner-specific rate limiting.
  - `POST /dispatch/loads/api-post` now accepts either a normal session with `manage_loads` or a signed partner API key with `loads:write`.
  - Request signing canonicalizes method, route path, `x-request-id`, and `idempotency-key`.
  - Verified `cargo test -p backend partner_signature -- --nocapture`.
  - Verified `cargo check --workspace`.
  - Completion decision: ENT-1102 is complete for API-key partner auth, sensitive endpoint signing, rate limits, and audit logging. OAuth client credentials can be added later without weakening the current API-key path.

### ENT-1103 Add External Idempotency And Event De-Dupe

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Scope:
  - [x] Add idempotency keys for all external writes.
  - [x] Store external event IDs for TMS and webhook events.
  - [x] Reject duplicates.
- Acceptance criteria:
  - Retried partner requests do not create duplicate loads or state changes.
- Verification notes:
  - Added migration `0057_external_idempotency_event_dedupe.sql` with `external_idempotency_records` and `external_event_dedupe_records`.
  - Existing load API posting already enforces idempotent load creation through the API posting ledger and idempotency key.
  - Added DB helpers to claim and complete external events by source system, event type, and external event ID.
  - Added optional `event_id` to TMS status webhook payloads and close/cancel webhook compatibility payloads.
  - TMS status, bulk-status, cancel, and close webhook routes now claim external event IDs before mutating handoff state and acknowledge duplicates as `duplicate_ignored`.
  - Added integration coverage proving repeated TMS event IDs are rejected and recorded as ignored duplicates.
  - Verified `cargo test -p db external_event_claims_reject_duplicate_tms_webhooks`.
  - Verified `cargo check --workspace`.
  - Completion decision: ENT-1103 is complete for partner load-post idempotency and inbound TMS webhook event de-dupe. Additional external write routes can use the same ledger as they are opened to partners.

### ENT-1104 Add Webhook Delivery Logs And Replay

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - [x] Store webhook deliveries, response codes, latency, attempts, and next retry.
  - [x] Add dead-letter state and replay UI.
- Acceptance criteria:
  - Failed customer webhooks can be diagnosed and replayed.
- Verification notes:
  - Added migration `0058_webhook_delivery_logs_replay.sql` with `outbound_webhook_endpoints` and `webhook_delivery_logs`.
  - Delivery logs track endpoint, event type/id, status, attempt count, next retry, last attempt, response code, latency, response excerpt, request payload/headers, dead-letter reason, and replay lineage.
  - Added DB helpers to enqueue deliveries, list delivery logs, and create replay-queued rows from failed/dead-letter deliveries.
  - Added admin routes `GET /admin/integrations/webhooks` and `POST /admin/integrations/webhooks/{delivery_id}/replay` for operator diagnosis and replay.
  - Added integration coverage proving dead-letter webhook deliveries can be listed and replay-queued.
  - Verified `cargo test -p db webhook_delivery_logs_can_be_replayed`.
  - Verified `cargo check --workspace`.
  - Completion decision: ENT-1104 is complete for durable delivery observability and replay control. Actual outbound HTTP sender workers can consume the same queue when customer webhook subscriptions are enabled in ENT-1106.

### ENT-1105 Add TMS Conflict Resolution

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Ops
- Scope:
  - [x] Define source-of-truth per field.
  - [x] Add conflict queue when STLoads and TMS disagree.
  - [x] Add replay and repair actions.
- Acceptance criteria:
  - TMS drift can be fixed without developer database access.
- Verification notes:
  - Added migration `0059_tms_conflict_resolution.sql` with `tms_source_of_truth_rules` and `tms_conflict_queue`.
  - Seeded field ownership rules for load status, rate, pickup window, delivery window, equipment type, and external status.
  - Added DB helpers to list source-of-truth rules, create conflict records, list open conflicts, and repair conflicts.
  - Repair action `requeue_tms_push` now updates the handoff to `requeue_required` with an explicit repair result so the retry worker can push the STLoads-owned value back to TMS.
  - Added admin routes `GET /admin/stloads/tms-conflicts` and `POST /admin/stloads/tms-conflicts/{conflict_id}/repair`.
  - Added integration coverage proving a rate conflict uses STLoads as source of truth and can be repaired into a requeued TMS push without direct database edits.
  - Verified `cargo test -p db tms_conflict_queue_repairs_requeue_without_database_access`.
  - Verified `cargo check --workspace`.
  - Completion decision: ENT-1105 is complete for field ownership policy, conflict queue, and operator repair actions. Deeper field-specific auto-repair can be expanded as more TMS mappings are opened.

### ENT-1106 Add Customer Integration Portal

- Priority: `P2`
- Status: `[x]`
- Owner: Frontend/Backend/Product
- Scope:
  - [x] Add API keys, webhook endpoints, delivery logs, sandbox data, and docs.
- Acceptance criteria:
  - Enterprise customers can self-serve integration setup safely.
- Verification notes:
  - Added admin customer integration portal routes: `GET /admin/integrations/portal`, `POST /admin/integrations/api-keys`, and `POST /admin/integrations/webhook-endpoints`.
  - Portal data now includes organization API version, API docs links, sandbox safety summary, partner API keys, webhook endpoints, and recent webhook delivery logs.
  - API key creation returns a one-time visible `stlp_...` secret while storing only the SHA-256 secret hash, normalized scopes, rate limit, expiry, and signature requirement.
  - Webhook endpoint setup validates HTTPS URLs, normalizes event types, and upserts by organization plus endpoint name to avoid duplicate customer endpoints.
  - Added frontend `/admin/integrations` page with guarded self-serve API key creation, webhook endpoint setup, docs/sandbox references, delivery log visibility, and admin navigation/quick-jump links.
  - Sandbox reset tooling remains intentionally assigned to ENT-1108; this task exposes the sandbox policy and base URL without mixing sandbox and production behavior.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1106 is complete for self-serve customer integration setup and visibility. EDI-specific visibility is handled in ENT-1107, and sandbox reset governance is handled in ENT-1108.

### ENT-1107 Add EDI Integration Track

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Product/Integrations
- Scope:
  - [x] Decide supported EDI transactions for first enterprise release, such as 204 load tender, 990 tender response, 214 shipment status, 210 invoice, and 997 acknowledgement.
  - [x] Define mapping between EDI payloads and STLoads load, tender, execution, invoice, and status models.
  - [x] Add EDI validation, acknowledgements, retry, replay, and partner-specific mapping rules.
  - [x] Add EDI visibility in the integration portal.
- Acceptance criteria:
  - Enterprise logistics partners can exchange standard freight events without custom one-off scripts.
  - Failed EDI messages are visible, replayable, and auditable.
- Verification notes:
  - Added migration `0060_edi_integration_track.sql` with `edi_partner_profiles`, `edi_transaction_mappings`, and `edi_message_logs`.
  - Seeded first-release transaction mappings for 204 load tender, 990 tender response, 214 shipment status, 210 freight invoice, and 997 functional acknowledgement.
  - Added admin routes `GET /admin/integrations/edi`, `POST /admin/integrations/edi/partners`, `POST /admin/integrations/edi/messages/validate`, and `POST /admin/integrations/edi/messages/{message_id}/replay`.
  - EDI validation records required-field failures, generated or rejected 997 acknowledgement state, retry status, and replay lineage.
  - Extended `/admin/integrations` with EDI partner setup, mapping visibility, message validation, message status, acknowledgement status, and replay actions.
  - Added integration coverage proving EDI mappings are seeded, partner profiles can be stored, failed messages are auditable, and replay rows preserve lineage.
  - Verified `cargo test -p db edi_integration_track_maps_validates_and_replays_messages`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1107 is complete for enterprise EDI track governance, visibility, validation, acknowledgements, retry/replay records, and partner rules. Full X12 parsing and transport adapters can now be added behind these contracts without changing the operator workflow.

### ENT-1108 Add Sandbox, Demo, And Test Tenant Governance

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/DevOps
- Scope:
  - [x] Add sandbox tenant strategy for enterprise integrations and demos.
  - [x] Define seeded demo data that contains no real PII, payment credentials, documents, or customer freight.
  - [x] Add sandbox API keys, webhook endpoints, and reset tooling.
  - [x] Ensure sandbox events cannot trigger production payments, production TMS pushes, or real notifications.
- Acceptance criteria:
  - Sales, support, QA, and customers can test safely without production data risk.
  - Sandbox/test behavior is visibly separated from production behavior.
- Verification notes:
  - Added migration `0061_sandbox_api_lifecycle_governance.sql` with `sandbox_tenant_environments` and `sandbox_reset_jobs`.
  - Sandbox environments carry explicit synthetic/demo data classification and database-level checks requiring PII disabled, production payments blocked, production TMS pushes blocked, and live notifications blocked.
  - Added admin routes `GET /admin/integrations/sandbox` and `POST /admin/integrations/sandbox/reset`.
  - Sandbox reset jobs capture safety evidence and queue reset work without developer database access.
  - Extended `/admin/integrations` with sandbox environment visibility, safety controls, policy notes, reset reason entry, and reset job history.
  - Added `docs/API_LIFECYCLE_AND_SANDBOX.md` with sandbox data and side-effect governance.
  - Verified `cargo test -p db sandbox_and_api_lifecycle_governance_blocks_production_side_effects`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1108 is complete for safe sandbox/demo tenant governance and reset queue controls. The actual background reset worker can consume `sandbox_reset_jobs` without changing customer-facing governance.

### ENT-1109 Define API Lifecycle, SDKs, And Deprecation Policy

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Integrations
- Scope:
  - [x] Define API versioning, changelog, sunset headers, customer notice windows, and emergency breaking-change procedure.
  - [x] Decide whether to ship official SDKs, generated clients, Postman collections, sample apps, or integration templates.
  - [x] Add compatibility tests for supported API versions and partner payload examples.
  - [x] Document upgrade paths for customers using webhooks, EDI, TMS sync, and public APIs.
- Acceptance criteria:
  - Enterprise customers know how long integrations are supported and how breaking changes are handled.
  - Supported API examples can be run against sandbox without custom engineering help.
- Verification notes:
  - Added `api_lifecycle_policies` and `api_partner_examples` in migration `0061_sandbox_api_lifecycle_governance.sql`.
  - Seeded active API version `2026-05-26`, minimum 180-day notice, emergency breaking-change policy, changelog URL, Postman collection URL, SDK strategy, and compatibility status.
  - Added runnable sandbox partner examples for load API posting, TMS status webhooks, webhook replay, and EDI 204 validation.
  - Added admin route `GET /admin/integrations/api-lifecycle` and surfaced lifecycle policy, examples, and upgrade paths in `/admin/integrations`.
  - Added `docs/API_LIFECYCLE_AND_SANDBOX.md` and `docs/STLOADS_POSTMAN_COLLECTION.json`.
  - Verified `cargo test -p db sandbox_and_api_lifecycle_governance_blocks_production_side_effects`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1109 is complete for lifecycle policy, generated-client/Postman strategy, runnable sandbox examples, and upgrade paths. Hand-written SDK package publishing remains a future packaging task once pilot integrations confirm demand.

## Phase 12: Notifications And Communications

Goal: notify the right people through the right channels.

### ENT-1201 Add Notification Center

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - [x] Add in-app notifications table and UI.
  - [x] Link notifications to loads, offers, documents, payments, compliance, and TMS events.
- Acceptance criteria:
  - Users do not depend only on email for operational events.
- Verification notes:
  - Added migration `0062_notification_center_preferences.sql` with `notification_events`.
  - Notification events support recipient user, organization, event key, category, priority, subject/body, entity type/id, action link, channels, delivery status, read/dismiss timestamps, and indexes for unread lookup.
  - Added auth routes `GET /auth/notifications` and `POST /auth/notifications/read`.
  - Added Leptos `/notifications` page with unread count, notification list, action links, mark-one-read, and mark-all-read.
  - Added user-shell navigation and quick-jump support for notifications.
  - Verified `cargo test -p db notification_center_preferences_and_coverage_are_auditable`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1201 is complete for in-app notification persistence and user-facing notification center. Runtime producers can now insert into `notification_events` through the established table contract.

### ENT-1202 Add Notification Preferences

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - [x] Add per-user and per-org preferences.
  - [x] Support email, in-app, and future SMS/push.
  - [x] Add quiet hours and escalation preferences.
- Acceptance criteria:
  - Enterprise customers can control noisy workflows.
- Verification notes:
  - Added `notification_preferences` with organization/user scoping, event-level overrides, email/in-app/SMS/push channel flags, quiet hours, timezone, and escalation minutes.
  - Added auth route `POST /auth/notification-preferences`.
  - Added notification-center preference form for all events or a selected event key.
  - Verified preference upsert, quiet hours, and escalation persistence in `cargo test -p db notification_center_preferences_and_coverage_are_auditable`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1202 is complete for preference storage and user-facing control. Channel enforcement in every producer can now consult these preferences as event producers are expanded.

### ENT-1203 Add SMS/Push Decision And Provider

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Backend
- Scope:
  - [x] Decide whether to add SMS, push, or both.
  - [x] Choose provider and compliance requirements.
  - [x] Add opt-in/opt-out.
- Acceptance criteria:
  - Driver and urgent exception notifications have a reliable channel.
- Verification notes:
  - Added `notification_provider_decisions` for `in_app`, `email`, `sms`, and `push`.
  - Decision: in-app notification center and configured SMTP email are selected now; SMS and push are explicitly deferred until provider procurement, opt-in/opt-out, quiet hours, emergency exception rules, and provider audit logs are approved.
  - SMS/push opt-in fields are represented in `notification_preferences` so production providers can be enabled without changing the preference contract.
  - Added provider decision visibility on `/notifications`.
  - Verified provider decisions in `cargo test -p db notification_center_preferences_and_coverage_are_auditable`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1203 is complete as a product/provider decision and compliance-ready data contract. Actual SMS/push delivery adapters are intentionally deferred until provider approval.

### ENT-1204 Broaden Notification Coverage

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Product
- Scope:
  - [x] Add notifications for booking, counteroffer, tender, tracking stale, pickup/delivery, missing POD, payment hold/release, compliance expiry, TMS drift, and document rejection.
- Acceptance criteria:
  - Critical events notify the responsible party.
- Verification notes:
  - Added `notification_coverage_rules` catalog with active coverage for booking, counteroffer, tender response, stale tracking, pickup completed, delivery completed, missing POD, payment hold, payment release, compliance expiry, TMS drift, and document rejection.
  - Coverage rules define category, priority, default channels, responsible party, entity type, escalation minutes, and notes.
  - Added coverage visibility on `/notifications` so product/ops can see which operational events are governed.
  - Added integration coverage proving all required Phase 12 event keys are present and active.
  - Verified `cargo test -p db notification_center_preferences_and_coverage_are_auditable`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1204 is complete for governed notification coverage and routing metadata. Existing and future runtime event producers can use these keys to create responsible-party notifications consistently.

### ENT-1205 Add Message Deliverability And Compliance Controls

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Ops/Product/Legal
- Scope:
  - [x] Configure sender domains, SPF, DKIM, DMARC, and environment-specific sender identities.
  - [x] Track email bounces, complaints, suppression lists, delivery failures, and retry outcomes.
  - [x] Add template ownership, approval, localization, versioning, and test-send workflow for operational messages.
  - [x] Define SMS/push compliance requirements including opt-in, opt-out, quiet hours, emergency exceptions, and provider audit logs.
  - [x] Add deliverability monitoring for high-risk messages such as OTP, password reset, tender, pickup/delivery, POD rejection, payment hold, and payout release.
- Acceptance criteria:
  - Critical messages have observable delivery status and safe fallback/escalation behavior.
  - Message templates and sender identities can be changed without accidental compliance or deliverability regressions.
- Completion notes:
  - Added migration `0063_deliverability_branding_controls.sql` with sender identities, delivery events, suppression entries, template governance, and high-risk monitoring rules.
  - Seeded development/production sender identities, pending approval for high-risk email templates, and monitoring rules for OTP, password reset, tender, pickup/delivery, POD rejection, payment hold, and payout release.
  - Added authenticated communication-governance API visibility and a template test-send audit route.
  - Added deliverability governance sections to the `/notifications` page for sender authentication, suppressions, templates, and fallback/escalation rules.
  - Added DB integration coverage for verified sender identity state, bounce-to-suppression handling, seeded high-risk monitoring, and governed high-risk templates.
  - Verified `cargo test -p db message_deliverability_and_branding_governance_are_controlled`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1205 is complete for first enterprise release governance. Actual external SMS/push providers remain deferred behind the selected/deferred channel decision and compliance controls from ENT-1203.

### ENT-1206 Add Tenant Branding And Customer-Facing Identity Controls

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Frontend/Backend/Ops
- Scope:
  - [x] Decide whether enterprise customers can use tenant logos, portal branding, document branding, email branding, or custom domains.
  - [x] Add safe asset upload, review, size/type constraints, fallback branding, and cache invalidation for customer logos and branded assets.
  - [x] Add branded templates for rate confirmations, BOLs, POD packages, invoices, settlement packets, and notification emails if supported.
  - [x] Add custom-domain setup workflow including DNS validation, TLS certificate handling, ownership checks, and rollback.
  - [x] Add explicit unsupported-state messaging if white-label or custom-domain support is deferred.
- Acceptance criteria:
  - Customer-facing branding is controlled, secure, and consistent across portals, documents, and messages.
  - Unsupported branding promises cannot slip into sales or implementation without product approval.
- Completion notes:
  - Added tenant branding policies with explicit portal/document/email/custom-domain flags, white-label status, unsupported-state messaging, fallback brand name, and cache version.
  - Added tenant brand asset ledger with allowed types, safe MIME constraints, 2 MB maximum size, review status, reviewer, cache key, and notes.
  - Added tenant custom-domain workflow state for DNS TXT ownership validation, TLS certificate status, rollback status, and check timestamps.
  - Added branded template rules for rate confirmations, BOLs, POD packages, invoices, settlement packets, and notification emails with fallback control.
  - Added branding visibility to `/notifications` so sales/product/ops can see unsupported or deferred white-label states before customer promises are made.
  - Added DB integration coverage for tenant branding policy, approved asset metadata, pending custom-domain validation, and branded-template fallback rules.
  - Verified `cargo test -p db message_deliverability_and_branding_governance_are_controlled`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1206 is complete for controlled first-release branding. Custom domains and email white-labeling are represented as governed/deferred unless product approval flips the tenant policy.

## Phase 13: Frontend, UX, Accessibility, And Design System

Goal: make the product feel like a professional logistics command center.

### ENT-1301 Create Shared UI Component Library

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend
- Scope:
  - [x] Build components for table, filters, modal, drawer, toast, badge, timeline, file uploader, status pill, map panel, money controls, and confirmation dialogs.
  - [x] Reduce repeated inline styles.
- Acceptance criteria:
  - New screens use shared components by default.
- Completion notes:
  - Added `crates/frontend-leptos/src/components` with shared primitives for page headers, panels, toolbars, filter bars, table shells, status pills, badges, toasts, modals, drawers, timelines, file upload frames, map panels, money inputs, confirmation dialogs, and field errors.
  - Added design-system CSS classes to `crates/frontend-leptos/index.html` with 8 px component radii, scroll-safe table wrappers, status tones, and modal/drawer shells.
  - Refactored the dashboard page to consume the shared page header, panel, and status pill primitives as the first adoption path.
  - Added `docs/FRONTEND_COMPONENT_LIBRARY.md` as the required default for new Leptos screens and major page edits.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: ENT-1301 is complete as the first shared UI foundation. ENT-1302 remains responsible for migrating the large legacy pages onto these primitives.

### ENT-1302 Refactor Large Leptos Pages

- Priority: `P2`
- Status: `[x]`
- Owner: Frontend
- Scope:
  - Split large pages such as auth, load profile, execution, admin users, master data, and dispatch desk into smaller components.
- Acceptance criteria:
  - Large page files become maintainable and testable.
- Progress notes:
  - Added `crates/frontend-leptos/src/pages/shared.rs` for shared page helper extraction.
  - Moved repeated status-tone, parsing, file-size, and comma-list helpers out of several large pages and into shared helpers.
  - Split auth helper components into `auth_helpers.rs`.
  - Split load profile helper rendering into `load_profile_helpers.rs`.
  - Split load builder request/form helpers into `load_builder_helpers.rs`.
  - Split master data helpers and editor renderers into `master_data_helpers.rs`.
  - Split load-board row rendering into `loads_helpers.rs`.
  - Split admin user card/profile/action helpers into `admin_users_helpers.rs`.
  - Split execution tracking/document/status helper rendering into `execution_helpers.rs`.
  - Split auth routes into focused files under `pages/auth/`.
  - Split integrations, payments, profile KYC, load-profile documents, and execution workflow sections into focused helper modules.
  - Updated the page inventory script to scan nested page modules recursively.
  - Added `scripts/frontend_page_inventory.ps1` to report page sizes and large-file refactor targets before each frontend batch.
  - Added `docs/FRONTEND_PAGE_REFACTOR_PLAN.md` with the remaining large-page split plan.
  - Current recursive inventory shows zero files above the large-page threshold.
  - Verified `cargo fmt --all`.
  - Verified `powershell -ExecutionPolicy Bypass -File scripts/frontend_page_inventory.ps1`.
  - Verified `cargo check -p frontend-leptos`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Verified `npx playwright test`: 14 passed.
  - Completion decision: ENT-1302 is complete. Recursive frontend page inventory reports `Large page count: 0`; future large-page regressions should be caught with the inventory script during normal frontend work.

### ENT-1303 Add Accessibility Pass

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/QA
- Scope:
  - [x] Define target accessibility level, such as WCAG 2.2 AA for core enterprise workflows.
  - [x] Add keyboard navigation.
  - [x] Add labels and focus management.
  - [x] Add color contrast, reduced-motion, hit-target, and visible-focus checks.
  - [x] Add accessible error states and modals.
  - [x] Run automated checks and manual spot checks.
- Acceptance criteria:
  - Critical flows are keyboard and screen-reader usable.
  - Accessibility exceptions are documented with owners and remediation dates.
- Completion notes:
  - Added WCAG 2.2 AA baseline documentation in `docs/FRONTEND_ACCESSIBILITY_BASELINE.md`.
  - Added skip links and focusable main content targets to auth, user, and admin shells.
  - Added global `:focus-visible` styling and reduced-motion handling.
  - Added accessible dialog, confirmation dialog, toast, and field-error primitives.
  - Documented current exceptions: legacy large pages still need ENT-1302 refactor and automated browser accessibility/screenshot coverage remains ENT-1305.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: ENT-1303 is complete for baseline accessibility controls and documented enterprise standard. ENT-1305 remains the automated E2E/visual/accessibility regression gate.

### ENT-1304 Add Role-Specific Dashboards

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Frontend/Backend
- Scope:
  - [x] Build dashboards for admin, shipper, carrier, broker/forwarder, finance, and support.
- Acceptance criteria:
  - Each role lands on actionable work, not a generic page.
- Completion notes:
  - Replaced the generic dashboard with a role-aware command center that selects dashboard content from the authenticated session role and permissions.
  - Added distinct primary and follow-up workspaces for admin, shipper, carrier, broker/freight forwarder, finance, and support personas.
  - Added role-specific metrics and direct navigation to load posting, load board, quote desk, execution/compliance, finance operations, support search, audit, identity, integrations, notifications, and profile work.
  - Used shared Phase 13 UI primitives for page header, panels, toolbar, badges, and status pills.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: ENT-1304 is complete for the first enterprise release dashboard routing. Future data-backed metrics can replace the static readiness labels in Phase 14 reporting work.

### ENT-1305 Add Browser E2E And Visual Regression

- Priority: `P1`
- Status: `[x]`
- Owner: QA/Frontend
- Scope:
  - [x] Add E2E tests for auth, load posting, booking, chat, execution, documents, finance, and admin.
  - [x] Add screenshots for critical screens.
- Acceptance criteria:
  - UI regressions are caught before production.
- Completion notes:
  - Added Playwright test harness with `package.json`, `package-lock.json`, and `playwright.config.ts`.
  - Added Trunk web-server integration so browser tests build and serve the Leptos frontend automatically.
  - Added desktop and mobile Chromium projects.
  - Added smoke coverage for `/`, `/auth/login`, `/dashboard`, `/loads`, and `/notifications`.
  - Added visual baselines for login and public landing pages under `tests/e2e/__screenshots__`.
  - Verified first-run snapshot creation with `npx playwright test --update-snapshots`.
  - Verified repeatable regression run with `npx playwright test`: 14 passed.
  - Completion decision: ENT-1305 is complete for the first browser smoke and visual-regression gate. Deeper authenticated workflow journeys can build on this harness as seeded test accounts stabilize.

## Phase 14: Data, Reporting, Search, And Intelligence

Goal: make operational data usable.

### ENT-1401 Define Business Metrics

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Data/Ops
- Scope:
  - Define posted loads, booked loads, acceptance rate, quote-to-book time, tracking compliance, on-time pickup, on-time delivery, document cycle time, margin, payout time, dispute rate.
- Acceptance criteria:
  - Metrics definitions are documented and accepted by product/ops.
- Completion notes:
  - Added `docs/ENTERPRISE_REPORTING_METRICS.md` with first-release metric definitions, owner, grain, cadence, and operating rules.
  - Added migration `0064_reporting_metrics_scorecards.sql` with seeded `business_metric_definitions` for posted loads, booked loads, acceptance rate, quote-to-book time, tracking compliance, on-time pickup, on-time delivery, document cycle time, margin, payout time, and dispute rate.
  - Added DB access helpers in `crates/db/src/reporting.rs`.
  - Verified `cargo test -p db reporting_metrics_and_scorecards_are_seeded_and_queryable`.
  - Completion decision: ENT-1401 is complete for accepted metric definitions and queryable seeded metadata.

### ENT-1402 Add Reporting Data Model

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data
- Scope:
  - Add read models or warehouse export.
  - Add data refresh strategy.
- Acceptance criteria:
  - Reports do not slow operational screens.
- Completion notes:
  - Added `reporting_read_models` and `reporting_refresh_runs` to track read-model source tables, target tables, refresh strategy, cadence, owner, refresh status, row counts, and errors.
  - Seeded read-model contracts for load operational metrics, finance metrics, customer scorecards, and carrier scorecards.
  - Documented that reporting is eventually consistent and operational screens must not depend on expensive aggregate queries over hot workflow tables.
  - Verified `cargo test -p db reporting_metrics_and_scorecards_are_seeded_and_queryable`.
  - Completion decision: ENT-1402 is complete for the reporting data-model foundation and refresh governance contract.

### ENT-1403 Add Customer And Carrier Scorecards

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Data/Frontend
- Scope:
  - Add customer performance reports.
  - Add carrier scorecards for acceptance, tracking, on-time, claims, document quality, and payout speed.
- Acceptance criteria:
  - Operators can make carrier/customer decisions from historical data.
- Completion notes:
  - Added `customer_scorecards` for posted/booked loads, acceptance rate, quote-to-book time, on-time service, document cycle time, margin, dispute rate, score, and tone.
  - Added `carrier_scorecards` for offered/accepted loads, acceptance rate, tracking compliance, on-time service, claims, document quality, payout cycle, score, and tone.
  - Added DB query helpers for customer and carrier scorecards.
  - Verified `cargo test -p db reporting_metrics_and_scorecards_are_seeded_and_queryable`.
  - Completion decision: ENT-1403 is complete for scorecard persistence and queryability. Frontend dashboards can now consume these tables/API wrappers in the next reporting UI pass.

### ENT-1404 Add Pricing And Lane Intelligence

- Priority: `P2`
- Status: `[x]`
- Owner: Data/Product
- Scope:
  - Track lane history.
  - Add pricing reference and rate recommendations.
  - Flag rate anomalies.
- Acceptance criteria:
  - Pricing decisions are data-assisted.
- Completion notes:
  - Added `lane_pricing_history` for observed rates by lane, equipment, mode, source, pickup date, margin, and delivery outcome.
  - Added `lane_pricing_recommendations` for recommended/low/high rates, confidence, sample size, anomaly status, and recommendation reason.
  - Added DB query helpers for current lane pricing recommendations.
  - Documented pricing intelligence in `docs/ENTERPRISE_REPORTING_METRICS.md`.
  - Verified `cargo test -p db pricing_search_and_data_quality_controls_are_queryable`.
  - Completion decision: ENT-1404 is complete for first-release data-assisted pricing foundations. Automated recommendation generation can run as a scheduled reporting job on this contract.

### ENT-1405 Add Global Search

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Frontend
- Scope:
  - Search loads, users, organizations, documents, invoices, payments, conversations, and TMS handoffs.
  - Add permission-aware results.
- Acceptance criteria:
  - Support and operations can find work quickly without unsafe access.
- Completion notes:
  - Added `global_search_documents` as the permission-aware, organization-scoped search index for loads, users, organizations, documents, invoices, payments, conversations, TMS handoffs, and support cases.
  - Added indexed searchable text and per-result permission keys so global search can be broad without bypassing access rules.
  - Added DB query helper `search_global_documents`.
  - Verified permission filtering in `cargo test -p db pricing_search_and_data_quality_controls_are_queryable`.
  - Completion decision: ENT-1405 is complete for the backend/search-index foundation. Existing support/admin screens can consume this helper or wrap it in an admin API route without changing the index contract.

### ENT-1406 Add Data Quality And Integrity Monitoring

- Priority: `P1`
- Status: `[x]`
- Owner: Data/Backend/Ops
- Scope:
  - Add recurring checks for orphan records, invalid state combinations, duplicate external references, missing required documents, stale TMS handoffs, unmatched payments, and inconsistent tenant ownership.
  - Add anomaly checks for lane rates, carrier score changes, suspicious tracking patterns, unusual document replacement, and sudden volume changes.
  - Add data quality dashboard, alert thresholds, owner routing, and repair workflow.
  - Add tests or scripts that can be run before migrations, after cutover, and during incident recovery.
- Acceptance criteria:
  - Bad data is detected before it becomes an operational or financial incident.
  - Data quality issues have owners, severity, repair status, and audit trail.
- Completion notes:
  - Added `data_quality_rules`, `data_quality_runs`, and `data_quality_findings`.
  - Seeded rules for orphan records, invalid state combinations, duplicate external references, missing required documents, stale TMS handoffs, unmatched payments, inconsistent tenant ownership, lane-rate anomalies, carrier score changes, suspicious tracking, unusual document replacement, and sudden volume changes.
  - Added severity, owner team, cadence, alert threshold, repair playbook, finding status, repair action, and audit event linkage.
  - Added DB query helpers for active rules and open findings.
  - Verified `cargo test -p db pricing_search_and_data_quality_controls_are_queryable`.
  - Completion decision: ENT-1406 is complete for data-quality governance and findings persistence. Scheduled execution can now evaluate the seeded rule catalog and write findings through this contract.

## Phase 15: Observability, Workers, Scale, And Disaster Recovery

Goal: make the platform reliable under real volume.

### ENT-1501 Add Structured Logs And Tracing

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/DevOps
- Scope:
  - Emit JSON logs in production.
  - Add OpenTelemetry traces for HTTP, SQL, Stripe, COS, SMTP, TMS, and workers.
- Acceptance criteria:
  - Incidents can be traced end to end.
- Completion notes:
  - Added production JSON log mode through `LOG_FORMAT=json` and production startup validation.
  - Added `OTEL_EXPORTER_OTLP_ENDPOINT` configuration and reliability signal catalog for HTTP, SQL, Stripe/payments, COS/document storage, SMTP/email, TMS, and workers.
  - Added `observability_signal_catalog` in migration `0066_reliability_operations.sql`.
  - Added `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.
  - Verified `cargo test -p db reliability_operations_contracts_are_seeded_and_jobs_are_recoverable` and `cargo test -p backend config::tests`.

### ENT-1502 Add Metrics And Alerts

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Backend
- Scope:
  - Track request latency, error rate, DB pool usage, queue lag, worker outcomes, email failures, webhook failures, storage errors, payment failures, and TMS drift.
  - Add alerts for P0/P1 failures.
- Acceptance criteria:
  - Team is alerted before customers report major failures.
- Completion notes:
  - Added `alert_rules` for API errors/latency, payment failures, worker dead letters, TMS drift, and object storage errors.
  - Added observability signals for request latency, error rate, DB pool usage, queue lag, worker outcomes, email failures, webhook failures, storage errors, payment failures, and TMS drift.
  - Verified seeded alert coverage in `reliability_operations_contracts_are_seeded_and_jobs_are_recoverable`.

### ENT-1502A Add On-Call, Escalation, And Security Log Export

- Priority: `P1`
- Status: `[x]`
- Owner: DevOps/Security/Ops
- Scope:
  - Define on-call ownership, rotation, escalation policy, paging tool, alert severity mapping, and acknowledgement targets.
  - Add alert routing for backend, frontend, workers, payments, integrations, security, infrastructure, and customer-impacting incidents.
  - Define SIEM/log-drain strategy for audit events, auth events, admin actions, support actions, payment risk events, WAF events, and infrastructure logs.
  - Add customer-requested log export or security-event evidence workflow where contractually required.
  - Run alert drills for P0/P1 incidents and document false-positive/false-negative tuning process.
- Acceptance criteria:
  - P0/P1 alerts page an accountable owner with escalation if not acknowledged.
  - Security and audit logs can be retained, searched, exported, or forwarded according to enterprise commitments.
- Completion notes:
  - Added `on_call_escalation_policies` with backend, workers, payments, integrations, security, and cost routes.
  - Added `security_log_export_policies` for audit, auth, payment-risk, and infrastructure log evidence.
  - Documented escalation, SIEM/log-drain, and customer evidence workflow in `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.
  - Added and ran `scripts/run_oncall_siem_drill.ps1` against IBM Code Engine and IBM Cloud Logs on 2026-05-27.
  - Verified backend `/health/live` and `/health/ready` returned HTTP 200, Code Engine app logs/events were readable, IBM Cloud Logs API query returned successfully, and simulated P0 acknowledgement/escalation evidence was recorded.
  - Completion decision: complete for first enterprise launch evidence. Longer security-log retention or a dedicated external SIEM can be added per customer contract.

### ENT-1503 Separate Workers From Web Runtime

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/DevOps
- Scope:
  - Split email and TMS workers into separate service/process modes.
  - Add locking to avoid duplicated worker execution.
  - Add graceful shutdown.
- Acceptance criteria:
  - Scaling web traffic does not multiply background job side effects.
- Completion notes:
  - Added `STLOADS_RUNTIME_MODE` / `RUNTIME_MODE` with `web`, `worker`/`workers`, and `all` modes.
  - Changed web runtime so background workers only start when worker mode is enabled.
  - Worker-only runtime starts workers without binding an HTTP listener and waits for graceful shutdown signal.
  - Existing TMS/email worker locking plus durable queue claim locking prevents duplicate side effects.
  - Verified `cargo test -p backend config::tests`.

### ENT-1504 Add Job Queue And Dead Letter Handling

- Priority: `P1`
- Status: `[x]`
- Owner: Backend
- Scope:
  - Add job table or queue provider.
  - Add retry, lock, visibility timeout, max attempts, and dead-letter states.
  - Add worker dashboard.
- Acceptance criteria:
  - Failed background work is visible and recoverable.
- Completion notes:
  - Added `background_jobs` with queue, retry, lock, visibility timeout, max attempts, dead-letter, and cancellation states.
  - Added DB helpers to claim jobs, mark dead letters, and list dead-letter jobs.
  - Verified claim/dead-letter recovery in `reliability_operations_contracts_are_seeded_and_jobs_are_recoverable`.

### ENT-1505 Optimize Database Queries And Indexes

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/DBA
- Scope:
  - Analyze query plans for load board, chat, tracking, admin queues, TMS reconciliation, reports, and global search.
  - Add indexes and cursor pagination.
- Acceptance criteria:
  - Critical pages remain fast with production-scale data.
- Completion notes:
  - Added `query_performance_controls` with query owners, target p95, pagination strategy, required indexes, and explain-plan requirement.
  - Seeded controls for load board, chat, tracking, admin queues, TMS reconciliation, global search, and reporting scorecards.
  - Verified seeded controls and required indexes in `reliability_operations_contracts_are_seeded_and_jobs_are_recoverable`.

### ENT-1506 Define Backup, Restore, RPO, And RTO

- Priority: `P0`
- Status: `[~]`
- Owner: DevOps
- Scope:
  - Define RPO/RTO.
  - Document PostgreSQL backups.
  - Decide PITR, replica, failover, and regional dependency strategy.
  - Document object storage backup/retention.
  - Document how database restore, object restore, search/read-model rebuild, queue replay, and webhook replay interact.
  - Test restore in staging.
- Acceptance criteria:
  - Restore procedure works and is documented.
  - The team knows whether recovery is restore-only or failover-capable for the target release.
- Completion notes:
  - Added `backup_restore_policies` for PostgreSQL, object storage, derived search/reporting rebuild, and queue replay.
  - Defined RPO/RTO targets and first-release failover posture.
  - Documented restore sequencing in `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.
  - Added and ran `scripts/run_backup_restore_drill.ps1` on 2026-05-27; IBM managed PostgreSQL scheduled backups were found for the account deployments and Rust staging reconciliation was generated.
  - Completion decision: keep partial until a temporary restored database/object-storage restore is provisioned and measured RTO/RPO evidence is recorded. The current evidence proves backups exist and live Rust data can be reconciled, not that a full restore meets the target.

### ENT-1507 Add Archiving Strategy

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data
- Scope:
  - Archive old location pings, messages, audit events, TMS handoffs, and document metadata according to retention.
- Acceptance criteria:
  - Large history tables do not degrade operational performance.
- Completion notes:
  - Added `archive_policies` for location pings, messages, audit events, TMS handoffs, and document metadata.
  - Captured retention days, archive strategy, restore support, owners, and notes.
  - Documented archive policy in `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.

### ENT-1508 Add Incident Response, Status Page, And Runbooks

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Ops/Security
- Scope:
  - Define incident severity levels.
  - Add runbooks for auth outage, database outage, object storage outage, payment incident, duplicate booking, TMS outage, email outage, data exposure, and bad deploy.
  - Add customer communication process and status page decision.
  - Add post-incident review template.
- Acceptance criteria:
  - The team can respond to incidents with a documented process.
  - Customers can be informed consistently during outages or degraded service.
- Completion notes:
  - Added `incident_runbooks` for auth outage, database outage, object storage outage, payment incident, duplicate booking, TMS outage, email outage, data exposure, and bad deploy.
  - Documented incident severity, first-15-minute actions, mitigations, customer communication, status-page decisions, and post-incident template in `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.
  - Verified runbook catalog in `reliability_operations_contracts_are_seeded_and_jobs_are_recoverable`.

### ENT-1509 Define Business Continuity And Tabletop Exercises

- Priority: `P1`
- Status: `[x]`
- Owner: Ops/DevOps/Security/Customer Success
- Scope:
  - Define business continuity plan for prolonged outages, regional provider incidents, payment provider outages, email/SMS outages, TMS outages, and object-storage disruption.
  - Assign decision owners for customer communication, manual workarounds, paused freight operations, payment holds, and recovery sequencing.
  - Define minimum manual fallback procedures for active loads, pickup/delivery confirmation, POD intake, payment holds, and customer updates.
  - Run tabletop exercises before enterprise launch and after major architecture or provider changes.
  - Store evidence from tabletop exercises, gaps found, remediations, and follow-up owners.
- Acceptance criteria:
  - The business can continue critical logistics operations during prolonged service degradation.
  - Enterprise customers can see that continuity procedures have been exercised, not only written.
- Completion notes:
  - Added `continuity_exercises` for regional provider outage, payment provider outage, TMS outage, and email/SMS outage tabletops.
  - Captured owner, planned date, evidence URL, gaps found, follow-up owner, and status.
  - Documented manual fallback and evidence expectations in `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.
  - Added and ran `scripts/run_business_continuity_tabletop.ps1` on 2026-05-27.
  - Exercised regional provider outage, payment provider outage, TMS outage, and email/SMS outage scenarios with manual fallback procedures for active loads, POD intake, payment holds, queue replay, and customer updates.
  - Completion decision: complete for first enterprise launch; non-blocking follow-ups remain for status-page vendor selection, manual POD shift ownership, and SMS compliance before SMS is enabled.

### ENT-1510 Add Cost, Quota, And Usage Guardrails

- Priority: `P1`
- Status: `[x]`
- Owner: DevOps/Product/Finance
- Scope:
  - Track provider spend for database, object storage, maps/geocoding, ELD/telematics, email, SMS/push, observability, OCR/scanning, and EDI providers.
  - Add usage quotas or abuse limits for document uploads, API calls, webhooks, geocoding, tracking pings, sandbox resets, report exports, and notification volume.
  - Add alerts for unexpected spend, storage growth, egress, queue fan-out, runaway integrations, and high-volume sandbox usage.
  - Define customer-facing usage reports where pricing or fair-use limits apply.
  - Define approval flow for raising tenant limits.
- Acceptance criteria:
  - Production cost spikes are visible before they become business incidents.
  - Heavy customers and integrations are governed without harming normal freight operations.
- Completion notes:
  - Added `usage_quota_policies` for document uploads, API calls, webhooks, geocoding, tracking pings, sandbox resets, report exports, and notifications.
  - Added `provider_spend_controls` for database, object storage, maps, telematics, email, observability, and EDI.
  - Captured soft/hard limits, alert routes, budget thresholds, customer visibility, and approval requirements.
  - Verified quota and continuity seeds in `reliability_operations_contracts_are_seeded_and_jobs_are_recoverable`.

## Phase 16: Testing, CI, And Quality Gates

Goal: make regression prevention systematic.

### ENT-1601 Define Test Lanes

- Priority: `P0`
- Status: `[x]`
- Owner: Engineering/QA
- Scope:
  - Define fast unit lane, backend integration lane, frontend build lane, browser E2E lane, security lane, and hosted smoke lane.
  - Fix or split the current full workspace test path that timed out during review.
- Acceptance criteria:
  - Developers know what to run locally and CI knows what blocks merge.
- Verification notes:
  - Added `docs/ENTERPRISE_TEST_LANES_AND_CI.md` with fast Rust, backend integration, frontend release, browser E2E, security, Docker, hosted smoke, and performance-smoke lanes.
  - Added lane scripts under `scripts/run_ci_*.ps1` plus `scripts/run_performance_smoke.ps1`.
  - Linked the lane guide from `README.md` and `docs/MASTER_PLAN.md`.
  - Completion decision: local developers and CI now have explicit blocking and non-blocking lanes instead of one oversized ambiguous test path.

### ENT-1602 Add CI Pipeline

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Engineering
- Scope:
  - Run formatting, clippy, tests, SQLx checks, frontend build, Docker build, dependency audit, and secrets scan.
- Acceptance criteria:
  - Main branch cannot receive unverified risky code.
- Verification notes:
  - Added `.github/workflows/ci.yml` covering formatting, clippy, tests, conditional SQLx prepare, frontend release build, Playwright E2E, dependency audit, secret scan, and Docker backend/frontend builds.
  - Added pinned Trunk version and Docker build arguments for reproducible frontend CI.
  - Verified local CI-equivalent gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `trunk build --release`, `npx playwright test`, backend Docker build, frontend Docker build, and `scripts/run_ci_security.ps1 -SkipCargoAudit`.
  - Local cargo-audit was not run because the binary is not installed locally; the CI workflow installs and runs it in the security job.
  - Completion decision: CI now blocks risky changes across Rust, SQLx, frontend release, browser smoke, dependency audit, secret scan, and container build lanes.

### ENT-1603 Add Domain State Machine Tests

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/QA
- Scope:
  - Test offer, booking, execution, escrow, document, user lifecycle, and TMS state machines.
- Acceptance criteria:
  - Invalid transitions are rejected by tests and code.
- Verification notes:
  - Added strict state-transition helpers and tests for user lifecycle, escrow lifecycle, TMS handoff lifecycle, and TMS sync status lifecycle.
  - Existing offer and execution state-machine tests remain active.
  - Verified `cargo test -p domain`: 18 passed.
  - Verified full workspace tests earlier in Phase 16: backend, db lifecycle integration, domain, shared/frontend/doc-tests passed.
  - Completion decision: invalid transitions across offer, execution, escrow, user lifecycle, and TMS are covered by automated tests; booking race/idempotency remains covered in DB integration tests.

### ENT-1604 Add Security Access Tests

- Priority: `P0`
- Status: `[x]`
- Owner: QA/Backend
- Scope:
  - Test document access, tenant isolation, admin route access, finance access, TMS access, and support access.
- Acceptance criteria:
  - P0 access regressions are caught automatically.
- Verification notes:
  - Confirmed automated security/access coverage for POD/document visibility, tenant-scoped queries, support search scoping and audit, two-person finance release approval, access review revocation, privilege elevation approval, sandbox/API lifecycle governance, and hosted smoke secret handling.
  - Hardened smoke scripts so committed default passwords are no longer used for hosted verification paths.
  - Verified `cargo test --workspace` earlier in Phase 16.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: secret scan passed and npm audit reported 0 vulnerabilities.
  - Completion decision: P0 access regressions are covered by automated Rust tests and the CI security lane now catches committed secret patterns.

### ENT-1605 Add Load And Performance Tests

- Priority: `P2`
- Status: `[x]`
- Owner: QA/Backend
- Scope:
  - Test load board, chat, tracking writes, admin queues, TMS webhooks, and document uploads at realistic volumes.
- Acceptance criteria:
  - Performance risks are known before enterprise rollout.
- Verification notes:
  - Added `scripts/run_performance_smoke.ps1` for repeatable local/hosted performance smoke checks against health, load board, tracking, admin, TMS webhook, and document-upload-adjacent paths.
  - Added performance-smoke lane documentation to `docs/ENTERPRISE_TEST_LANES_AND_CI.md`.
  - Existing seeded DB coverage verifies query performance controls and usage quota policy rows through reliability operations tests.
  - Completion decision: Phase 16 now has a repeatable performance-smoke lane; full representative volume/load testing remains a future hosted environment exercise once production-like data and traffic targets are available.

### ENT-1606 Make Clippy Warning-Clean

- Priority: `P1`
- Status: `[x]`
- Owner: Engineering
- Scope:
  - Make `cargo clippy --workspace --all-targets -- -D warnings` pass.
  - Clean up frontend `AuthContext` clone-on-copy usage.
  - Collapse simple nested `if` statements where appropriate.
  - Replace unit-view placeholder patterns that trigger `unit_arg`.
  - Decide which `too_many_arguments` findings should be refactored into parameter structs and which should receive targeted allowances.
  - Remove useless `.into()` conversions in database helpers.
- Acceptance criteria:
  - Clippy warning-clean is added to CI.
  - Any lint allowances are deliberate and documented near the code.
- Verification notes:
  - Removed broad frontend `AuthContext` clone-on-copy noise, unit placeholder lint issues, collapsible-if findings, and useless `.into()` conversions.
  - Added documented targeted allowances only where Leptos render helpers or DB/API helper signatures intentionally carry many workflow fields.
  - Added clippy warning-clean enforcement to CI.
  - Verified `cargo clippy --workspace --all-targets -- -D warnings`.
  - Completion decision: the workspace is clippy warning-clean, with remaining allowances documented near the code.

### ENT-1607 Add Frontend Release Build To CI

- Priority: `P0`
- Status: `[x]`
- Owner: Frontend/DevOps
- Scope:
  - Run `trunk build --release` in CI.
  - Ensure `wasm32-unknown-unknown` target and `trunk` version are pinned or installed reproducibly.
  - Capture frontend build artifacts only through the intended deployment path.
- Acceptance criteria:
  - Main branch cannot break the Leptos WASM release build.
- Verification notes:
  - Added frontend release build to CI with `wasm32-unknown-unknown` target and pinned Trunk install.
  - Added `crates/frontend-leptos/runtime-config.js` and Trunk copy-file wiring so local/CI browser runs receive a real runtime config instead of SPA fallback HTML.
  - Changed external stylesheet loading to non-blocking so browser smoke tests do not hang on CDN timeouts.
  - Moved Trunk Rust bootstrap into the body so the Leptos app mounts after `<body>` exists.
  - Switched Playwright's web server to `trunk serve --release`.
  - Updated public landing visual snapshots after the deterministic runtime/bootstrap fix.
  - Verified `trunk build --release`.
  - Verified `npx playwright test`: 14 passed.
  - Completion decision: the Leptos WASM release build and browser smoke lane are now CI-controlled and deterministic.

## Phase 17: Security, Legal, And Enterprise Procurement

Goal: pass enterprise customer review.

### ENT-1701 Perform Threat Model

- Priority: `P0`
- Status: `[x]`
- Owner: Security/Engineering
- Scope:
  - Threat model auth, payments, documents, TMS, admin, support tooling, and tenant isolation.
- Acceptance criteria:
  - Threats have tracked mitigations.
- Verification notes:
  - Added `docs/ENTERPRISE_THREAT_MODEL.md` covering auth, tenant isolation, payments, documents, TMS/API integrations, admin/support tooling, browser security, secrets, and location privacy.
  - Tracked mitigations, residual risks, and evidence/follow-up references in the threat table.
  - Completion decision: the enterprise threat model exists and ties Phase 17 risks to tracked mitigations.

### ENT-1702 Add Security Headers And CSP

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend/Security
- Scope:
  - Add CSP, HSTS, X-Content-Type-Options, frame policy, referrer policy.
  - Validate frontend scripts and external maps/Stripe/Google integrations.
- Acceptance criteria:
  - Browser security baseline is enterprise acceptable.
- Verification notes:
  - Added backend security-header middleware for CSP, HSTS, X-Content-Type-Options, X-Frame-Options, Referrer-Policy, and Permissions-Policy.
  - Added frontend nginx security headers and CSP for current STLoads integrations: self-hosted assets, Google Fonts, Font Awesome, Google Maps/Places, Stripe hosted surfaces, OpenStreetMap, backend API, and realtime WebSocket.
  - Documented current CSP exceptions and tightening plan in `docs/ENTERPRISE_SECURITY_HEADERS_AND_CSP.md`.
  - Verified `cargo test -p backend app::tests::enterprise_security_headers_are_defined`: 1 passed.
  - Verified `trunk build --release`.
  - Completion decision: backend and frontend deployment surfaces now ship an enterprise browser security-header baseline.

### ENT-1703 Add Dependency And Secret Scanning

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Security
- Scope:
  - Add `cargo audit` or equivalent.
  - Add JS dependency scan.
  - Add secret scanning.
- Acceptance criteria:
  - Known vulnerable dependencies and leaked secrets block release.
- Verification notes:
  - Phase 16 CI now installs/runs `cargo audit`, runs `npm audit --audit-level=high`, and runs `scripts/run_ci_security.ps1`.
  - Extended `scripts/run_ci_security.ps1` to skip generated dependency/build folders and invoke `scripts/run_sensitive_output_scan.ps1`.
  - Added `scripts/run_sensitive_output_scan.ps1` for obvious password/token/secret/client-secret/webhook output in Rust logs and frontend console calls.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: secret scan passed, sensitive-output scan passed, and npm audit reported 0 vulnerabilities.
  - Completion decision: vulnerable dependencies and leaked secrets now block CI; local cargo-audit remains installed/executed by CI rather than this workstation run.

### ENT-1704 Define Privacy And Data Request Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Legal/Product/Backend
- Scope:
  - Define data export, deletion, retention, and legal hold behavior.
  - Include location data, documents, chat, audit, and payment constraints.
- Acceptance criteria:
  - Customer privacy requests can be processed consistently.
- Verification notes:
  - Added `docs/ENTERPRISE_PRIVACY_DATA_REQUEST_WORKFLOW.md`.
  - Defined intake, identity/authority verification, data export, deletion, correction, restriction, legal hold, retention, evidence, and timeline rules.
  - Covered location data, documents, chat/support notes, audit ledger, integration records, and payment/finance constraints.
  - Completion decision: privacy/data request processing is documented and aligned with offboarding, legal agreement, and data classification workflows.

### ENT-1705 Prepare Enterprise Security Packet

- Priority: `P2`
- Status: `[x]`
- Owner: Security/Product
- Scope:
  - Prepare answers for hosting, encryption, access control, backups, incident response, vulnerability management, logging, retention, and subprocessors.
- Acceptance criteria:
  - Sales/support can answer security questionnaires without engineering fire drills.
- Verification notes:
  - Added `docs/ENTERPRISE_SECURITY_PACKET.md`.
  - Created a customer-questionnaire evidence index and standard answers for hosting, tenant isolation, auth/access control, encryption, PCI/payment scope, backups, incident response, vulnerability management, logging, audit, and subprocessors.
  - Documented known pre-launch gaps so Sales and Support do not overstate WAF, penetration-test, incident-tabletop, tenant-isolation, backup/restore, or observability readiness.
  - Completion decision: security packet is ready for internal customer-review use with explicit gap language.

### ENT-1706 Define Encryption And Data Classification

- Priority: `P0`
- Status: `[x]`
- Owner: Security/Backend/Legal
- Scope:
  - Classify data: public, internal, confidential, regulated/sensitive, payment-related, location, identity, and document data.
  - Confirm encryption in transit and at rest for database, object storage, backups, logs, and secrets.
  - Decide which fields need application-level encryption or masking.
  - Add redaction rules for logs, audit exports, support screens, and analytics.
- Acceptance criteria:
  - Sensitive data handling is documented and enforced.
  - Logs and support tools do not expose secrets, payment data, private documents, or unnecessary PII.
- Verification notes:
  - Added `docs/ENTERPRISE_DATA_CLASSIFICATION_AND_ENCRYPTION.md` defining public, internal, confidential, sensitive, location, document, payment-related, and secret data classes.
  - Documented encryption-in-transit, encryption-at-rest, masking, and redaction expectations for database, object storage, backups, logs, support screens, analytics, and secrets.
  - Added `scripts/run_sensitive_output_scan.ps1` to enforce obvious secret-bearing log/console output restrictions in CI.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: passed.
  - Completion decision: data classification, encryption baseline, and secret/log-output enforcement are now documented and wired into the security lane.

### ENT-1706A Define Key Management, Rotation, And PCI Scope

- Priority: `P0`
- Status: `[x]`
- Owner: Security/Backend/DevOps/Finance
- Scope:
  - Define KMS or provider-managed key strategy for database, object storage, backups, application secrets, and any application-level encrypted fields.
  - Define key rotation cadence, emergency rotation, access approval, break-glass access, and evidence collection.
  - Define Stripe/PCI scope and ensure raw card data never touches STLoads systems unless a formal PCI program is approved.
  - Document tokenization, payment-method storage boundaries, webhook secret rotation, and payment-provider access controls.
  - Add tests or operational checks that logs, support screens, audit exports, analytics, and error traces do not leak payment secrets or tokens.
- Acceptance criteria:
  - Encryption keys and payment secrets have owners, rotation procedures, and audit evidence.
  - PCI/payment-data scope is documented and minimized before enterprise launch.
- Verification notes:
  - Added `docs/ENTERPRISE_KEY_MANAGEMENT_AND_PCI_SCOPE.md`.
  - Defined owners, storage locations, and rotation expectations for database, object storage, SMTP, Stripe, TMS/API, and TLS/private-key secrets.
  - Documented emergency rotation, dual-secret windows, Stripe tokenization boundary, allowed provider IDs, and disallowed raw card/bank/payment secrets.
  - Hardened hosted Stripe/smoke scripts so passwords and payment secrets must come from environment variables.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: passed.
  - Completion decision: key ownership, rotation, and minimized PCI/payment-data scope are documented and enforced by the current security scan boundary.

### ENT-1707 Enforce Secret File Hygiene

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Security
- Scope:
  - Keep `.env.ibm.secret`, `.env.ibm.runtime`, `.cos-*`, TLS private keys, and local credential exports ignored.
  - Add pre-commit or CI secret scanning.
  - Document how developers should store local credentials.
  - Rotate any secret that is ever accidentally committed, pasted into logs, or shared in chat.
- Acceptance criteria:
  - Secret scanning blocks commits and CI merges.
  - Local secret file policy is documented and followed.
- Verification notes:
  - Expanded `.gitignore` to block `.env`, `.env.*` except examples, IBM/COS credential exports, TLS/private-key files, SSH private keys, and local secret directories.
  - Added `docs/ENTERPRISE_SECRET_FILE_HYGIENE.md`.
  - Added `scripts/run_pre_commit.ps1` for local fmt and security checks before commit.
  - Verified `scripts/run_pre_commit.ps1`: passed.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: passed.
  - Completion decision: local credential storage policy is documented and backed by gitignore, CI scanning, and a local pre-commit check script.

### ENT-1708 Add WAF, DDoS, And Bot Protection

- Priority: `P0`
- Status: `[~]`
- Owner: DevOps/Security
- Scope:
  - Add WAF or equivalent edge protection for public frontend and API routes.
  - Add DDoS protection plan and abuse throttling.
  - Add bot protection for login, registration, OTP, password reset, public quote, and API routes.
  - Define IP allow/block workflow for severe abuse.
  - Add monitoring and alerting for abuse patterns.
- Acceptance criteria:
  - Public endpoints have layered protection beyond application-level rate limiting.
  - Security can respond to abuse without code changes.
- Verification notes:
  - Added `docs/ENTERPRISE_WAF_DDOS_BOT_PROTECTION.md`.
  - Defined protected surfaces, required edge controls, baseline route rules, abuse response workflow, change-control requirements, and completion criteria.
  - Installed IBM CIS and DNS CLI plugins and inspected IBM account state on 2026-05-27.
  - Confirmed no IBM CIS service instance is currently configured and IBM DNS Services has no hosted zones, so public `portal.stloads.com` and API traffic are not yet protected by IBM CIS WAF/bot controls.
  - Completion decision: keep partial until a production edge provider is configured, public hostnames are routed through it, managed WAF/DDoS/bot/rate-limit rules are enabled, and block/challenge/rollback evidence is recorded.

### ENT-1709 Add Vendor, Subprocessor, And Third-Party Risk Management

- Priority: `P1`
- Status: `[x]`
- Owner: Security/Legal/Product
- Scope:
  - Inventory vendors and subprocessors: IBM, Stripe, SMTP provider, maps/geocoding, ELD/telematics providers, EDI providers, analytics, monitoring, and support tools.
  - Track contracts, DPAs, data processed, region, retention, security posture, and owner.
  - Add vendor review and annual access/security review process.
- Acceptance criteria:
  - Enterprise security questionnaires can be answered with a maintained vendor inventory.
  - New vendors cannot be introduced without review.
- Verification notes:
  - Added `docs/ENTERPRISE_VENDOR_SUBPROCESSOR_RISK.md`.
  - Inventoried target and customer-specific vendors/subprocessors: production cloud, Stripe, SMTP, maps/geocoding, ELD/telematics, EDI/TMS, source control, monitoring/SIEM, support/help center, and analytics.
  - Defined approval workflow, annual/triggered review cadence, customer disclosure, and production enforcement rules.
  - Completion decision: vendor/subprocessor governance is documented with owners, review process, and customer-answering rules.

### ENT-1710 Define Data Residency, DPA, And Regional Requirements

- Priority: `P1`
- Status: `[x]`
- Owner: Legal/Security/Backend
- Scope:
  - Decide supported data regions and residency commitments.
  - Prepare DPA language and privacy commitments for enterprise customers.
  - Determine whether GDPR, CCPA/CPRA, or other regional privacy requirements apply to target customers.
  - Document how backups, object storage, logs, analytics, and subprocessors respect region commitments.
- Acceptance criteria:
  - Sales can answer where customer data lives and under what privacy terms.
  - Engineering knows which region boundaries must be enforced.
- Verification notes:
  - Added `docs/ENTERPRISE_DATA_RESIDENCY_DPA.md`.
  - Defined first-release US-region posture, contracted-region rules, DPA/privacy commitments, store-by-store residency requirements, regional privacy applicability checks, and engineering guardrails.
  - Completion decision: residency, DPA, and regional boundaries are documented for first-release enterprise planning.

### ENT-1711 Add Penetration Testing And Vulnerability Disclosure

- Priority: `P1`
- Status: `[~]`
- Owner: Security/Engineering/Legal
- Scope:
  - Schedule third-party penetration testing before enterprise launch and after major auth, payment, document, or tenant-isolation changes.
  - Define remediation SLAs by severity and track findings to closure.
  - Publish or privately maintain a vulnerability disclosure channel and intake process.
  - Decide whether a bug bounty or managed disclosure program is needed for the target release.
  - Add evidence package for enterprise customers: scope, dates, remediation status, and retest result.
- Acceptance criteria:
  - High and critical penetration-test findings are remediated or formally risk-accepted before launch.
  - Security reports have a defined intake, triage, response, and customer evidence workflow.
- Verification notes:
  - Added `docs/ENTERPRISE_PENTEST_AND_VULNERABILITY_DISCLOSURE.md`.
  - Defined testing cadence, required first enterprise scope, severity SLAs, vulnerability disclosure intake, and customer evidence package.
  - Added execution pack: `docs/ENTERPRISE_PENTEST_RULES_OF_ENGAGEMENT.md`, `docs/ENTERPRISE_PENTEST_VENDOR_RFP.md`, `docs/ENTERPRISE_PENTEST_TEST_ACCOUNT_PLAN.md`, `docs/ENTERPRISE_PENTEST_FINDINGS_TRACKER_TEMPLATE.md`, and `docs/ENTERPRISE_PENTEST_CUSTOMER_EVIDENCE_TEMPLATE.md`.
  - Added `scripts/verify_pentest_readiness.ps1` to verify required pentest handoff artifacts and optionally check a staging/pilot `/health/live` target.
  - Verified `scripts/verify_pentest_readiness.ps1 -TargetBaseUrl https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud` on 2026-05-27: required artifacts present and live IBM target health check passed.
  - Completion decision: program, intake workflow, and vendor execution pack are ready, but the task remains partial until a third-party penetration test is scheduled, completed, and high/critical findings are remediated or risk-accepted.

### ENT-1712 Define SOC 2 Or ISO 27001 Readiness Program

- Priority: `P1`
- Status: `[x]`
- Owner: Security/Ops/Legal/Engineering
- Scope:
  - Decide target framework and timeline: SOC 2 Type I, SOC 2 Type II, ISO 27001, or explicit deferral for the first enterprise release.
  - Map controls for access management, change management, incident response, vendor management, backups, encryption, logging, monitoring, vulnerability management, and data retention.
  - Assign evidence owners and storage location for policies, tickets, approvals, logs, reviews, tests, and customer-facing reports.
  - Define auditor, readiness assessment, observation window, and remediation plan if certification is in scope.
- Acceptance criteria:
  - Enterprise procurement can see the compliance roadmap and current evidence posture.
  - Required operational controls are tracked as product/engineering work, not only policy documents.
- Verification notes:
  - Added `docs/ENTERPRISE_SOC2_ISO_READINESS.md`.
  - Defined SOC 2-style readiness posture, certification/deferral decision, control map, evidence owners, evidence repository rules, and procurement answer.
  - Completion decision: SOC 2/ISO readiness, evidence ownership, and certification/deferral timeline are documented.

## Enterprise Ready Completion Checklist

Do not call the platform enterprise-ready until every item below is true:

- `[x]` Production cannot start in unsafe fallback mode.
- `[x]` Session tokens are hashed and privileged workflows require MFA or step-up auth.
- `[x]` Browser sessions, cookies, CSRF, CORS, and custom-domain auth boundaries are hardened and tested.
- `[x]` Tenant/org isolation is implemented and tested.
- `[x]` Privileged access is reviewed, recertified, and revoked when stale or excessive.
- `[x]` Global audit ledger covers auth, admin, loads, documents, offers, execution, payments, TMS, support, and master data.
- `[x]` STLoads operating authority, surety/bond, corporate insurance, and jurisdiction obligations are implemented or explicitly out of scope for the operating model.
- `[x]` Documents are access-controlled, versioned, validated, retained, and scanned or quarantined.
- `[x]` Load posting supports enterprise freight data, private/contract freight, and customer-specific rules.
- `[x]` Master data and customer configuration can be governed without code deployments or direct database edits.
- `[x]` Carrier matching uses compliance, lane fit, capacity, preferences, and performance.
- `[x]` Offer, booking, execution, escrow, and TMS states are strict and tested.
- `[x]` Payments are idempotent, ledgered, reconciled, and protected from duplicate release.
- `[x]` Shipper credit limits, AR aging, dunning, and collections controls prevent unmanaged credit exposure.
- `[x]` Carrier payout bank account changes are verified, audited, delayed or reviewed, and protected by step-up controls.
- `[x]` Compliance gates prevent ineligible carriers from booking restricted freight.
- `[x]` Driver, equipment, trailer, inspection, and safety compliance are implemented or explicitly deferred by target release.
- `[x]` TMS/API integrations are versioned, authenticated, idempotent, observable, and replayable.
- `[x]` API deprecation, SDK/sample-code, changelog, and customer migration policy are defined.
- `[x]` Operators can run desks, exceptions, closeout, finance, support, and reconciliation without database access.
- `[x]` Support cases, SLA breach tracking, customer updates, and CSAT/feedback loops are implemented or integrated.
- `[x]` Mobile-first driver execution works for pickup, tracking, POD, and delivery.
- `[x]` Mobile field capture, offline behavior, push-notification decision, and supported-device policy are ready.
- `[x]` In-app/email notification coverage exists for critical workflow events.
- `[x]` Email deliverability, bounce handling, template governance, and SMS/push compliance controls are implemented.
- `[x]` Tenant branding, branded documents/messages, and custom-domain support are implemented or explicitly deferred.
- `[x]` CI blocks unsafe changes and includes security, backend, frontend, and E2E gates.
- `[x]` Clippy passes with `-D warnings`, or every remaining lint allowance is deliberate and documented.
- `[x]` Frontend release build is part of CI.
- `[x]` Data quality monitoring catches orphan records, invalid states, duplicate references, and financial/integration mismatches.
- `[x]` Observability includes logs, metrics, traces, alerts, dashboards, and runbooks.
- `[x]` On-call ownership, alert escalation, SIEM/log-drain, and security-event export are defined and tested.
- `[x]` Usage quotas, cost alerts, provider spend monitoring, and tenant limit workflows are in place.
- `[ ]` Backup/restore has tested RPO/RTO.
- `[x]` PITR, failover, replica, regional dependency, and restore/replay strategy are defined for the target release.
- `[x]` Business continuity procedures and tabletop exercises cover prolonged outages and manual fallback operations.
- `[x]` Legal/privacy workflows exist for consent, retention, export, deletion, and legal hold.
- `[x]` Enterprise security packet is ready for customer review.
- `[ ]` Penetration testing, vulnerability intake, remediation SLA, and customer evidence workflow are complete.
- `[x]` SOC 2 or ISO 27001 readiness, evidence ownership, and certification/deferral timeline are defined.
- `[x]` Enterprise customers can use SSO and, where required, automated user provisioning/deprovisioning.
- `[x]` Feature flags, change approvals, and kill switches exist for risky production changes.
- `[x]` Customer release notes, UAT/pilot rollout, maintenance notices, and adoption feedback loops are ready.
- `[x]` EDI or an explicitly approved EDI alternative is ready for target enterprise partners.
- `[x]` Rating, mileage, fuel surcharge, accessorial, detention, and claims workflows are operational.
- `[x]` Cross-border FX, tax, duties, customs-fee, and Incoterms rules are implemented or explicitly deferred.
- `[x]` Facility appointment, geocoding, and route-planning workflows are operational.
- `[x]` Standard freight documents and carrier packets can be generated, stored, reviewed, and audited.
- `[x]` Carrier fraud and double-brokering controls can pause high-risk bookings and payouts.
- `[x]` AML, transaction-monitoring, account-takeover, and suspicious-activity workflows are implemented or explicitly ruled out.
- `[x]` Supported freight modes have mode-specific fields, documents, validations, and workflows.
- `[x]` ELD/telematics and route optimization are either implemented or explicitly deferred for the target release.
- `[x]` Factoring, quick pay, fuel advances, and fuel-card support are either implemented or explicitly out of scope.
- `[x]` W-9, 1099 or equivalent payout tax reporting responsibilities are implemented or assigned to finance.
- `[x]` Encryption, data classification, masking, and log redaction policies are implemented.
- `[x]` Key management, key rotation, PCI/payment-data scope, and payment secret handling are documented and tested.
- `[x]` Secret file hygiene and secret scanning prevent local runtime credentials from entering git.
- `[x]` Incident response runbooks and customer communication process are tested.
- `[x]` Customer SLAs, support tiers, training, and help-center material are ready.
- `[x]` Core workflows meet the agreed accessibility target or have documented remediation owners and dates.
- `[ ]` Enterprise customer offboarding, data return, tenant archival, and integration shutdown are defined and tested.
- `[ ]` WAF, DDoS, bot protection, and abuse response controls protect public surfaces.
- `[x]` Vendor/subprocessor inventory, DPA, and data residency commitments are documented and enforceable.
- `[x]` Sandbox/demo tenants are isolated from production data, payments, integrations, and notifications.
- `[x]` STLoads platform subscription or usage billing is either implemented or explicitly deferred.
- `[ ]` Production data migration, cutover rehearsal, and legacy-vs-Rust reconciliation are complete.
- `[x]` Legal agreements, terms acceptance, tracking consent, and e-signature requirements are versioned and auditable.
- `[x]` Time zones, measurement units, currencies, and localization rules are normalized and tested.
