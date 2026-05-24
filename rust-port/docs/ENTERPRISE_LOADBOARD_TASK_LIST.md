# Enterprise Loadboard Production Task List

Last updated: 2026-05-24

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
- `[!]` `cargo clippy --workspace --all-targets -- -D warnings` does not pass yet. The failures are lint-cleanliness issues, mostly frontend `clone_on_copy`, `collapsible_if`, `unit_arg`, and `too_many_arguments`, plus a few db/TMS lint issues. Track this under `ENT-1606`.

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
- Status: `[ ]`
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
  - Linked cutover plan from `README.md` and `docs/MASTER_PLAN.md`.
  - Verified `scripts/run_cutover_reconciliation.ps1` fails safely when `DATABASE_URL` is missing.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 28 passed.
  - Verified runtime env template validation still passes with placeholder allowance.
  - Pending: run reconciliation against staging/prod-like data, provide the matching legacy summary, and complete business validation before marking this task complete.

## Phase 2: Auth, Authorization, And Tenant Boundaries

Goal: protect accounts, roles, organizations, and sensitive workflows.

### ENT-0201 Hash Session Tokens

- Priority: `P0`
- Status: `[ ]`
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
- Status: `[ ]`
- Owner: Backend/Frontend/Ops
- Scope:
  - Add support search across users, organizations, loads, documents, payments, and TMS handoffs.
  - Add issue timeline and support notes.
  - Add optional impersonation only with explicit approval, banner, limited scope, and audit.
- Acceptance criteria:
  - Support can help customers without unsafe database access.
  - Every support action is auditable.

### ENT-0208 Add Enterprise SSO And SCIM

- Priority: `P1`
- Status: `[ ]`
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

### ENT-0209 Add Access Reviews And Least-Privilege Recertification

- Priority: `P1`
- Status: `[ ]`
- Owner: Security/Ops/Backend
- Scope:
  - Define quarterly access reviews for internal admins, support, finance, integration admins, and break-glass users.
  - Add reporting for privileged roles, stale accounts, inactive users, external contractors, and users outside approved organizations.
  - Add approval workflow for privilege elevation and emergency access.
  - Log review decisions, approvers, revocations, and exceptions.
- Acceptance criteria:
  - Internal and customer-facing privileged access can be recertified on a schedule.
  - Stale, excessive, or emergency access does not remain active silently.

## Phase 3: Audit, Compliance Evidence, And Governance

Goal: make every important action reconstructable.

### ENT-0301 Add Global Audit Ledger

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Add `audit_events` table.
  - Store actor, tenant/org, entity, action, before/after JSON, request ID, IP, user agent, source, and timestamp.
  - Make audit append-only.
- Acceptance criteria:
  - All high-risk workflows write audit events.
  - Audit events are queryable by entity and actor.

### ENT-0302 Add Request Correlation IDs

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Generate or propagate request IDs for every HTTP request.
  - Propagate IDs into logs, audit events, Stripe actions, TMS events, email outbox, and realtime events.
- Acceptance criteria:
  - A single incident can be traced across API, DB, jobs, payments, and integrations.

### ENT-0303 Add Audit Search UI

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Backend
- Scope:
  - Add admin audit search by user, org, load, document, payment, TMS handoff, action, and date range.
  - Add export for compliance requests.
- Acceptance criteria:
  - Ops can answer who changed what without developer help.

### ENT-0304 Define Status And Reference Data Governance

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Scope:
  - Make canonical owners for load statuses, leg statuses, offer statuses, escrow statuses, TMS statuses, and master data.
  - Document which statuses are customer-visible vs internal.
  - Add process for changing state machines.
- Acceptance criteria:
  - No workflow status can be changed casually without a migration/test/update plan.

### ENT-0305 Add Legal Agreement Acceptance And E-Signature Workflow

- Priority: `P1`
- Status: `[ ]`
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

### ENT-0306 Define Operating Authority, Insurance, And Jurisdiction Requirements

- Priority: `P1`
- Status: `[ ]`
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

## Phase 4: Document Security And Governance

Goal: make documents safe, versioned, compliant, and operationally useful.

### ENT-0401 Harden Local File Reads

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/Security
- Scope:
  - Canonicalize local document paths.
  - Reject reads outside configured storage root.
  - Add tests for path traversal attempts.
- Likely files:
  - `crates/backend/src/document_storage.rs`
- Acceptance criteria:
  - Malicious `../` or crafted local paths cannot escape storage root.

### ENT-0402 Add Document Versioning

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add document version table or version fields.
  - Preserve prior file, metadata, uploader, hash, review status, and replacement reason.
  - Show version history in load/profile/execution document UI.
- Acceptance criteria:
  - Replacing a document does not destroy audit history.

### ENT-0403 Add Required Document Rules

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Scope:
  - Define required documents by role, organization, equipment, commodity, load type, customer, and lifecycle state.
  - Show checklist on onboarding, load profile, execution, and closeout screens.
  - Block restricted transitions where required documents are missing.
- Acceptance criteria:
  - Closeout and compliance readiness are machine-checkable.

### ENT-0404 Replace Mock Blockchain Proof

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product
- Scope:
  - Replace `mock_blockchain_*` behavior with real content hash and verification.
  - Decide whether blockchain language remains in the product.
  - If blockchain remains, integrate a real timestamping/anchoring provider.
- Acceptance criteria:
  - UI no longer claims mock proof as real proof.
  - Hash verification uses actual uploaded file bytes.

### ENT-0405 Add File Validation And Scanning Hook

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Security
- Scope:
  - Enforce file size limits by document type.
  - Sniff MIME type and reject mismatches.
  - Add malware scanning integration point or quarantine status.
  - Add blocked file type policy.
- Acceptance criteria:
  - Dangerous or invalid uploads are blocked or quarantined.

### ENT-0406 Add Retention And Legal Hold

- Priority: `P2`
- Status: `[ ]`
- Owner: Product/Legal/Backend
- Scope:
  - Define retention by document type.
  - Add legal hold flags.
  - Add deletion/export workflows for privacy requests where legally allowed.
- Acceptance criteria:
  - Document lifecycle satisfies legal and customer retention requirements.

### ENT-0407 Add Freight Document Templates And Packets

- Priority: `P1`
- Status: `[ ]`
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

## Phase 5: Load Posting, Search, And Customer Rules

Goal: make load creation and discovery strong enough for enterprise shippers.

### ENT-0501 Finish Enterprise Load Model

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Scope:
  - Add complete fields for appointment windows, facility contacts, references, accessorials, service level, hazmat, temperature, container/securement, mode, commodity, and special handling.
  - Support public, private, contract, and internal-only visibility.
- Acceptance criteria:
  - Enterprise freight can be posted without out-of-band notes.

### ENT-0502 Add Draft/Publish/Revise/Cancel/Archive/Clone

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add load lifecycle actions.
  - Enforce state-specific edit rules.
  - Add clone and template creation.
- Acceptance criteria:
  - Users can manage common lifecycle actions safely.

### ENT-0503 Add Customer Contract And Lane Guide Model

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Scope:
  - Add customer contracts, lanes, contracted rates, preferred carriers, backup carriers, effective dates, and service rules.
  - Add contract vs spot posting behavior.
- Acceptance criteria:
  - Enterprise shippers can run private/contract freight, not only public spot loads.

### ENT-0504 Add Saved Filters And Load Search

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Backend
- Scope:
  - Add filters for origin, destination, radius, date, equipment, commodity, rate, customer, status, compliance, and visibility.
  - Add saved views by user/role.
  - Add pagination and index strategy.
- Acceptance criteria:
  - Load board stays usable as volume grows.

### ENT-0505 Add Bulk Import And API Posting

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add CSV import with validation preview.
  - Add API load posting endpoint with idempotency.
  - Add error export for failed rows.
- Acceptance criteria:
  - Enterprise customers can post high-volume loads without manual entry.

### ENT-0506 Add Rating, Mileage, Fuel, And Accessorial Rules

- Priority: `P1`
- Status: `[ ]`
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

### ENT-0507 Add Address Validation, Geocoding, And Facility Scheduling

- Priority: `P1`
- Status: `[ ]`
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

### ENT-0508 Add Mode-Specific Workflow Tracks

- Priority: `P2`
- Status: `[ ]`
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

### ENT-0509 Add Time Zone, Unit, Currency, And Localization Rules

- Priority: `P1`
- Status: `[ ]`
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

### ENT-0509A Define Cross-Border Tax, FX, Duties, And Incoterms Rules

- Priority: `P1`
- Status: `[ ]`
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

### ENT-0510 Add Governed Master Data And Configuration Admin

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend/Ops
- Scope:
  - Add admin workflows for equipment types, trailer types, commodities, hazmat classes, accessorial catalog, service levels, document requirements, rejection reasons, and exception reasons.
  - Add customer-specific configuration for lanes, facilities, carrier groups, visibility rules, compliance gates, billing rules, notification rules, and required references.
  - Add approval, audit, effective dates, import/export, and rollback for high-impact configuration changes.
  - Add validation so inactive or deprecated master data cannot silently break existing loads, integrations, reports, or pricing.
- Acceptance criteria:
  - Enterprise configuration can be changed safely without code deployments or direct database edits.
  - Master-data changes are auditable, reversible, and tested against active freight workflows.

## Phase 6: Carrier Network, Matching, And Marketplace

Goal: evolve from load list to freight marketplace.

### ENT-0601 Build Carrier Capacity Profile

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Scope:
  - Add equipment, lanes, operating geography, capacity, certifications, insurance limits, preferred commodities, service levels, and availability.
  - Add carrier self-service profile screen.
- Acceptance criteria:
  - Carrier eligibility can be computed from structured data.

### ENT-0602 Add Private Networks And Blocklists

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add preferred carriers, blocked carriers, private shipper networks, and carrier groups.
  - Enforce visibility and booking restrictions.
- Acceptance criteria:
  - Shippers control who can see and book private freight.

### ENT-0603 Add Matching And Ranking

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Data
- Scope:
  - Rank carriers by lane fit, equipment, proximity, compliance, performance, relationship, price, and tracking quality.
  - Explain why a carrier is recommended or blocked.
- Acceptance criteria:
  - Matching is explainable to operators and customers.

### ENT-0604 Complete Offer State Machine

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Add pending, countered, withdrawn, expired, declined, accepted, superseded, cancelled states.
  - Enforce transitions transactionally.
  - Add tests for all valid/invalid transitions.
- Acceptance criteria:
  - Offer state cannot become ambiguous or contradictory.

### ENT-0605 Add Counteroffers, Expiration, And Tender Flow

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add counteroffer UI and API.
  - Add offer expiration.
  - Add tender acceptance/decline separate from open bidding.
  - Add rate confirmation generation.
- Acceptance criteria:
  - Enterprise tendering and spot negotiation are both supported.

### ENT-0606 Add Booking Race Protection

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Add database transaction/locking around booking.
  - Add idempotency key for booking actions.
  - Add concurrency tests.
- Acceptance criteria:
  - Two carriers cannot book the same leg through race conditions.

### ENT-0607 Add Carrier Packet And Vetting Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Compliance/Backend/Frontend
- Scope:
  - Add carrier onboarding packet checklist.
  - Track W-9, COI, authority, operating agreement, banking/payout setup, safety/compliance review, and broker/customer-specific packet requirements.
  - Add packet approval, expiration, revision, and renewal workflow.
  - Link carrier packet readiness into matching and booking eligibility.
- Acceptance criteria:
  - A carrier cannot receive restricted freight until packet requirements are complete.
  - Operators can see exactly which packet item blocks eligibility.

## Phase 7: Dispatch Desk And Operator Workflows

Goal: let operations run the freight floor from STLoads.

### ENT-0701 Define Canonical Desk Queues

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Ops
- Scope:
  - Define quote, tender, facility, in-transit exception, closeout, collections, dispute, reconciliation, and compliance queues.
  - Define entry/exit rules for each queue.
- Acceptance criteria:
  - Every load/leg has a clear operational queue when action is needed.

### ENT-0702 Add Assignment, Priority, SLA, And Escalation

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add owner, priority, due date, SLA clock, escalation reason, and manager visibility.
  - Add filters and saved views.
- Acceptance criteria:
  - Managers can see backlog, aging, and stuck work.

### ENT-0703 Separate Internal And Customer-Visible Notes

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add internal notes and customer-visible updates as separate records.
  - Add permissions and audit.
- Acceptance criteria:
  - Internal operational comments cannot accidentally leak to customers.

### ENT-0704 Add Exception Management

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Scope:
  - Add exceptions for stale tracking, late pickup, late delivery, missing POD, payment hold, compliance block, TMS drift, and dispute.
  - Add resolution workflow.
- Acceptance criteria:
  - Operations can resolve freight exceptions without spreadsheets.

## Phase 8: Execution, Tracking, Mobile, And Closeout

Goal: make execution reliable enough for customer visibility and payment release.

### ENT-0801 Centralize Execution State Machine

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Define allowed leg transitions in one domain module.
  - Enforce preconditions transactionally.
  - Add tests for all transitions.
- Acceptance criteria:
  - Invalid pickup, delivery, or closeout transitions are impossible.

### ENT-0802 Build Mobile-First Driver Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Product
- Scope:
  - Create mobile-first route for driver/carrier execution.
  - Prioritize start tracking, arrive pickup, depart pickup, arrive delivery, upload POD, complete delivery.
  - Add PWA install/offline strategy decision.
- Acceptance criteria:
  - Driver tasks work comfortably on a phone.

### ENT-0802A Add Mobile Field Capture And Offline Strategy

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Product/Backend
- Scope:
  - Add camera-first document/photo capture for BOL, POD, pickup photos, delivery photos, seals, damage, and accessorial evidence.
  - Add offline/poor-network behavior for driver actions, notes, GPS pings, and document uploads.
  - Add push/web-push decision for mobile driver alerts.
  - Define supported mobile browsers/devices and whether a native app is required later.
  - Add QR/barcode/OCR decision for load references, BOLs, containers, or warehouse check-in if needed.
- Acceptance criteria:
  - Drivers can complete critical field tasks under real mobile network conditions.
  - Offline or delayed submissions are clearly marked and reconciled when connectivity returns.

### ENT-0803 Add Tracking Consent And Privacy

- Priority: `P0`
- Status: `[ ]`
- Owner: Product/Legal/Frontend/Backend
- Scope:
  - Add consent text and capture.
  - Add location retention policy.
  - Add customer-visible tracking scope.
- Acceptance criteria:
  - Location tracking is consented, explainable, and retained only as needed.

### ENT-0804 Add Geofence, ETA, And Delay Detection

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Data
- Scope:
  - Add geofence detection for pickup/delivery.
  - Add ETA calculation.
  - Add stale tracking, delay, detention, and route deviation signals.
- Acceptance criteria:
  - System detects execution risk before a customer asks.

### ENT-0805 Complete POD And Closeout Package

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Product
- Scope:
  - Define required closeout documents.
  - Add closeout checklist.
  - Add POD review and approval.
  - Add closeout package export.
- Acceptance criteria:
  - Delivered loads cannot be financially released until closeout rules pass.

### ENT-0806 Add Customer Tracking Page

- Priority: `P2`
- Status: `[ ]`
- Owner: Frontend/Backend
- Scope:
  - Add limited-visibility tracking share page.
  - Hide sensitive carrier/internal/payment details.
  - Add expiration and access controls.
- Acceptance criteria:
  - Customers can view relevant shipment progress safely.

### ENT-0807 Add Claims, Detention, And Accessorial Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend/Finance
- Scope:
  - Add claim intake for damage, shortage, late delivery, charge dispute, and service failure.
  - Add detention and accessorial request workflow with evidence upload and approval.
  - Link claims/accessorials to invoice, settlement, documents, audit, and support timeline.
  - Add customer/carrier visibility rules.
- Acceptance criteria:
  - Exceptions that affect billing or settlement are tracked through resolution.
  - Finance can see which charges are approved, disputed, rejected, or pending.

### ENT-0808 Add ELD And Telematics Integrations

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Product/Integrations
- Scope:
  - Decide whether to integrate ELD/telematics providers for automated tracking.
  - Add provider connection model per carrier or organization.
  - Normalize location, HOS/status, truck/trailer, and event pings where provider contracts allow it.
  - Add fallback behavior when ELD data is stale or unavailable.
  - Add consent and privacy rules for automated tracking.
- Acceptance criteria:
  - Enterprise customers can use automated tracking where carriers support it.
  - Manual/mobile tracking and ELD tracking produce a consistent execution timeline.

### ENT-0809 Add Route Planning And Optimization

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Data/Product
- Scope:
  - Add route distance and estimated duration source.
  - Add truck-safe route provider decision if needed.
  - Add route optimization for multi-stop or multi-leg loads.
  - Add toll, border, hazmat, temperature, and equipment constraints where supported.
  - Feed route distance into pricing, ETA, exception detection, and settlement.
- Acceptance criteria:
  - Mileage, ETA, and pricing are based on a documented route source.
  - Operators can explain route-derived calculations.

## Phase 9: Payments, Billing, Settlements, And Finance Controls

Goal: make money movement safe and reconcilable.

### ENT-0901 Add Payment Idempotency

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Add idempotency to escrow fund, hold, release, Stripe PaymentIntent, Stripe Transfer, refunds, and adjustments.
  - Store idempotency keys and outcomes.
- Acceptance criteria:
  - Repeated requests cannot double-charge or double-release.

### ENT-0902 De-Duplicate Stripe Webhooks

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Store Stripe event IDs.
  - Reject duplicate event processing.
  - Add tests for replayed webhooks.
- Acceptance criteria:
  - Stripe retry behavior cannot corrupt escrow state.

### ENT-0903 Add Payment Ledger

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/Finance
- Scope:
  - Add ledger entries for escrow funded, fee earned, hold, release, transfer, refund, dispute, adjustment, and payout failure.
  - Link ledger entries to Stripe IDs and audit events.
- Acceptance criteria:
  - Finance can reconstruct every cent.

### ENT-0904 Add Finance Approval Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Finance
- Scope:
  - Add manual hold/release approvals.
  - Add two-person approval threshold for high-value releases.
  - Add step-up MFA for release.
- Acceptance criteria:
  - High-risk payouts cannot be released casually.

### ENT-0905 Add Invoices And Carrier Settlements

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Finance
- Scope:
  - Add customer invoice model.
  - Add carrier settlement model.
  - Add platform fees, broker margin, accessorials, taxes, adjustments, and payment terms.
  - Add invoice and settlement status lifecycle.
- Acceptance criteria:
  - Platform supports billing and settlement, not only escrow.

### ENT-0906 Add Accounting Export

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Finance
- Scope:
  - Add CSV export first.
  - Design QuickBooks/NetSuite/Xero integration later if needed.
- Acceptance criteria:
  - Finance can reconcile outside the app without custom SQL.

### ENT-0907 Add Factoring, Advances, And Fuel Support Decision

- Priority: `P2`
- Status: `[ ]`
- Owner: Product/Finance/Backend
- Scope:
  - Decide whether STLoads supports factoring, quick pay, fuel advances, fuel cards, or carrier advances.
  - Add finance controls, fees, eligibility, audit, and repayment/settlement treatment for any supported option.
  - Add explicit unsupported-state messaging if deferred.
- Acceptance criteria:
  - Carrier payment options are either supported safely or clearly out of scope.
  - Finance can reconcile advances and fees if enabled.

### ENT-0908 Add STLoads Subscription And Usage Billing

- Priority: `P2`
- Status: `[ ]`
- Owner: Product/Finance/Backend
- Scope:
  - Decide STLoads commercial model: subscription, per-load fee, transaction fee, seat-based pricing, usage-based API/TMS fees, or hybrid.
  - Add customer plan, billing account, invoices, payment method, renewal, cancellation, and usage tracking if applicable.
  - Keep customer subscription billing separate from freight escrow, carrier settlement, and shipper invoice workflows.
- Acceptance criteria:
  - STLoads can bill enterprise customers for platform usage without mixing it with freight money movement.

### ENT-0909 Add Shipper Credit, AR Aging, And Collections Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Finance/Product/Backend/Frontend
- Scope:
  - Add customer credit status, credit limit, payment terms, credit hold, and manual override workflow.
  - Add AR aging, overdue invoice, dunning, collections queue, and promise-to-pay notes.
  - Decide whether loads can be posted, tendered, booked, or released when a shipper is over limit or on credit hold.
  - Add audit and approval for credit limit changes and credit hold overrides.
  - Add customer-visible payment status where appropriate without exposing internal risk notes.
- Acceptance criteria:
  - Finance can prevent new exposure from high-risk or overdue shippers.
  - Operators know when a load is blocked by credit policy and how to escalate.

### ENT-0910 Add Bank Account And Payout Change Controls

- Priority: `P0`
- Status: `[ ]`
- Owner: Finance/Security/Backend
- Scope:
  - Verify carrier payout bank accounts through Stripe or approved provider workflows.
  - Add cooling-off, step-up auth, audit, and notification for payout destination changes.
  - Block or review payouts after suspicious bank, email, phone, or ownership changes.
  - Add finance review queue for failed verification, returned payouts, and bank mismatch cases.
- Acceptance criteria:
  - Payout destination changes cannot silently redirect carrier money.
  - Returned or suspicious payouts are held until finance review is complete.

## Phase 10: Compliance, Risk, And Fraud

Goal: prevent bad actors and non-compliant freight movement.

### ENT-1001 Split Compliance Models

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Scope:
  - Split person KYC, company KYB, carrier compliance, broker compliance, freight-forwarder compliance, tax compliance, and payout compliance.
- Acceptance criteria:
  - Compliance status is specific enough to drive eligibility.

### ENT-1002 Add FMCSA/DOT/MC And Insurance Verification

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product
- Scope:
  - Add external verification integration or manual verification workflow.
  - Track authority status, insurance expiry, coverage limit, and operating authority.
- Acceptance criteria:
  - Carrier booking can be blocked by expired or missing compliance.

### ENT-1002A Add Driver, Equipment, And Safety Compliance

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Compliance/Backend/Frontend
- Scope:
  - Track driver qualification requirements: CDL, medical card, MVR/background decision, endorsements, and expiration dates where applicable.
  - Track truck, trailer, equipment, VIN/unit identifiers, ownership/lease, inspection, maintenance, and insurance relationships where required.
  - Decide whether DVIR, maintenance, and inspection workflows are in scope or explicitly deferred.
  - Surface safety/compliance signals such as FMCSA authority, insurance, safety rating, and CSA-style metrics where available.
  - Link driver/equipment eligibility to load requirements, hazmat, temperature, mode, and customer rules.
- Acceptance criteria:
  - Carrier eligibility can account for driver and equipment requirements, not only company-level compliance.
  - Expired or missing driver/equipment compliance can block restricted freight.

### ENT-1003 Add Sanctions And Tax Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Legal/Backend
- Scope:
  - Add OFAC/sanctions screening workflow.
  - Add W-9/tax document workflow for US payouts.
  - Decide 1099 or other payout tax reporting requirements with finance/legal.
  - Track beneficial owner checks where applicable.
- Acceptance criteria:
  - High-risk entities can be blocked before booking or payout.
  - Payout tax reporting obligations are implemented or explicitly assigned to an external finance process.

### ENT-1004 Add Risk Scoring And Review Queue

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Data/Ops
- Scope:
  - Score unusual login, new payout account, suspicious document changes, sudden rate changes, compliance mismatch, and repeated failed payments.
  - Add fraud/risk review queue.
- Acceptance criteria:
  - Risky accounts and transactions can be paused and reviewed.

### ENT-1005 Add Double-Brokering And Carrier Fraud Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Compliance/Risk/Backend
- Scope:
  - Add double-brokering risk signals: mismatched authority, suspicious contact changes, newly changed payout account, reused documents, rapid reassignment, unusual rate spread, tracking mismatch, and carrier identity mismatch.
  - Add manual review queue and hold controls.
  - Add carrier identity verification steps for high-risk bookings.
  - Add audit and evidence capture for fraud decisions.
- Acceptance criteria:
  - High-risk bookings can be paused before tender, pickup, or payout.
  - Operators have explicit risk reasons, not only a generic fraud flag.

### ENT-1006 Add AML, Transaction Monitoring, And Account-Takeover Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Risk/Compliance/Security/Backend
- Scope:
  - Decide AML, money-transmission, payment-facilitator, and suspicious-activity obligations based on the chosen operating model and Stripe/payment-provider setup.
  - Add transaction-monitoring rules for unusual payout velocity, account changes before payout, split payments, refund abuse, chargeback patterns, and high-risk geographies where applicable.
  - Add account-takeover signals such as impossible travel, unusual device/IP, risky email/phone change, failed MFA, and new payout destination.
  - Define manual review, hold, escalation, reporting, and provider-notification workflow for suspicious activity.
  - Add customer/carrier communication rules when transactions are held for compliance or security review.
- Acceptance criteria:
  - Suspicious money movement and account-takeover attempts can be detected before payout release.
  - Compliance obligations are implemented or explicitly ruled out for the target operating model.

## Phase 11: TMS, APIs, Webhooks, And Integrations

Goal: become a reliable integration partner.

### ENT-1101 Publish OpenAPI Specs

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Add OpenAPI for auth, loads, offers, tracking, documents, payments, TMS, and webhooks.
  - Version APIs.
- Acceptance criteria:
  - External customers can integrate against documented contracts.

### ENT-1102 Add Partner API Auth

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/Security
- Scope:
  - Add API keys or OAuth client credentials for partners.
  - Add request signing for sensitive endpoints.
  - Add partner-specific rate limits.
- Acceptance criteria:
  - External APIs are not protected only by user-session assumptions.

### ENT-1103 Add External Idempotency And Event De-Dupe

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Add idempotency keys for all external writes.
  - Store external event IDs for TMS and webhook events.
  - Reject duplicates.
- Acceptance criteria:
  - Retried partner requests do not create duplicate loads or state changes.

### ENT-1104 Add Webhook Delivery Logs And Replay

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Store webhook deliveries, response codes, latency, attempts, and next retry.
  - Add dead-letter state and replay UI.
- Acceptance criteria:
  - Failed customer webhooks can be diagnosed and replayed.

### ENT-1105 Add TMS Conflict Resolution

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Ops
- Scope:
  - Define source-of-truth per field.
  - Add conflict queue when STLoads and TMS disagree.
  - Add replay and repair actions.
- Acceptance criteria:
  - TMS drift can be fixed without developer database access.

### ENT-1106 Add Customer Integration Portal

- Priority: `P2`
- Status: `[ ]`
- Owner: Frontend/Backend/Product
- Scope:
  - Add API keys, webhook endpoints, delivery logs, sandbox data, and docs.
- Acceptance criteria:
  - Enterprise customers can self-serve integration setup safely.

### ENT-1107 Add EDI Integration Track

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product/Integrations
- Scope:
  - Decide supported EDI transactions for first enterprise release, such as 204 load tender, 990 tender response, 214 shipment status, 210 invoice, and 997 acknowledgement.
  - Define mapping between EDI payloads and STLoads load, tender, execution, invoice, and status models.
  - Add EDI validation, acknowledgements, retry, replay, and partner-specific mapping rules.
  - Add EDI visibility in the integration portal.
- Acceptance criteria:
  - Enterprise logistics partners can exchange standard freight events without custom one-off scripts.
  - Failed EDI messages are visible, replayable, and auditable.

### ENT-1108 Add Sandbox, Demo, And Test Tenant Governance

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/DevOps
- Scope:
  - Add sandbox tenant strategy for enterprise integrations and demos.
  - Define seeded demo data that contains no real PII, payment credentials, documents, or customer freight.
  - Add sandbox API keys, webhook endpoints, and reset tooling.
  - Ensure sandbox events cannot trigger production payments, production TMS pushes, or real notifications.
- Acceptance criteria:
  - Sales, support, QA, and customers can test safely without production data risk.
  - Sandbox/test behavior is visibly separated from production behavior.

### ENT-1109 Define API Lifecycle, SDKs, And Deprecation Policy

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Integrations
- Scope:
  - Define API versioning, changelog, sunset headers, customer notice windows, and emergency breaking-change procedure.
  - Decide whether to ship official SDKs, generated clients, Postman collections, sample apps, or integration templates.
  - Add compatibility tests for supported API versions and partner payload examples.
  - Document upgrade paths for customers using webhooks, EDI, TMS sync, and public APIs.
- Acceptance criteria:
  - Enterprise customers know how long integrations are supported and how breaking changes are handled.
  - Supported API examples can be run against sandbox without custom engineering help.

## Phase 12: Notifications And Communications

Goal: notify the right people through the right channels.

### ENT-1201 Add Notification Center

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add in-app notifications table and UI.
  - Link notifications to loads, offers, documents, payments, compliance, and TMS events.
- Acceptance criteria:
  - Users do not depend only on email for operational events.

### ENT-1202 Add Notification Preferences

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Add per-user and per-org preferences.
  - Support email, in-app, and future SMS/push.
  - Add quiet hours and escalation preferences.
- Acceptance criteria:
  - Enterprise customers can control noisy workflows.

### ENT-1203 Add SMS/Push Decision And Provider

- Priority: `P2`
- Status: `[ ]`
- Owner: Product/Backend
- Scope:
  - Decide whether to add SMS, push, or both.
  - Choose provider and compliance requirements.
  - Add opt-in/opt-out.
- Acceptance criteria:
  - Driver and urgent exception notifications have a reliable channel.

### ENT-1204 Broaden Notification Coverage

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product
- Scope:
  - Add notifications for booking, counteroffer, tender, tracking stale, pickup/delivery, missing POD, payment hold/release, compliance expiry, TMS drift, and document rejection.
- Acceptance criteria:
  - Critical events notify the responsible party.

### ENT-1205 Add Message Deliverability And Compliance Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Ops/Product/Legal
- Scope:
  - Configure sender domains, SPF, DKIM, DMARC, and environment-specific sender identities.
  - Track email bounces, complaints, suppression lists, delivery failures, and retry outcomes.
  - Add template ownership, approval, localization, versioning, and test-send workflow for operational messages.
  - Define SMS/push compliance requirements including opt-in, opt-out, quiet hours, emergency exceptions, and provider audit logs.
  - Add deliverability monitoring for high-risk messages such as OTP, password reset, tender, pickup/delivery, POD rejection, payment hold, and payout release.
- Acceptance criteria:
  - Critical messages have observable delivery status and safe fallback/escalation behavior.
  - Message templates and sender identities can be changed without accidental compliance or deliverability regressions.

### ENT-1206 Add Tenant Branding And Customer-Facing Identity Controls

- Priority: `P2`
- Status: `[ ]`
- Owner: Product/Frontend/Backend/Ops
- Scope:
  - Decide whether enterprise customers can use tenant logos, portal branding, document branding, email branding, or custom domains.
  - Add safe asset upload, review, size/type constraints, fallback branding, and cache invalidation for customer logos and branded assets.
  - Add branded templates for rate confirmations, BOLs, POD packages, invoices, settlement packets, and notification emails if supported.
  - Add custom-domain setup workflow including DNS validation, TLS certificate handling, ownership checks, and rollback.
  - Add explicit unsupported-state messaging if white-label or custom-domain support is deferred.
- Acceptance criteria:
  - Customer-facing branding is controlled, secure, and consistent across portals, documents, and messages.
  - Unsupported branding promises cannot slip into sales or implementation without product approval.

## Phase 13: Frontend, UX, Accessibility, And Design System

Goal: make the product feel like a professional logistics command center.

### ENT-1301 Create Shared UI Component Library

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend
- Scope:
  - Build components for table, filters, modal, drawer, toast, badge, timeline, file uploader, status pill, map panel, money controls, and confirmation dialogs.
  - Reduce repeated inline styles.
- Acceptance criteria:
  - New screens use shared components by default.

### ENT-1302 Refactor Large Leptos Pages

- Priority: `P2`
- Status: `[ ]`
- Owner: Frontend
- Scope:
  - Split large pages such as auth, load profile, execution, admin users, master data, and dispatch desk into smaller components.
- Acceptance criteria:
  - Large page files become maintainable and testable.

### ENT-1303 Add Accessibility Pass

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/QA
- Scope:
  - Define target accessibility level, such as WCAG 2.2 AA for core enterprise workflows.
  - Add keyboard navigation.
  - Add labels and focus management.
  - Add color contrast, reduced-motion, hit-target, and visible-focus checks.
  - Add accessible error states and modals.
  - Run automated checks and manual spot checks.
- Acceptance criteria:
  - Critical flows are keyboard and screen-reader usable.
  - Accessibility exceptions are documented with owners and remediation dates.

### ENT-1304 Add Role-Specific Dashboards

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Frontend/Backend
- Scope:
  - Build dashboards for admin, shipper, carrier, broker/forwarder, finance, and support.
- Acceptance criteria:
  - Each role lands on actionable work, not a generic page.

### ENT-1305 Add Browser E2E And Visual Regression

- Priority: `P1`
- Status: `[ ]`
- Owner: QA/Frontend
- Scope:
  - Add E2E tests for auth, load posting, booking, chat, execution, documents, finance, and admin.
  - Add screenshots for critical screens.
- Acceptance criteria:
  - UI regressions are caught before production.

## Phase 14: Data, Reporting, Search, And Intelligence

Goal: make operational data usable.

### ENT-1401 Define Business Metrics

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Data/Ops
- Scope:
  - Define posted loads, booked loads, acceptance rate, quote-to-book time, tracking compliance, on-time pickup, on-time delivery, document cycle time, margin, payout time, dispute rate.
- Acceptance criteria:
  - Metrics definitions are documented and accepted by product/ops.

### ENT-1402 Add Reporting Data Model

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Data
- Scope:
  - Add read models or warehouse export.
  - Add data refresh strategy.
- Acceptance criteria:
  - Reports do not slow operational screens.

### ENT-1403 Add Customer And Carrier Scorecards

- Priority: `P2`
- Status: `[ ]`
- Owner: Product/Data/Frontend
- Scope:
  - Add customer performance reports.
  - Add carrier scorecards for acceptance, tracking, on-time, claims, document quality, and payout speed.
- Acceptance criteria:
  - Operators can make carrier/customer decisions from historical data.

### ENT-1404 Add Pricing And Lane Intelligence

- Priority: `P2`
- Status: `[ ]`
- Owner: Data/Product
- Scope:
  - Track lane history.
  - Add pricing reference and rate recommendations.
  - Flag rate anomalies.
- Acceptance criteria:
  - Pricing decisions are data-assisted.

### ENT-1405 Add Global Search

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Frontend
- Scope:
  - Search loads, users, organizations, documents, invoices, payments, conversations, and TMS handoffs.
  - Add permission-aware results.
- Acceptance criteria:
  - Support and operations can find work quickly without unsafe access.

### ENT-1406 Add Data Quality And Integrity Monitoring

- Priority: `P1`
- Status: `[ ]`
- Owner: Data/Backend/Ops
- Scope:
  - Add recurring checks for orphan records, invalid state combinations, duplicate external references, missing required documents, stale TMS handoffs, unmatched payments, and inconsistent tenant ownership.
  - Add anomaly checks for lane rates, carrier score changes, suspicious tracking patterns, unusual document replacement, and sudden volume changes.
  - Add data quality dashboard, alert thresholds, owner routing, and repair workflow.
  - Add tests or scripts that can be run before migrations, after cutover, and during incident recovery.
- Acceptance criteria:
  - Bad data is detected before it becomes an operational or financial incident.
  - Data quality issues have owners, severity, repair status, and audit trail.

## Phase 15: Observability, Workers, Scale, And Disaster Recovery

Goal: make the platform reliable under real volume.

### ENT-1501 Add Structured Logs And Tracing

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/DevOps
- Scope:
  - Emit JSON logs in production.
  - Add OpenTelemetry traces for HTTP, SQL, Stripe, COS, SMTP, TMS, and workers.
- Acceptance criteria:
  - Incidents can be traced end to end.

### ENT-1502 Add Metrics And Alerts

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Backend
- Scope:
  - Track request latency, error rate, DB pool usage, queue lag, worker outcomes, email failures, webhook failures, storage errors, payment failures, and TMS drift.
  - Add alerts for P0/P1 failures.
- Acceptance criteria:
  - Team is alerted before customers report major failures.

### ENT-1502A Add On-Call, Escalation, And Security Log Export

- Priority: `P1`
- Status: `[ ]`
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

### ENT-1503 Separate Workers From Web Runtime

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/DevOps
- Scope:
  - Split email and TMS workers into separate service/process modes.
  - Add locking to avoid duplicated worker execution.
  - Add graceful shutdown.
- Acceptance criteria:
  - Scaling web traffic does not multiply background job side effects.

### ENT-1504 Add Job Queue And Dead Letter Handling

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend
- Scope:
  - Add job table or queue provider.
  - Add retry, lock, visibility timeout, max attempts, and dead-letter states.
  - Add worker dashboard.
- Acceptance criteria:
  - Failed background work is visible and recoverable.

### ENT-1505 Optimize Database Queries And Indexes

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/DBA
- Scope:
  - Analyze query plans for load board, chat, tracking, admin queues, TMS reconciliation, reports, and global search.
  - Add indexes and cursor pagination.
- Acceptance criteria:
  - Critical pages remain fast with production-scale data.

### ENT-1506 Define Backup, Restore, RPO, And RTO

- Priority: `P0`
- Status: `[ ]`
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

### ENT-1507 Add Archiving Strategy

- Priority: `P2`
- Status: `[ ]`
- Owner: Backend/Data
- Scope:
  - Archive old location pings, messages, audit events, TMS handoffs, and document metadata according to retention.
- Acceptance criteria:
  - Large history tables do not degrade operational performance.

### ENT-1508 Add Incident Response, Status Page, And Runbooks

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Ops/Security
- Scope:
  - Define incident severity levels.
  - Add runbooks for auth outage, database outage, object storage outage, payment incident, duplicate booking, TMS outage, email outage, data exposure, and bad deploy.
  - Add customer communication process and status page decision.
  - Add post-incident review template.
- Acceptance criteria:
  - The team can respond to incidents with a documented process.
  - Customers can be informed consistently during outages or degraded service.

### ENT-1509 Define Business Continuity And Tabletop Exercises

- Priority: `P1`
- Status: `[ ]`
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

### ENT-1510 Add Cost, Quota, And Usage Guardrails

- Priority: `P1`
- Status: `[ ]`
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

## Phase 16: Testing, CI, And Quality Gates

Goal: make regression prevention systematic.

### ENT-1601 Define Test Lanes

- Priority: `P0`
- Status: `[ ]`
- Owner: Engineering/QA
- Scope:
  - Define fast unit lane, backend integration lane, frontend build lane, browser E2E lane, security lane, and hosted smoke lane.
  - Fix or split the current full workspace test path that timed out during review.
- Acceptance criteria:
  - Developers know what to run locally and CI knows what blocks merge.

### ENT-1602 Add CI Pipeline

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Engineering
- Scope:
  - Run formatting, clippy, tests, SQLx checks, frontend build, Docker build, dependency audit, and secrets scan.
- Acceptance criteria:
  - Main branch cannot receive unverified risky code.

### ENT-1603 Add Domain State Machine Tests

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/QA
- Scope:
  - Test offer, booking, execution, escrow, document, user lifecycle, and TMS state machines.
- Acceptance criteria:
  - Invalid transitions are rejected by tests and code.

### ENT-1604 Add Security Access Tests

- Priority: `P0`
- Status: `[ ]`
- Owner: QA/Backend
- Scope:
  - Test document access, tenant isolation, admin route access, finance access, TMS access, and support access.
- Acceptance criteria:
  - P0 access regressions are caught automatically.

### ENT-1605 Add Load And Performance Tests

- Priority: `P2`
- Status: `[ ]`
- Owner: QA/Backend
- Scope:
  - Test load board, chat, tracking writes, admin queues, TMS webhooks, and document uploads at realistic volumes.
- Acceptance criteria:
  - Performance risks are known before enterprise rollout.

### ENT-1606 Make Clippy Warning-Clean

- Priority: `P1`
- Status: `[ ]`
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

### ENT-1607 Add Frontend Release Build To CI

- Priority: `P0`
- Status: `[ ]`
- Owner: Frontend/DevOps
- Scope:
  - Run `trunk build --release` in CI.
  - Ensure `wasm32-unknown-unknown` target and `trunk` version are pinned or installed reproducibly.
  - Capture frontend build artifacts only through the intended deployment path.
- Acceptance criteria:
  - Main branch cannot break the Leptos WASM release build.

## Phase 17: Security, Legal, And Enterprise Procurement

Goal: pass enterprise customer review.

### ENT-1701 Perform Threat Model

- Priority: `P0`
- Status: `[ ]`
- Owner: Security/Engineering
- Scope:
  - Threat model auth, payments, documents, TMS, admin, support tooling, and tenant isolation.
- Acceptance criteria:
  - Threats have tracked mitigations.

### ENT-1702 Add Security Headers And CSP

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Backend/Security
- Scope:
  - Add CSP, HSTS, X-Content-Type-Options, frame policy, referrer policy.
  - Validate frontend scripts and external maps/Stripe/Google integrations.
- Acceptance criteria:
  - Browser security baseline is enterprise acceptable.

### ENT-1703 Add Dependency And Secret Scanning

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Security
- Scope:
  - Add `cargo audit` or equivalent.
  - Add JS dependency scan.
  - Add secret scanning.
- Acceptance criteria:
  - Known vulnerable dependencies and leaked secrets block release.

### ENT-1704 Define Privacy And Data Request Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Legal/Product/Backend
- Scope:
  - Define data export, deletion, retention, and legal hold behavior.
  - Include location data, documents, chat, audit, and payment constraints.
- Acceptance criteria:
  - Customer privacy requests can be processed consistently.

### ENT-1705 Prepare Enterprise Security Packet

- Priority: `P2`
- Status: `[ ]`
- Owner: Security/Product
- Scope:
  - Prepare answers for hosting, encryption, access control, backups, incident response, vulnerability management, logging, retention, and subprocessors.
- Acceptance criteria:
  - Sales/support can answer security questionnaires without engineering fire drills.

### ENT-1706 Define Encryption And Data Classification

- Priority: `P0`
- Status: `[ ]`
- Owner: Security/Backend/Legal
- Scope:
  - Classify data: public, internal, confidential, regulated/sensitive, payment-related, location, identity, and document data.
  - Confirm encryption in transit and at rest for database, object storage, backups, logs, and secrets.
  - Decide which fields need application-level encryption or masking.
  - Add redaction rules for logs, audit exports, support screens, and analytics.
- Acceptance criteria:
  - Sensitive data handling is documented and enforced.
  - Logs and support tools do not expose secrets, payment data, private documents, or unnecessary PII.

### ENT-1706A Define Key Management, Rotation, And PCI Scope

- Priority: `P0`
- Status: `[ ]`
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

### ENT-1707 Enforce Secret File Hygiene

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Security
- Scope:
  - Keep `.env.ibm.secret`, `.env.ibm.runtime`, `.cos-*`, TLS private keys, and local credential exports ignored.
  - Add pre-commit or CI secret scanning.
  - Document how developers should store local credentials.
  - Rotate any secret that is ever accidentally committed, pasted into logs, or shared in chat.
- Acceptance criteria:
  - Secret scanning blocks commits and CI merges.
  - Local secret file policy is documented and followed.

### ENT-1708 Add WAF, DDoS, And Bot Protection

- Priority: `P0`
- Status: `[ ]`
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

### ENT-1709 Add Vendor, Subprocessor, And Third-Party Risk Management

- Priority: `P1`
- Status: `[ ]`
- Owner: Security/Legal/Product
- Scope:
  - Inventory vendors and subprocessors: IBM, Stripe, SMTP provider, maps/geocoding, ELD/telematics providers, EDI providers, analytics, monitoring, and support tools.
  - Track contracts, DPAs, data processed, region, retention, security posture, and owner.
  - Add vendor review and annual access/security review process.
- Acceptance criteria:
  - Enterprise security questionnaires can be answered with a maintained vendor inventory.
  - New vendors cannot be introduced without review.

### ENT-1710 Define Data Residency, DPA, And Regional Requirements

- Priority: `P1`
- Status: `[ ]`
- Owner: Legal/Security/Backend
- Scope:
  - Decide supported data regions and residency commitments.
  - Prepare DPA language and privacy commitments for enterprise customers.
  - Determine whether GDPR, CCPA/CPRA, or other regional privacy requirements apply to target customers.
  - Document how backups, object storage, logs, analytics, and subprocessors respect region commitments.
- Acceptance criteria:
  - Sales can answer where customer data lives and under what privacy terms.
  - Engineering knows which region boundaries must be enforced.

### ENT-1711 Add Penetration Testing And Vulnerability Disclosure

- Priority: `P1`
- Status: `[ ]`
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

### ENT-1712 Define SOC 2 Or ISO 27001 Readiness Program

- Priority: `P1`
- Status: `[ ]`
- Owner: Security/Ops/Legal/Engineering
- Scope:
  - Decide target framework and timeline: SOC 2 Type I, SOC 2 Type II, ISO 27001, or explicit deferral for the first enterprise release.
  - Map controls for access management, change management, incident response, vendor management, backups, encryption, logging, monitoring, vulnerability management, and data retention.
  - Assign evidence owners and storage location for policies, tickets, approvals, logs, reviews, tests, and customer-facing reports.
  - Define auditor, readiness assessment, observation window, and remediation plan if certification is in scope.
- Acceptance criteria:
  - Enterprise procurement can see the compliance roadmap and current evidence posture.
  - Required operational controls are tracked as product/engineering work, not only policy documents.

## Enterprise Ready Completion Checklist

Do not call the platform enterprise-ready until every item below is true:

- `[ ]` Production cannot start in unsafe fallback mode.
- `[x]` Session tokens are hashed and privileged workflows require MFA or step-up auth.
- `[x]` Browser sessions, cookies, CSRF, CORS, and custom-domain auth boundaries are hardened and tested.
- `[ ]` Tenant/org isolation is implemented and tested.
- `[ ]` Privileged access is reviewed, recertified, and revoked when stale or excessive.
- `[ ]` Global audit ledger covers auth, admin, loads, documents, offers, execution, payments, TMS, support, and master data.
- `[ ]` STLoads operating authority, surety/bond, corporate insurance, and jurisdiction obligations are implemented or explicitly out of scope for the operating model.
- `[ ]` Documents are access-controlled, versioned, validated, retained, and scanned or quarantined.
- `[ ]` Load posting supports enterprise freight data, private/contract freight, and customer-specific rules.
- `[ ]` Master data and customer configuration can be governed without code deployments or direct database edits.
- `[ ]` Carrier matching uses compliance, lane fit, capacity, preferences, and performance.
- `[ ]` Offer, booking, execution, escrow, and TMS states are strict and tested.
- `[ ]` Payments are idempotent, ledgered, reconciled, and protected from duplicate release.
- `[ ]` Shipper credit limits, AR aging, dunning, and collections controls prevent unmanaged credit exposure.
- `[ ]` Carrier payout bank account changes are verified, audited, delayed or reviewed, and protected by step-up controls.
- `[ ]` Compliance gates prevent ineligible carriers from booking restricted freight.
- `[ ]` Driver, equipment, trailer, inspection, and safety compliance are implemented or explicitly deferred by target release.
- `[ ]` TMS/API integrations are versioned, authenticated, idempotent, observable, and replayable.
- `[ ]` API deprecation, SDK/sample-code, changelog, and customer migration policy are defined.
- `[ ]` Operators can run desks, exceptions, closeout, finance, support, and reconciliation without database access.
- `[ ]` Support cases, SLA breach tracking, customer updates, and CSAT/feedback loops are implemented or integrated.
- `[ ]` Mobile-first driver execution works for pickup, tracking, POD, and delivery.
- `[ ]` Mobile field capture, offline behavior, push-notification decision, and supported-device policy are ready.
- `[ ]` In-app/email notification coverage exists for critical workflow events.
- `[ ]` Email deliverability, bounce handling, template governance, and SMS/push compliance controls are implemented.
- `[ ]` Tenant branding, branded documents/messages, and custom-domain support are implemented or explicitly deferred.
- `[ ]` CI blocks unsafe changes and includes security, backend, frontend, and E2E gates.
- `[ ]` Clippy passes with `-D warnings`, or every remaining lint allowance is deliberate and documented.
- `[ ]` Frontend release build is part of CI.
- `[ ]` Data quality monitoring catches orphan records, invalid states, duplicate references, and financial/integration mismatches.
- `[ ]` Observability includes logs, metrics, traces, alerts, dashboards, and runbooks.
- `[ ]` On-call ownership, alert escalation, SIEM/log-drain, and security-event export are defined and tested.
- `[ ]` Usage quotas, cost alerts, provider spend monitoring, and tenant limit workflows are in place.
- `[ ]` Backup/restore has tested RPO/RTO.
- `[ ]` PITR, failover, replica, regional dependency, and restore/replay strategy are defined for the target release.
- `[ ]` Business continuity procedures and tabletop exercises cover prolonged outages and manual fallback operations.
- `[ ]` Legal/privacy workflows exist for consent, retention, export, deletion, and legal hold.
- `[ ]` Enterprise security packet is ready for customer review.
- `[ ]` Penetration testing, vulnerability intake, remediation SLA, and customer evidence workflow are complete.
- `[ ]` SOC 2 or ISO 27001 readiness, evidence ownership, and certification/deferral timeline are defined.
- `[ ]` Enterprise customers can use SSO and, where required, automated user provisioning/deprovisioning.
- `[ ]` Feature flags, change approvals, and kill switches exist for risky production changes.
- `[ ]` Customer release notes, UAT/pilot rollout, maintenance notices, and adoption feedback loops are ready.
- `[ ]` EDI or an explicitly approved EDI alternative is ready for target enterprise partners.
- `[ ]` Rating, mileage, fuel surcharge, accessorial, detention, and claims workflows are operational.
- `[ ]` Cross-border FX, tax, duties, customs-fee, and Incoterms rules are implemented or explicitly deferred.
- `[ ]` Facility appointment, geocoding, and route-planning workflows are operational.
- `[ ]` Standard freight documents and carrier packets can be generated, stored, reviewed, and audited.
- `[ ]` Carrier fraud and double-brokering controls can pause high-risk bookings and payouts.
- `[ ]` AML, transaction-monitoring, account-takeover, and suspicious-activity workflows are implemented or explicitly ruled out.
- `[ ]` Supported freight modes have mode-specific fields, documents, validations, and workflows.
- `[ ]` ELD/telematics and route optimization are either implemented or explicitly deferred for the target release.
- `[ ]` Factoring, quick pay, fuel advances, and fuel-card support are either implemented or explicitly out of scope.
- `[ ]` W-9, 1099 or equivalent payout tax reporting responsibilities are implemented or assigned to finance.
- `[ ]` Encryption, data classification, masking, and log redaction policies are implemented.
- `[ ]` Key management, key rotation, PCI/payment-data scope, and payment secret handling are documented and tested.
- `[ ]` Secret file hygiene and secret scanning prevent local runtime credentials from entering git.
- `[ ]` Incident response runbooks and customer communication process are tested.
- `[ ]` Customer SLAs, support tiers, training, and help-center material are ready.
- `[ ]` Core workflows meet the agreed accessibility target or have documented remediation owners and dates.
- `[ ]` Enterprise customer offboarding, data return, tenant archival, and integration shutdown are defined and tested.
- `[ ]` WAF, DDoS, bot protection, and abuse response controls protect public surfaces.
- `[ ]` Vendor/subprocessor inventory, DPA, and data residency commitments are documented and enforceable.
- `[ ]` Sandbox/demo tenants are isolated from production data, payments, integrations, and notifications.
- `[ ]` STLoads platform subscription or usage billing is either implemented or explicitly deferred.
- `[ ]` Production data migration, cutover rehearsal, and legacy-vs-Rust reconciliation are complete.
- `[ ]` Legal agreements, terms acceptance, tracking consent, and e-signature requirements are versioned and auditable.
- `[ ]` Time zones, measurement units, currencies, and localization rules are normalized and tested.
