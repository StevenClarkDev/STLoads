# Enterprise Work Board

Last updated: 2026-05-25

This board is generated from `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md` and is the execution board for `P0` and `P1` enterprise-readiness tasks. Keep task IDs stable and update this board whenever the task list changes.

## Board Rules

- One card exists for every `P0` and `P1` task in the enterprise task list.
- Card titles preserve the task ID from the task list.
- Every card includes owner, priority, dependencies, acceptance criteria, verification notes, and issue link fields.
- Update this board and the task list in the same change set after completing or blocking work.

## Summary

- Total P0/P1 cards: 124
- P0 cards: 43
- P1 cards: 81

## Phase 0: Program Setup And Governance

### ENT-0001 Create Enterprise Work Board

- Priority: `P0`
- Status: `[x]`
- Owner: Product/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `None`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Every `P0` and `P1` task exists in the work board.
  - No task is accepted without a test or verification plan.
- Verification notes:
  - Created this repo-local execution board.
  - Verified 124 `P0`/`P1` source tasks and 124 work-board cards.
  - Cards preserve task IDs, owners, priorities, acceptance criteria, issue link placeholders, dependency placeholders, estimate placeholders, and verification notes.

### ENT-0002 Align Documentation Truth

- Priority: `P0`
- Status: `[x]`
- Owner: Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0001`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0001`, `ENT-0002`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0003`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0003`, `ENT-0004`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0005`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0006`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise support work is tracked as cases with SLA visibility, not only chat/email history.
  - Product and operations can see recurring customer pain from support data.
- Verification notes:
  - Created `docs/ENTERPRISE_SUPPORT_CASE_MANAGEMENT.md`.
  - Chose STLoads-native support cases as the initial system of record with future external helpdesk integration points.
  - Defined case model, intake, SLA management, escalation, customer updates, feedback loop, reporting, tooling requirements, and verification checklist.
  - Earlier audit correction found this was only documented; the implementation below closes that gap.
  - Added `0067_support_case_management.sql` with support-case and support-case-event tables.
  - Added admin APIs and shared DTOs for case list/create/update/feedback.
  - Added support-console UI for case creation, resolution, and CSAT recording.
  - Verified focused backend support-case acceptance test, backend/frontend checks, frontend release build, clippy, and security scan.
  - Linked support-case documentation from `README.md` and `docs/MASTER_PLAN.md`.

## Phase 1: Production Safety Foundation

### ENT-0101 Add Production Runtime Guardrails

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Production cannot serve fallback screen data.
  - Production cannot boot with permissive CORS.
  - Tests cover production vs development behavior.
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - A failed deploy has a written recovery path.
  - Rollback plan is tested at least once in staging.
- Verification notes:
  - Created `docs/ENTERPRISE_ROLLBACK_RUNBOOK.md`.
  - Documented backend rollback, frontend rollback, database rollback/forward-fix policy, object-storage rollback, payment/TMS/notification cautions, staging rollback drill, and release record requirements.
  - Linked rollback runbook from `README.md`, `docs/MASTER_PLAN.md`, and `docs/IBM_CODE_ENGINE_DEPLOYMENT.md`.
  - Pending: run the staging rollback drill and record revision IDs/results before marking this task complete.

### ENT-0106 Add Feature Flags And Change Approval

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Product/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Cutover can be rehearsed in staging with production-like data.
  - Business users can validate migrated data before the Rust platform becomes authoritative.
- Verification notes:
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
  - Next action: produce the approved legacy summary JSON, rerun reconciliation with `-ExpectedJsonPath`, and record business validation signoff.

## Phase 2: Auth, Authorization, And Tenant Boundaries

### ENT-0201 Hash Session Tokens

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0201`
- Estimate: `1 day`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Browser session handling is safe across normal domains and any supported custom domains.
  - CSRF, session fixation, and permissive CORS risks are explicitly tested or formally ruled out.
- Verification notes:
  - Added `docs/ENTERPRISE_BROWSER_SESSION_SECURITY.md` for bearer-auth, CSRF decision, cookie migration gate, CORS, custom domains, and fixation controls.
  - Verified by static search that there is no backend `Set-Cookie` auth path and that frontend auth uses explicit bearer headers.
  - Added auth-session tests for bearer-only authentication, cookie-only rejection, and non-bearer rejection.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend auth_session::tests`: 5 passed.

### ENT-0202 Add Rate Limiting And Account Lockout

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0201A`
- Estimate: `2 days baseline + distributed hardening`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Brute-force attempts are blocked.
  - Legitimate users receive clear recovery guidance.
- Verification notes:
  - Added `crates/backend/src/rate_limit.rs` with fixed-window limits, account/OTP failure lockout, forwarded-IP fingerprinting, and success cleanup.
  - Added shared `RateLimiter` to `AppState`.
  - Wired rate limits into auth, OTP/password recovery, KYC uploads, dispatch/execution document upload/read paths, Stripe Connect onboarding, escrow fund/hold/release, Stripe webhooks, and TMS webhooks.
  - Added Postgres migration `0009_security_rate_limits.sql` and DB-backed throttle helpers in `crates/db/src/security.rs`.
  - Switched route checks through `AppState` helpers so production uses shared Postgres counters and local/dev can fall back to process-local counters.
  - Added `scripts/security_rate_limit_status.ps1` for support inspection and controlled lockout clearing.
  - Added `docs/ENTERPRISE_RATE_LIMITING_AND_LOCKOUT.md`.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p db`: 7 passed.
  - Verified `cargo test -p backend`: 36 passed.
  - Verified `cargo test --workspace`.

### ENT-0202A Add Distributed Rate Limit Store And Lockout Visibility

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security/Platform/Support
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0202`
- Estimate: `3 days`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Rate limits and account lockouts are enforced consistently across multiple backend replicas.
  - Support can explain and remediate a locked account without database spelunking.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0201`, `ENT-0202`
- Estimate: `3 days`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Privileged accounts cannot perform high-risk actions without MFA.
  - Recovery flow is auditable.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0203`
- Estimate: `2 days`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - All new data is tenant-scoped where appropriate.
  - Existing data is assigned to a default organization without breaking current workflows.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0204`
- Estimate: `Complete`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - One customer cannot access another customer's data.
  - Tenant isolation tests fail before implementation and pass after implementation.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0204`, `ENT-0205`
- Estimate: `Complete`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Every route and action maps to a permission.
  - Role changes invalidate affected sessions.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Support can help customers without unsafe database access.
  - Every support action is auditable.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise customers can manage users through their identity provider.
  - Deprovisioning removes access without manual STLoads intervention.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Internal and customer-facing privileged access can be recertified on a schedule.
  - Stale, excessive, or emergency access does not remain active silently.
- Verification notes:
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
- Verification notes:
  - `TBD`

## Phase 3: Audit, Compliance Evidence, And Governance

### ENT-0301 Add Global Audit Ledger

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - All high-risk workflows write audit events.
  - Audit events are queryable by entity and actor.
- Verification notes:
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
- Verification notes:
  - `TBD`

### ENT-0302 Add Request Correlation IDs

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - A single incident can be traced across API, DB, jobs, payments, and integrations.
- Verification notes:
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
- Verification notes:
  - `TBD`

### ENT-0303 Add Audit Search UI

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Ops can answer who changed what without developer help.
- Verification notes:
  - Started after ENT-0302 completion.
  - Added shared audit search/export DTOs for backend and Leptos contracts.
  - Added backend `/admin/audit` search and `/admin/audit/export` routes scoped by organization with break-glass enforcement for cross-organization access.
  - Search supports actor user, organization, entity type, entity ID, action, request ID, date range, and free-text metadata/reason matching.
  - Export uses the same filters and returns compliance-ready CSV content.
  - Added Leptos `/admin/audit` page with filters, results table, sidebar/quick-jump navigation, and CSV export preview.
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - No workflow status can be changed casually without a migration/test/update plan.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Legal acceptance can be proven for every required operational agreement.
  - Updated terms can be rolled out and tracked by version.
- Verification notes:
  - Started after ENT-0304 completion.
  - Added migration `0019_legal_agreements.sql` with versioned legal agreement templates and signer acceptance proof records.
  - Seeded active templates for platform terms, privacy policy, tracking consent, payment terms, carrier operating agreement, broker/customer contract terms, shipper customer contract terms, and freight forwarder terms.
  - Added `db::legal_agreements` for required-agreement lookup, missing-agreement detection, acceptance proof storage, and audit-event linkage.
  - Added auth API DTOs and backend routes `GET /auth/legal-agreements` and `POST /auth/legal-agreements/accept`.
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - The company can prove it is legally allowed and insured to operate under the chosen business model.
  - Enterprise customers can receive required authority, bond, and insurance evidence without ad hoc legal work.
- Verification notes:
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

### ENT-0401 Harden Local File Reads

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Malicious `../` or crafted local paths cannot escape storage root.
- Verification notes:
  - Hardened `DocumentStorageService::read_document` and `delete_document` for local storage with safe relative-path validation and canonical storage-root containment checks.
  - Rejects `../`, absolute paths, root-prefixed paths, and Windows-style backslash traversal before reading or deleting local files.
  - Keeps valid generated local paths readable under the configured storage root.
  - Added document storage tests `local_read_allows_canonical_path_inside_storage_root`, `local_read_rejects_path_traversal_attempts`, and `local_delete_rejects_path_traversal_attempts`.
  - Verified `cargo test -p backend document_storage`: 4 passed.
  - Verified `cargo check -p backend`.
  - Completion decision: local document reads/deletes now canonicalize and enforce storage-root containment, so crafted local paths cannot escape the configured document root.

### ENT-0402 Add Document Versioning

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Replacing a document does not destroy audit history.
- Verification notes:
  - Added version tables for load, KYC/profile, and execution leg documents with prior file metadata, uploader, hash/blockchain fields where applicable, replacement reason, and rollout backfill.
  - Added document version counters and labels to API payloads and displayed them on onboarding, profile, admin user, load profile, and execution document screens.
  - Preserved prior KYC local files during replacement instead of deleting the old file immediately.
  - Added `kyc_document_replacement_preserves_version_history` integration coverage.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, `trunk build --release`, and `git diff --check`.

### ENT-0403 Add Required Document Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Closeout and compliance readiness are machine-checkable.
- Verification notes:
  - Added seeded `required_document_rules` for onboarding, load profile, and execution closeout requirements.
  - Added checklist payloads and UI rendering to onboarding, self-profile, load profile, and execution screens.
  - Blocked carrier/broker onboarding submission when required onboarding documents are missing.
  - Kept delivery completion blocked by POD-plus-note guard and surfaced the POD requirement as checklist data.
  - Added backend unit coverage for carrier required documents and DB integration coverage for seeded checklist rules.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, `trunk build --release`, and `git diff --check`.

### ENT-0404 Replace Mock Blockchain Proof

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - UI no longer claims mock proof as real proof.
  - Hash verification uses actual uploaded file bytes.
- Verification notes:
  - Replaced generated mock hash tokens with SHA-256 hashes calculated from document bytes.
  - Load document verification reads stored document bytes before persisting hash evidence.
  - KYC/profile upload, replacement, and explicit verification paths now store `hash_algorithm = 'sha256'` from real bytes.
  - User-facing document copy now says content hash/SHA-256 rather than presenting mock blockchain proof as real.
  - External timestamping/blockchain provider integration remains deferred until a real provider is selected.
  - Added backend SHA-256 unit coverage and DB coverage that rejects mock hash tokens.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, `trunk build --release`, and `git diff --check`.

### ENT-0405 Add File Validation And Scanning Hook

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Dangerous or invalid uploads are blocked or quarantined.
- Verification notes:
  - Added shared upload validation for KYC/profile, load, and execution documents.
  - Enforced 25 MB upload limit, blocked executable/script/web extensions, and added MIME sniffing for common enterprise document/image types.
  - Added scanner-hook verdict status for a future malware scanning provider.
  - Added backend tests for blocked extensions, MIME mismatches, and scanner-hook acceptance.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, `trunk build --release`, and `git diff --check`.

### ENT-0407 Add Freight Document Templates And Packets

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Operators can generate and retrieve standard freight documents without manual template work.
  - Generated documents are linked to load, carrier, customer, audit, and document history.
- Verification notes:
  - Added versioned freight document templates and a generated-document ledger.
  - Seeded rate confirmation, BOL, carrier packet, and shipper package templates.
  - Added backend generation that renders templates from load context, stores generated files, creates load document rows, and records template/version evidence.
  - Added load-profile UI action to generate standard freight documents and refresh retrieval.
  - Added DB integration coverage for seeded templates, load/carrier context, document linkage, and generation ledger.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, `trunk build --release`, and `git diff --check`.

## Phase 5: Load Posting, Search, And Customer Rules

### ENT-0501 Finish Enterprise Load Model

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0407`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise freight can be posted without out-of-band notes.
- Verification notes:
  - Added migration `0024_enterprise_load_model.sql` with enterprise core load fields for mode, visibility, service level, references, appointment windows, facility contact, accessorials, temperature, container, and securement data.
  - Extended shared DTOs, DB create/update/read paths, TMS load materialization, backend validation, load profile fields, and the Leptos load builder.
  - Added DB integration coverage for enterprise field create/update persistence.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, `trunk build --release` from `crates/frontend-leptos`, and `git diff --check`.

### ENT-0502 Add Draft/Publish/Revise/Cancel/Archive/Clone

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0501`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Users can manage common lifecycle actions safely.
- Verification notes:
  - Added migration `0025_load_lifecycle_actions.sql` for lifecycle state, revisions, clone source, template markers, lifecycle timestamps, and reason text.
  - Added shared lifecycle DTOs, DB lifecycle/clone helpers, backend lifecycle action route, TMS published defaults, and load-profile lifecycle action buttons.
  - Enforced booked/execution-stage locking for unsafe lifecycle actions.
  - Extended DB integration coverage for publish, revise, save-as-template, and clone-to-draft.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.

### ENT-0503 Add Customer Contract And Lane Guide Model

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0501`, `ENT-0502`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise shippers can run private/contract freight, not only public spot loads.
- Verification notes:
  - Added migration `0026_customer_contract_lane_guides.sql` for customer contracts, lanes, preferred/backup carriers, effective dates, service rules, contracted rates, and posting behavior.
  - Added load contract/lane linkage fields and DB helpers for contract/lane creation plus active lane lookup.
  - Extended load DTOs, DB load create/update/clone/read paths, backend contract-lane enrichment, and load-profile contract visibility.
  - Added DB integration coverage proving active contract lanes attach contract pricing and visibility to posted loads.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.

### ENT-0504 Add Saved Filters And Load Search

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0501`, `ENT-0503`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Load board stays usable as volume grows.
- Verification notes:
  - Added migration `0027_load_board_search_saved_filters.sql` for saved views and search indexes.
  - Added shared load-board filter and saved-view DTOs.
  - Added DB-backed auth-scoped filtered search with pagination, tab scope, date, location, equipment, commodity, rate, customer/reference, status, compliance, and visibility filters.
  - Updated load-board screen assembly and Leptos controls for filters, saved views, row count, reset, and previous/next paging.
  - Extended DB integration coverage for saved filters and filtered contract lane search.
  - Verified `cargo fmt --check`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.

### ENT-0505 Add Bulk Import And API Posting

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0501`, `ENT-0504`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise customers can post high-volume loads without manual entry.
- Verification notes:
  - Added migration `0028_bulk_load_import_api_posting.sql` for load import batches, row validation, idempotency keys, error CSV exports, and created-load linkage.
  - Added shared CSV import and API post contracts.
  - Added backend preview, commit, and idempotent API posting routes under `/dispatch/loads`.
  - Added CSV parser/validator coverage for required enterprise load fields, date ordering, bid status, weight units, and pricing.
  - Added load-board UI controls for CSV preview, commit, import result counts, and failed-row export review.
  - Verified `cargo fmt --check`, `cargo test -p backend`, `cargo test -p db`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.

### ENT-0506 Add Rating, Mileage, Fuel, And Accessorial Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0501`, `ENT-0503`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Rates can be calculated and explained before booking, billing, and settlement.
  - Manual pricing changes are auditable.
- Verification notes:
  - Added migration `0029_rating_mileage_fuel_accessorial_rules.sql` for mileage rules, fuel rules, accessorial catalog, rating quotes, and manual override audit.
  - Seeded the enterprise accessorial catalog for detention, layover, lumper, stop-off, TONU, chassis, storage, tolls, and special handling.
  - Added shared rate calculation contracts and the authenticated `/dispatch/loads/{load_id}/rating/calculate` route.
  - Added explainable rate calculation from contract/leg base rates, mileage rules, mileage overrides, fuel rules, accessorial flags, carrier rate, margin, and manual override audit events.
  - Added latest rating summary and a calculate-rate action on the Rust load profile.
  - Added backend unit coverage for accessorial amount/default handling.
  - Verified `cargo fmt --check`, `cargo test -p backend`, `cargo test -p db`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.

### ENT-0507 Add Address Validation, Geocoding, And Facility Scheduling

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0501`, `ENT-0502`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Pickup and delivery locations are operationally usable, not just text addresses.
  - Appointment changes are visible to carrier, shipper, operator, and audit history.
- Verification notes:
  - Added migration `0030_facility_geocoding_appointments.sql` for normalized geocoded locations, facilities, facility notes, appointments, and appointment event history.
  - Preserved Google place ids, latitude/longitude, validation status, and facility type from Rust load builder addresses.
  - Added coordinate validation for complete and in-range geocode pairs.
  - Added authenticated `/dispatch/loads/{load_id}/appointments` route for scheduling and rescheduling pickup/delivery appointments.
  - Appointment changes write facility appointment rows, appointment events, load history, and realtime operator notifications.
  - Added load-profile scheduling controls for leg, stop type, appointment start/end, dock, and notes.
  - Added backend unit coverage for coordinate validation.
  - Verified `cargo fmt --check`, `cargo test -p backend`, `cargo test -p db`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.

### ENT-0508 Add Mode-Specific Workflow Tracks

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0501`, `ENT-0507`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Unsupported modes are blocked or clearly marked as out of scope.
  - Supported modes have enough structure to operate without free-text workarounds.
- Verification notes:
  - Added migration `0031_mode_specific_workflow_tracks.sql` for freight mode rules, mode validation status, and validation event history.
  - Defined first-release supported modes as `FTL`, `LTL`, `drayage`, and `intermodal`.
  - Deferred `cross_border`, `freight_forwarding`, and `mixed_mode` with explicit validation messages until later legal, tax, FX, customs, segment, and localization work is complete.
  - Added backend mode validation across load create/update, CSV commit, and API posting paths.
  - Added structured mode-detail requirements for LTL, drayage, and intermodal.
  - Recorded mode validation events after successful create/update.
  - Updated the load builder mode selector to show supported modes and disabled deferred modes.
  - Added backend unit coverage for deferred-mode blocking and structured drayage payload validation.
  - Verified `cargo fmt --check`, `cargo test -p backend`, `cargo test -p db`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.

### ENT-0509 Add Time Zone, Unit, Currency, And Localization Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0507`, `ENT-0508`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Operators, carriers, and customers do not misread appointment times, units, or rates.
  - Reports and integrations use documented canonical units.
- Verification notes:
  - Added organization localization settings, load localization snapshots, canonical/display weight conversion, appointment time-zone persistence, profile localization summary, and scheduling form time-zone capture.
  - Verified `cargo fmt --check`, `cargo test -p backend` (64 passed), `cargo test -p db` (16 passed), `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.
  - Verified `git diff --check`; only CRLF normalization warnings were reported.

### ENT-0509A Define Cross-Border Tax, FX, Duties, And Incoterms Rules

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Finance/Legal/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0004`, `ENT-0306`, `ENT-0508`, `ENT-0509`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Cross-border and multi-currency freight cannot be billed or settled with ambiguous tax or FX rules.
  - Finance/legal can explain which party owns duties, taxes, and currency risk.
- Verification notes:
  - Added `docs/CROSS_BORDER_TAX_FX_INCOTERMS.md` as the US-domestic/USD-only first-release Finance/Legal decision record.
  - Added `0033_cross_border_tax_fx_incoterms.sql` for cross-border finance policies and future load-level policy checks.
  - Added explicit non-USD escrow funding rejection before Stripe or escrow persistence.
  - Verified `cargo fmt --check`, `cargo test -p backend` (65 passed), `cargo test -p db` (16 passed), `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.
  - Verified `git diff --check`; only CRLF normalization warnings were reported.

### ENT-0510 Add Governed Master Data And Configuration Admin

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise configuration can be changed safely without code deployments or direct database edits.
  - Master-data changes are auditable, reversible, and tested against active freight workflows.
- Verification notes:
  - In progress: added `docs/GOVERNED_MASTER_DATA_CONFIGURATION.md` and migration `0034_governed_master_data_configuration_admin.sql`.
  - Seeded governed service-level, rejection-reason, and exception-reason catalogs plus a governed configuration change ledger.
  - Surfaced governed catalog sections in the Rust master-data screen with active/effective-date and approval-gated labels.
  - Added backend routes and Leptos admin write/archive controls for governed service-level, rejection-reason, and exception-reason catalogs.
  - Governed saves and archives now write `governed_configuration_changes` ledger rows with actor, target, change type, approval status, summary, and effective dates.
  - Added backend route coverage proving governed save/archive ledger entries.
  - Added `0035_operational_catalog_governance.sql` for governed trailer types, hazmat classes, and accessorial governance columns.
  - Extended governed catalog sections, API routes, Leptos write/archive controls, and ledger evidence to trailer types, hazmat classes, and accessorials.
  - Added backend route coverage proving hazmat governed save/archive ledger entries.
  - Added `0036_required_document_rule_governance.sql` plus backend routes, Leptos admin controls, safe archive, screen visibility, and ledger writes for required document rules.
  - Added backend route coverage proving document requirement save/archive ledger entries.
  - Added `0037_customer_specific_configuration_admin.sql` plus backend routes, Leptos admin controls, safe archive, screen visibility, and ledger writes for customer-specific configuration rules.
  - Added governed export, dry-run import, committed import, and rollback execution endpoints with frontend API helpers.
  - Added backend route coverage proving customer-configuration save/archive ledger entries, governed dry-run import, committed import, export visibility, and rollback ledger execution.
  - Verified `cargo fmt --check`, `cargo test -p backend` (65 passed), `cargo test -p db` (16 passed), `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.
  - Verified `git diff --check`; only CRLF normalization warnings were reported.
  - Completion decision: governed master data and customer configuration now have admin write/archive flows, effective-date/approval labels, audit ledger evidence, import/export contracts, rollback execution, and regression coverage.

## Phase 6: Carrier Network, Matching, And Marketplace

### ENT-0601 Build Carrier Capacity Profile

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Carrier eligibility can be computed from structured data.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `Completed`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Shippers control who can see and book private freight.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Matching is explainable to operators and customers.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Offer state cannot become ambiguous or contradictory.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise tendering and spot negotiation are both supported.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Two carriers cannot book the same leg through race conditions.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - A carrier cannot receive restricted freight until packet requirements are complete.
  - Operators can see exactly which packet item blocks eligibility.
- Verification notes:
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

### ENT-0701 Define Canonical Desk Queues

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Every load/leg has a clear operational queue when action is needed.
- Verification notes:
  - Expanded the canonical Rust dispatch desk queue set to quote, tender, facility, in-transit exception, closeout, collections, dispute, reconciliation, and compliance.
  - Added queue-specific entry and exit rule labels to every dispatch desk row so operators know why work is in the queue and what clears it.
  - Added canonical routing aliases for exception/reconciliation/compliance desks while preserving existing quote/tender/facility/closeout/collections routes.
  - Completion decision: the dispatch desk now exposes every Phase 7 canonical queue with clear operational entry/exit rules.

### ENT-0702 Add Assignment, Priority, SLA, And Escalation

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Managers can see backlog, aging, and stuck work.
- Verification notes:
  - Added `0043_dispatch_desk_workflows.sql` with `dispatch_work_items` for assigned owner, priority, SLA due date, escalation reason, and work-item status.
  - Enriched dispatch desk rows with owner, priority badge, SLA due/overdue badge, escalation reason, and queue context.
  - Added manager status cards for unassigned work, SLA-at-risk work, and exception-backed work.
  - Completion decision: managers can scan assignment gaps, aging/SLA risk, and exception backlog directly from the Rust desk.

### ENT-0703 Separate Internal And Customer-Visible Notes

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Internal operational comments cannot accidentally leak to customers.
- Verification notes:
  - Added `dispatch_notes` with explicit `internal` and `customer_visible` visibility.
  - Extended dispatch follow-up requests with visibility and stored notes separately from legacy load-history remarks.
  - Updated the dispatch desk UI with separate internal follow-up and customer-visible update inputs.
  - Displayed latest internal note and latest customer update independently on each desk row.
  - Completion decision: internal comments and customer-visible updates now have separate storage, labels, and UI actions.

### ENT-0704 Add Exception Management

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Operations can resolve freight exceptions without spreadsheets.
- Verification notes:
  - Added `dispatch_exceptions` for stale tracking, late/missing milestone, missing POD, payment hold, compliance block, TMS drift, dispute, and manual exception workflows.
  - Added derived exception signals on the desk for in-transit stale activity, missing POD/closeout notes, payment holds, TMS drift, compliance review, and disputes.
  - Added `/dispatch/desk/legs/{leg_id}/exceptions/resolve` with a resolution note and audit-style dispatch note.
  - Added frontend exception resolution controls on dispatch desk rows.
  - Completion decision: operators can identify and resolve freight exceptions inside the Rust dispatch desk workflow.

## Phase 8: Execution, Tracking, Mobile, And Closeout

### ENT-0801 Centralize Execution State Machine

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Invalid pickup, delivery, or closeout transitions are impossible.
- Verification notes:
  - Added canonical domain execution transition checks and backend enforcement.
  - Verified in `cargo test -p domain`.

### ENT-0802 Build Mobile-First Driver Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Driver tasks work comfortably on a phone.
- Verification notes:
  - Added `/driver/legs/:leg_id` mobile-first driver execution route and reused the hardened execution workflow.

### ENT-0802A Add Mobile Field Capture And Offline Strategy

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Drivers can complete critical field tasks under real mobile network conditions.
  - Offline or delayed submissions are clearly marked and reconciled when connectivity returns.
- Verification notes:
  - Camera-first capture and offline note queueing exist.
  - Added offline queueing for driver actions, GPS pings, and document uploads with selected file bytes preserved for replay.

### ENT-0802B Complete Offline Replay And Mobile Alert Decisions

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0802A`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Offline submissions are automatically retried, reconciled, and visible to operations without manual database work.
- Verification notes:
  - Offline note replay exists.
  - Backend replay now reconciles driver notes, driver actions, GPS pings, and document uploads.
  - Failed replay processing is marked failed in the DB and remains queued locally.
  - Decision: web/PWA replay first; native push, QR/barcode, and OCR move to post-field-test enhancements.

### ENT-0803 Add Tracking Consent And Privacy

- Priority: `P0`
- Status: `[x]`
- Owner: Product/Legal/Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Location tracking is consented, explainable, and retained only as needed.
- Verification notes:
  - Added consent storage, UI capture, customer-visible scope, and consent enforcement before tracking.

### ENT-0804 Add Geofence, ETA, And Delay Detection

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0803`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - System detects execution risk before a customer asks.
- Verification notes:
  - Added domain geofence helpers, ETA/geofence risk labels, and execution risk event scaffolding.

### ENT-0805 Complete POD And Closeout Package

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope completed:
  - [x] Required closeout documents.
  - [x] Leg closeout checklist.
  - [x] POD review and approval.
  - [x] Closeout package export.
- Acceptance criteria:
  - Delivered loads cannot be financially released until closeout rules pass.
- Verification notes:
  - Closeout schema, checklist, approval control, export endpoint, and payment-release guard exist.
  - Added approval endpoint prerequisite enforcement for POD, open finance exceptions, and pending offline replay blockers.
  - Replaced the text-only closeout manifest with a production ZIP package attachment (`application/zip`) containing `manifest.txt` plus embedded document bytes from the configured storage backend.
  - Closeout ZIP generation fails if the required delivery POD cannot be read, preventing incomplete closeout artifacts from being exported as complete packages.
  - Verified `cargo test -p backend execution_routes_enforce_pod_note_and_document_visibility`; the test now opens the exported ZIP and confirms `manifest.txt` plus the uploaded POD file are present.
  - Added secure document endpoint links to every closeout manifest document line.
  - Verified full Phase 8 pass after the ENT-0805 correction: `cargo fmt --all -- --check`, `cargo test -p domain`, `cargo test -p db`, `cargo test -p backend`, `cargo check -p frontend-leptos`, and `trunk build --release` from `crates/frontend-leptos`.
  - Completion decision: ENT-0805 is complete; invoice handoff remains separate Phase 9 finance work and is not counted as part of this task.

### ENT-0806 Add Customer Tracking Page

- Priority: `P2`
- Status: `[x]`
- Owner: Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0803`, `ENT-0804`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Customers can view relevant shipment progress safely.
- Verification notes:
  - Public limited tracking page and token read endpoint exist.
  - Added authenticated create/rotate/revoke APIs with expiration hours, load-history audit notes, and execution realtime updates.
  - Added execution-screen controls to create/rotate links and revoke active customer tracking access.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.

### ENT-0807 Add Claims, Detention, And Accessorial Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend/Frontend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Exceptions that affect billing or settlement are tracked through resolution.
  - Finance can see which charges are approved, disputed, rejected, or pending.
- Verification notes:
  - Finance exception intake, summaries, and payment-release blocking exist.
  - Added approve, reject, dispute, review, and resolved decision workflow by exception type.
  - Decisions update open exceptions, record ownership/time for terminal outcomes, and write invoice/settlement/support timeline context to load history.

### ENT-0808 Add ELD And Telematics Integrations

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Product/Integrations
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0803`, `ENT-0804`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise customers can use automated tracking where carriers support it.
  - Manual/mobile tracking and ELD tracking produce a consistent execution timeline.
- Verification notes:
  - Governed telematics connection model and fallback status exist.
  - Added normalized telematics ping ingest for provider, location, HOS/status, truck, trailer, event type, and recorded timestamp.
  - Telematics pings write to leg locations, leg events, and execution/load history so provider and mobile tracking share one timeline.

### ENT-0809 Add Route Planning And Optimization

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0804`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Mileage, ETA, and pricing are based on a documented route source.
  - Operators can explain route-derived calculations.
- Verification notes:
  - Manual route source capture exists.
  - Route plan save validates provider, mileage, and duration.
  - Route plans write provider, miles, minutes, truck-safe status, constraints, and pricing/ETA/exception/settlement readiness context into load history.
  - Completion decision: first enterprise release supports governed manual or contracted-provider route sources per leg; deeper automated multi-leg optimization follows provider selection.

## Phase 9: Payments, Billing, Settlements, And Finance Controls

Phase 9 checkpoint after repeated verification:
- Status: `[x]`
- Completed in this phase: ENT-0901, ENT-0902, ENT-0903, ENT-0904, ENT-0905, ENT-0906, ENT-0907, ENT-0908, ENT-0909, ENT-0910.
- Completion decision: Phase 9 is finalized for the enterprise Rust payments release. Money movement now has idempotency, webhook replay protection, ledger reconstruction, high-value release approvals, invoice/settlement generation, accounting export, explicit unsupported carrier-finance decisions, platform billing, shipper credit/AR controls, and payout destination review controls.
- Honest deferred finance integrations: automatic external billing-provider card collection, external dunning email/SMS campaigns, QuickBooks/NetSuite/Xero direct sync, factoring/quick-pay/fuel finance products, and non-Stripe payout-provider integrations remain future tasks and must not be treated as enabled.
- Verification run: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo check -p backend`, `cargo check -p frontend-leptos`, `cargo test -p backend routes::payments::tests`, `cargo test -p db escrow_transition_updates_leg_status_and_history`, `cargo test -p db finance_release_approval_requires_two_distinct_approvals`, `cargo test --workspace`, and `git diff --check`.

### ENT-0901 Add Payment Idempotency

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope completed:
  - [x] Escrow fund idempotency.
  - [x] Escrow hold idempotency.
  - [x] Escrow release idempotency.
  - [x] Stripe PaymentIntent idempotency key forwarding.
  - [x] Stripe Transfer idempotency key forwarding.
  - [x] Stored successful response replay.
- Remaining scope:
  - [x] Refund endpoint idempotency after refund workflow exists.
  - [x] Adjustment endpoint idempotency after adjustment workflow exists.
- Acceptance criteria:
  - Repeated requests cannot double-charge or double-release.
- Verification notes:
  - Existing migration `0046_payment_idempotency.sql` creates unique payment idempotency key storage.
  - Added deterministic server-side fallback idempotency keys for supported escrow operations when callers omit a key.
  - Added release idempotency replay and response persistence.
  - Stripe PaymentIntent and Transfer calls now send `Idempotency-Key`.
  - Verified `cargo test -p backend routes::payments::tests`.
  - Verified `cargo check -p backend`.
  - Added idempotent full-refund endpoint and idempotent manual adjustment endpoint.
  - Completion decision: supported Rust payment money-movement and finance-event routes are now idempotent.

### ENT-0902 De-Duplicate Stripe Webhooks

- Priority: `P0`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope completed:
  - [x] Stripe event ID storage.
  - [x] Duplicate event rejection before processing.
  - [x] Replay/duplicate claim tests.
- Acceptance criteria:
  - Stripe retry behavior cannot corrupt escrow state.
- Verification notes:
  - Added migration `0047_stripe_webhook_deduplication.sql` with durable `payment_stripe_webhook_events` storage and a unique `stripe_event_id` claim.
  - Real Stripe webhook parsing now preserves the top-level Stripe event ID.
  - Stripe webhook handling now returns an acknowledged duplicate response without mutating escrow, payout, or user state if the event was already claimed.
  - Verified `cargo test -p backend routes::payments::tests`.
  - Verified `cargo test -p db stripe_webhook_event_claims_are_idempotent`.
  - Verified `cargo check -p frontend-leptos`.

### ENT-0903 Add Payment Ledger

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope:
  - [x] Add durable payment ledger table.
  - [x] Add ledger entries for escrow funded, hold, release, carrier transfer, platform fee earned, refund support, adjustment support, dispute support, and payout failure support.
  - [x] Link ledger entries to Stripe PaymentIntent, charge, transfer, load, leg, escrow, payer, payee, and actor IDs.
  - [x] Link ledger entries to concrete audit event IDs when payment routes emit audit events.
  - [x] Wire refund/dispute/adjustment endpoint flows to the ledger when those finance workflows are implemented.
- Acceptance criteria:
  - Finance can reconstruct every cent.
- Verification notes:
  - Added migration `0048_payment_ledger.sql`.
  - Escrow transitions now write ledger rows inside the same transaction as escrow, leg-status, and load-history changes.
  - Funded escrows create `escrow_funded` rows; held escrows create `escrow_hold`; released escrows create `carrier_transfer` and `fee_earned`; refunded and failed statuses have ledger support.
  - Added DB helper support for manual/future finance ledger entries.
  - Extended DB integration coverage for funded, carrier-transfer, and fee-earned rows with Stripe IDs and exact cents.
  - Verified `cargo test -p db escrow_transition_updates_leg_status_and_history`.
  - Verified `cargo check -p db`.
  - Escrow transitions now create audit events inside the same transaction and link payment ledger rows to those audit IDs.
  - Added MFA-protected refund, adjustment, and dispute routes that write idempotent ledger evidence.
  - Extended DB integration coverage for refund, adjustment, and dispute ledger rows.
  - Completion decision: finance can reconstruct every cent represented by the Rust payment flows with linked escrow, Stripe/reference, actor, load, leg, and audit evidence.

### ENT-0904 Add Finance Approval Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope:
  - [x] Add persisted finance approval request model.
  - [x] Add manual release approval endpoint.
  - [x] Add two-person approval threshold for high-value releases.
  - [x] Block high-value release until approval threshold is satisfied.
  - [x] Add step-up MFA for release and approval.
  - [x] Add frontend finance approval queue/action surface.
  - [x] Add explicit manual hold approval workflow if Finance wants holds to require approval before placement.
- Acceptance criteria:
  - High-risk payouts cannot be released casually.
- Verification notes:
  - Added migration `0049_finance_approval_workflow.sql`.
  - Added DB helpers to create/reuse approval requests, approve with distinct approvers, and verify release approval.
  - Added MFA-protected `POST /payments/legs/{leg_id}/release-approval`.
  - Release route now blocks payout at or above 500000 cents until two finance approvals exist.
  - Added `GET /payments/finance-approvals` for pending release and hold approvals, while keeping the release-approval route compatible.
  - Added a Rust admin payments finance approval queue/action surface with type-aware approve actions.
  - Added DB integration coverage for two-person approval behavior.
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope:
  - [x] Add customer invoice and invoice line models.
  - [x] Add carrier settlement and settlement line models.
  - [x] Add platform fee, accessorial, tax, adjustment, payment-term, and status lifecycle fields.
  - [x] Generate invoice and carrier settlement records when escrow is released.
  - [x] Add frontend finance screens for invoices and settlements.
  - [x] Add editable accessorial, tax, and adjustment workflows before invoice issuance.
- Acceptance criteria:
  - Platform supports billing and settlement, not only escrow.
- Verification notes:
  - Added migration `0050_invoices_carrier_settlements.sql`.
  - Released escrow transitions now generate issued customer invoices and released carrier settlements from exact escrow cents.
  - Added DB helper to fetch the invoice/settlement package for a load leg.
  - Added `GET /payments/invoice-settlements` and a Rust admin payments invoice/settlement table.
  - Payment adjustments now update customer invoice totals and carrier settlement net amounts while writing ledger evidence.
  - Extended DB integration coverage for invoice total, gross settlement, platform fee, and net settlement.
  - Extended DB integration coverage for adjustment updates to invoice and settlement totals.
  - Verified `cargo test -p db escrow_transition_updates_leg_status_and_history`.
  - Verified `cargo check -p backend`.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: finance-facing invoice/settlement visibility and controlled adjustment updates are now available in Rust.

### ENT-0906 Add Accounting Export

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0903`, `ENT-0905`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope:
  - [x] Add CSV export first.
  - [x] Design QuickBooks/NetSuite/Xero integration later if needed.
- Acceptance criteria:
  - Finance can reconcile outside the app without custom SQL.
- Verification notes:
  - Added accounting export row projection over payment ledger entries joined to invoice and settlement references.
  - Added `GET /payments/accounting/export`.
  - Export includes ledger, exact cents, Stripe, invoice, and settlement fields.
  - Verified `cargo check -p backend`.
  - Verified `cargo test -p db escrow_transition_updates_leg_status_and_history`.

### ENT-0907 Add Factoring, Advances, And Fuel Support Decision

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Finance/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0903`, `ENT-0905`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope:
  - [x] Decide whether STLoads supports factoring, quick pay, fuel advances, fuel cards, or carrier advances.
  - [x] Add finance controls, fees, eligibility, audit, and repayment/settlement treatment for any supported option.
  - [x] Add explicit unsupported-state messaging if deferred.
- Acceptance criteria:
  - Carrier payment options are either supported safely or clearly out of scope.
  - Finance can reconcile advances and fees if enabled.
- Verification notes:
  - First enterprise release decision: all carrier finance products are deferred.
  - Added `GET /payments/carrier-finance/options` with explicit deferred-state messaging.
  - No advance or quick-pay money movement is enabled, so there are no unsupported fees or repayment rows to reconcile yet.

### ENT-0908 Add STLoads Subscription And Usage Billing

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Finance/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0905`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope:
  - [x] Decide commercial model: `hybrid_subscription_usage`.
  - [x] Add subscription plan, billing account, payment method status, renewal/cancellation period, and usage tracking models.
  - [x] Keep platform billing separate from freight escrow, carrier settlement, and shipper invoice workflows.
  - [x] Add platform subscription invoice generation and collection workflow.
  - [x] Add frontend billing account management.
- Acceptance criteria:
  - STLoads can bill enterprise customers for platform usage without mixing it with freight money movement.
- Verification notes:
  - Added subscription and usage tables in `0051_subscription_credit_payout_controls.sql`.
  - Added `GET /payments/platform-billing/model`.
  - Added `0052_phase9_finance_completion.sql` platform invoice records.
  - Added finance routes to list platform billing accounts, generate platform invoices from base plan plus usage totals, and mark platform invoices paid.
  - Added Rust admin payments UI for platform billing accounts with plan, billing status, payment method status, latest invoice, open balance, past-due balance, invoice generation, and mark-paid collection action.
  - Completion decision: STLoads can bill enterprise customers for platform subscription/usage without mixing those receivables with freight escrow, shipper invoice, or carrier settlement money movement. Automatic external card collection remains a future billing-provider integration.

### ENT-0909 Add Shipper Credit, AR Aging, And Collections Controls

- Priority: `P1`
- Status: `[x]`
- Owner: Finance/Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0905`, `ENT-0908`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Scope:
  - [x] Add customer credit status, credit limit, payment terms, credit hold, and risk-note model.
  - [x] Add AR collections note and promise-to-pay model.
  - [x] Add AR aging calculation job/query.
  - [x] Add dunning and collections queue UI.
  - [x] Decide and enforce posting/tendering/booking/release policy when over limit or on credit hold.
  - [x] Add audit and approval for credit limit changes and credit hold overrides.
  - [x] Add customer-visible payment status without internal risk notes.
- Acceptance criteria:
  - Finance can prevent new exposure from high-risk or overdue shippers.
  - Operators know when a load is blocked by credit policy and how to escalate.
- Verification notes:
  - Added shipper credit and AR collections tables in `0051_subscription_credit_payout_controls.sql`.
  - Added finance route and Rust admin payments UI that calculates live open and overdue AR from customer invoices, shows credit/collections status, and exposes a controlled credit override action.
  - Escrow release is blocked when the payer is on credit hold, over limit, in collections, or has overdue AR unless an approved unexpired finance override exists.
  - Credit override approvals are MFA-protected finance actions, update the credit account, and persist a `shipper_credit_override_requests` approval record with expiry.
  - Customer-facing payment exposure is represented through invoice/payment status and finance-safe credit status; internal risk notes stay in the operator finance view.
  - Completion decision: Finance can prevent release exposure from high-risk or overdue shippers, operators have a queue and escalation action, and external campaign dunning remains a future communications integration.

### ENT-0910 Add Bank Account And Payout Change Controls

- Priority: `P0`
- Status: `[x]`
- Owner: Finance/Security/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0904`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
  - Added payout destination review table in `0051_subscription_credit_payout_controls.sql`.
  - Added `GET /payments/payout-change-controls`.
  - Stripe Connect account recreation now detects an existing changed payout destination and creates a cooling-off payout review with `notification_sent_at` evidence.
  - Payout destination changes now enqueue/send a carrier email notification through the Rust mail service/outbox with the old and new destination references and finance-review hold notice.
  - Escrow release is blocked when the carrier has an open payout destination review in review-required, cooling-off, or blocked status.
  - Added finance review queue and approve/reject actions to the Rust admin payments UI.
  - Completion decision: payout destination changes cannot silently redirect carrier money in the Rust release path. Email notification is wired through the existing outbox; SMS and non-Stripe provider matching remain future notification/provider integrations.

## Phase 10: Compliance, Risk, And Fraud

### ENT-1001 Split Compliance Models

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `Phase 2 KYC/document foundations`, `ENT-0904`, `ENT-0910`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1001`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1001`, `ENT-1002`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1001`, `ENT-1002`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1001`, `ENT-1003`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1004`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1003`, `ENT-1004`, `ENT-1005`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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

### ENT-1101 Publish OpenAPI Specs

- Priority: `P1`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `Phase 10`, `existing Rust route contracts`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1101`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1101`, `ENT-1102`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1101`, `ENT-1103`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1103`, `ENT-1104`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1101`, `ENT-1102`, `ENT-1104`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1101`, `ENT-1103`, `ENT-1106`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1106`, `ENT-1107`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1101`, `ENT-1108`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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

### ENT-1201 Add Notification Center

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `Phase 11 integration events`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1201`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1202`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1201`, `ENT-1202`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Critical messages have observable delivery status and safe fallback/escalation behavior.
  - Message templates and sender identities can be changed without accidental compliance or deliverability regressions.
- Verification notes:
  - Added migration `0063_deliverability_branding_controls.sql` with sender identities, delivery events, suppression entries, template governance, and high-risk monitoring rules.
  - Seeded development/production sender identities, pending approval for high-risk email templates, and monitoring rules for OTP, password reset, tender, pickup/delivery, POD rejection, payment hold, and payout release.
  - Added authenticated communication-governance API visibility and a template test-send audit route.
  - Added deliverability governance sections to `/notifications`.
  - Added DB integration coverage for verified sender identity state, bounce-to-suppression handling, seeded high-risk monitoring, and governed high-risk templates.
  - Verified `cargo test -p db message_deliverability_and_branding_governance_are_controlled`.
  - Verified `cargo check --workspace`.
  - Verified `cargo test --workspace`.
  - Completion decision: ENT-1205 is complete for first enterprise release governance. Actual external SMS/push providers remain deferred behind the selected/deferred channel decision and compliance controls from ENT-1203.

### ENT-1206 Add Tenant Branding And Customer-Facing Identity Controls

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Frontend/Backend/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-0805`, `ENT-1205`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Customer-facing branding is controlled, secure, and consistent across portals, documents, and messages.
  - Unsupported branding promises cannot slip into sales or implementation without product approval.
- Verification notes:
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

### ENT-1301 Create Shared UI Component Library

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - New screens use shared components by default.
- Verification notes:
  - Added `crates/frontend-leptos/src/components` with shared primitives for page headers, panels, toolbars, filter bars, table shells, status pills, badges, toasts, modals, drawers, timelines, file upload frames, map panels, money inputs, confirmation dialogs, and field errors.
  - Added design-system CSS classes to `crates/frontend-leptos/index.html`.
  - Refactored the dashboard page to consume the shared page header, panel, and status pill primitives.
  - Added `docs/FRONTEND_COMPONENT_LIBRARY.md` as the required default for new Leptos screens and major page edits.
  - Verified `cargo check -p frontend-leptos`.
  - Completion decision: ENT-1301 is complete as the first shared UI foundation. ENT-1302 remains responsible for migrating the large legacy pages onto these primitives.

### ENT-1302 Refactor Large Leptos Pages

- Priority: `P2`
- Status: `[x]`
- Owner: Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1301`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Large page files become maintainable and testable.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Critical flows are keyboard and screen-reader usable.
  - Accessibility exceptions are documented with owners and remediation dates.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Each role lands on actionable work, not a generic page.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - UI regressions are caught before production.
- Verification notes:
  - Added Playwright test harness with `package.json`, `package-lock.json`, and `playwright.config.ts`.
  - Added Trunk web-server integration so browser tests build and serve the Leptos frontend automatically.
  - Added desktop and mobile Chromium projects.
  - Added smoke coverage for `/`, `/auth/login`, `/dashboard`, `/loads`, and `/notifications`.
  - Added visual baselines for login and public landing pages under `tests/e2e/__screenshots__`.
  - Verified first-run snapshot creation with `npx playwright test --update-snapshots`.
  - Verified repeatable regression run with `npx playwright test`: 14 passed.
  - Completion decision: ENT-1305 is complete for the first browser smoke and visual-regression gate. Deeper authenticated workflow journeys can build on this harness as seeded test accounts stabilize.

## Phase 14: Data, Reporting, Search, And Intelligence

### ENT-1401 Define Business Metrics

- Priority: `P1`
- Status: `[x]`
- Owner: Product/Data/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `Phase 13 frontend/admin foundation`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Metrics definitions are documented and accepted by product/ops.
- Verification notes:
  - Added `docs/ENTERPRISE_REPORTING_METRICS.md` with first-release metric definitions, owner, grain, cadence, and operating rules.
  - Added migration `0064_reporting_metrics_scorecards.sql` with seeded `business_metric_definitions` for posted loads, booked loads, acceptance rate, quote-to-book time, tracking compliance, on-time pickup, on-time delivery, document cycle time, margin, payout time, and dispute rate.
  - Added DB access helpers in `crates/db/src/reporting.rs`.
  - Verified `cargo test -p db reporting_metrics_and_scorecards_are_seeded_and_queryable`.
  - Completion decision: ENT-1401 is complete for accepted metric definitions and queryable seeded metadata.

### ENT-1402 Add Reporting Data Model

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1401`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Reports do not slow operational screens.
- Verification notes:
  - Added `reporting_read_models` and `reporting_refresh_runs` to track read-model source tables, target tables, refresh strategy, cadence, owner, refresh status, row counts, and errors.
  - Seeded read-model contracts for load operational metrics, finance metrics, customer scorecards, and carrier scorecards.
  - Documented that reporting is eventually consistent and operational screens must not depend on expensive aggregate queries over hot workflow tables.
  - Verified `cargo test -p db reporting_metrics_and_scorecards_are_seeded_and_queryable`.
  - Completion decision: ENT-1402 is complete for the reporting data-model foundation and refresh governance contract.

### ENT-1403 Add Customer And Carrier Scorecards

- Priority: `P2`
- Status: `[x]`
- Owner: Product/Data/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1401`, `ENT-1402`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Operators can make carrier/customer decisions from historical data.
- Verification notes:
  - Added `customer_scorecards` for posted/booked loads, acceptance rate, quote-to-book time, on-time service, document cycle time, margin, dispute rate, score, and tone.
  - Added `carrier_scorecards` for offered/accepted loads, acceptance rate, tracking compliance, on-time service, claims, document quality, payout cycle, score, and tone.
  - Added DB query helpers for customer and carrier scorecards.
  - Verified `cargo test -p db reporting_metrics_and_scorecards_are_seeded_and_queryable`.
  - Completion decision: ENT-1403 is complete for scorecard persistence and queryability. Frontend dashboards can now consume these tables/API wrappers in the next reporting UI pass.

### ENT-1404 Add Pricing And Lane Intelligence

- Priority: `P2`
- Status: `[x]`
- Owner: Data/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1401`, `ENT-1402`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Pricing decisions are data-assisted.
- Verification notes:
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
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1402`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Support and operations can find work quickly without unsafe access.
- Verification notes:
  - Added `global_search_documents` as the permission-aware, organization-scoped search index for loads, users, organizations, documents, invoices, payments, conversations, TMS handoffs, and support cases.
  - Added indexed searchable text and per-result permission keys so global search can be broad without bypassing access rules.
  - Added DB query helper `search_global_documents`.
  - Verified permission filtering in `cargo test -p db pricing_search_and_data_quality_controls_are_queryable`.
  - Completion decision: ENT-1405 is complete for the backend/search-index foundation. Existing support/admin screens can consume this helper or wrap it in an admin API route without changing the index contract.

### ENT-1406 Add Data Quality And Integrity Monitoring

- Priority: `P1`
- Status: `[x]`
- Owner: Data/Backend/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1401`, `ENT-1402`, `ENT-1404`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Bad data is detected before it becomes an operational or financial incident.
  - Data quality issues have owners, severity, repair status, and audit trail.
- Verification notes:
  - Added `data_quality_rules`, `data_quality_runs`, and `data_quality_findings`.
  - Seeded rules for orphan records, invalid state combinations, duplicate external references, missing required documents, stale TMS handoffs, unmatched payments, inconsistent tenant ownership, lane-rate anomalies, carrier score changes, suspicious tracking, unusual document replacement, and sudden volume changes.
  - Added severity, owner team, cadence, alert threshold, repair playbook, finding status, repair action, and audit event linkage.
  - Added DB query helpers for active rules and open findings.
  - Verified `cargo test -p db pricing_search_and_data_quality_controls_are_queryable`.
  - Completion decision: ENT-1406 is complete for data-quality governance and findings persistence. Scheduled execution can now evaluate the seeded rule catalog and write findings through this contract.

## Phase 15: Observability, Workers, Scale, And Disaster Recovery

### ENT-1501 Add Structured Logs And Tracing

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `Phase 14 reporting foundations`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Incidents can be traced end to end.
- Verification notes:
  - Added production JSON log mode, OpenTelemetry endpoint config, observability signal catalog, and reliability runbook documentation.
  - Verified `cargo test -p db reliability_operations_contracts_are_seeded_and_jobs_are_recoverable` and `cargo test -p backend config::tests`.

### ENT-1502 Add Metrics And Alerts

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1501`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Team is alerted before customers report major failures.
- Verification notes:
  - Added `alert_rules` and required observability signals for HTTP, DB, queues, workers, email, webhooks, storage, payments, and TMS drift.
  - Verified seeded P0/P1 alert coverage in `reliability_operations_contracts_are_seeded_and_jobs_are_recoverable`.

### ENT-1502A Add On-Call, Escalation, And Security Log Export

- Priority: `P1`
- Status: `[x]`
- Owner: DevOps/Security/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1501`, `ENT-1502`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - P0/P1 alerts page an accountable owner with escalation if not acknowledged.
  - Security and audit logs can be retained, searched, exported, or forwarded according to enterprise commitments.
- Verification notes:
  - Added `on_call_escalation_policies` and `security_log_export_policies`.
  - Documented escalation, SIEM/log-drain, and customer evidence workflow in `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.
  - Added and ran `scripts/run_oncall_siem_drill.ps1` against IBM Code Engine and IBM Cloud Logs on 2026-05-27.
  - Verified backend `/health/live` and `/health/ready`, Code Engine app logs/events, IBM Cloud Logs API query reachability, and simulated P0 acknowledgement/escalation evidence.
  - Completion decision: complete for first enterprise launch evidence; longer retention or external SIEM forwarding can be added per customer contract.

### ENT-1503 Separate Workers From Web Runtime

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `Existing email/TMS workers`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Scaling web traffic does not multiply background job side effects.
- Verification notes:
  - Added explicit `STLOADS_RUNTIME_MODE` / `RUNTIME_MODE` support for `web`, `worker`/`workers`, and `all`.
  - Web runtime no longer starts workers unless worker mode is enabled; worker-only runtime does not bind HTTP and waits for shutdown signal.
  - Verified `cargo test -p backend config::tests`.

### ENT-1504 Add Job Queue And Dead Letter Handling

- Priority: `P1`
- Status: `[x]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1503`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Failed background work is visible and recoverable.
- Verification notes:
  - Added `background_jobs` with retry, lock, visibility timeout, max attempts, dead-letter, and cancellation states.
  - Added DB helpers for claim, dead-letter, and dead-letter listing.
  - Verified claim/dead-letter flow in `reliability_operations_contracts_are_seeded_and_jobs_are_recoverable`.

### ENT-1505 Optimize Database Queries And Indexes

- Priority: `P1`
- Status: `[x]`
- Owner: Backend/DBA
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `Phase 14 global search/reporting`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Critical pages remain fast with production-scale data.
- Verification notes:
  - Added `query_performance_controls` for load board, chat, tracking, admin queues, TMS reconciliation, global search, and reporting scorecards.
  - Captured expected p95, pagination strategy, required indexes, and explain-plan requirement.

### ENT-1506 Define Backup, Restore, RPO, And RTO

- Priority: `P0`
- Status: `[~]`
- Owner: DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1504`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Restore procedure works and is documented.
  - The team knows whether recovery is restore-only or failover-capable for the target release.
- Verification notes:
  - Added `backup_restore_policies` for PostgreSQL, object storage, derived search/reporting rebuild, and queue replay.
  - Documented RPO/RTO and first-release restore/failover posture in `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md`.
  - Added and ran `scripts/run_backup_restore_drill.ps1` on 2026-05-27; IBM managed PostgreSQL scheduled backups were found and Rust staging reconciliation evidence was generated.
  - Partial decision: a temporary restored database/object-storage restore still needs to be provisioned and timed before this can be marked complete.

### ENT-1507 Add Archiving Strategy

- Priority: `P2`
- Status: `[x]`
- Owner: Backend/Data
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1505`, `ENT-1506`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Large history tables do not degrade operational performance.
- Verification notes:
  - Added `archive_policies` for location pings, messages, audit events, TMS handoffs, and document metadata.
  - Captured retention days, archive strategy, restore support, and owners.

### ENT-1508 Add Incident Response, Status Page, And Runbooks

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Ops/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1501`, `ENT-1502A`, `ENT-1506`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - The team can respond to incidents with a documented process.
  - Customers can be informed consistently during outages or degraded service.
- Verification notes:
  - Added `incident_runbooks` for auth outage, database outage, object storage outage, payment incident, duplicate booking, TMS outage, email outage, data exposure, and bad deploy.
  - Documented first-15-minute response, mitigation, status-page/customer communication, and post-incident template.

### ENT-1509 Define Business Continuity And Tabletop Exercises

- Priority: `P1`
- Status: `[x]`
- Owner: Ops/DevOps/Security/Customer Success
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1506`, `ENT-1508`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - The business can continue critical logistics operations during prolonged service degradation.
  - Enterprise customers can see that continuity procedures have been exercised, not only written.
- Verification notes:
  - Added `continuity_exercises` for regional provider outage, payment provider outage, TMS outage, and email/SMS outage tabletop exercises.
  - Captured owner, planned date, evidence URL, gaps found, follow-up owner, and status.
  - Added and ran `scripts/run_business_continuity_tabletop.ps1` on 2026-05-27.
  - Exercised regional provider outage, payment provider outage, TMS outage, and email/SMS outage scenarios with manual fallback procedures.
  - Completion decision: complete for first enterprise launch; status-page vendor, manual POD shift owner, and SMS compliance remain non-blocking follow-ups.

### ENT-1510 Add Cost, Quota, And Usage Guardrails

- Priority: `P1`
- Status: `[x]`
- Owner: DevOps/Product/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `ENT-1502`, `ENT-1504`
- Estimate: `Done`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Production cost spikes are visible before they become business incidents.
  - Heavy customers and integrations are governed without harming normal freight operations.
- Verification notes:
  - Added `usage_quota_policies` and `provider_spend_controls`.
  - Covered document uploads, API calls, webhooks, geocoding, tracking pings, sandbox resets, report exports, notifications, database, object storage, maps, telematics, email, observability, and EDI spend.

## Phase 16: Testing, CI, And Quality Gates

### ENT-1601 Define Test Lanes

- Priority: `P0`
- Status: `[x]`
- Owner: Engineering/QA
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Developers know what to run locally and CI knows what blocks merge.
- Verification notes:
  - Added `docs/ENTERPRISE_TEST_LANES_AND_CI.md` with local and CI lanes for fast Rust, backend integration, frontend release, browser E2E, security, Docker, hosted smoke, and performance smoke.
  - Added lane scripts under `scripts/run_ci_*.ps1` plus `scripts/run_performance_smoke.ps1`.
  - Linked the lane guide from `README.md` and `docs/MASTER_PLAN.md`.

### ENT-1602 Add CI Pipeline

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Main branch cannot receive unverified risky code.
- Verification notes:
  - Added `.github/workflows/ci.yml` for formatting, clippy, tests, conditional SQLx prepare, frontend release build, Playwright E2E, dependency audit, secret scan, and Docker backend/frontend builds.
  - Verified local CI-equivalent gates including fmt, clippy, workspace tests, frontend release build, Playwright, Docker builds, and security scan. Local cargo-audit was skipped; CI installs and runs it.

### ENT-1603 Add Domain State Machine Tests

- Priority: `P0`
- Status: `[x]`
- Owner: Backend/QA
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Invalid transitions are rejected by tests and code.
- Verification notes:
  - Added state-transition helpers and tests for user lifecycle, escrow lifecycle, TMS handoff lifecycle, and TMS sync status lifecycle.
  - Existing offer and execution transition tests remain active.
  - Verified `cargo test -p domain`: 18 passed.

### ENT-1604 Add Security Access Tests

- Priority: `P0`
- Status: `[x]`
- Owner: QA/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - P0 access regressions are caught automatically.
- Verification notes:
  - Confirmed automated access coverage for document visibility, tenant isolation, support audit/scoping, finance approvals, access review revocation, privilege elevation, and sandbox/API lifecycle governance.
  - Hardened hosted smoke scripts so committed default passwords are no longer used.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: secret scan passed and npm audit reported 0 vulnerabilities.

### ENT-1605 Add Load And Performance Tests

- Priority: `P2`
- Status: `[x]`
- Owner: QA/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Performance risks are known before enterprise rollout.
- Verification notes:
  - Added `scripts/run_performance_smoke.ps1` for repeatable health, load board, tracking, admin, TMS webhook, and document-adjacent smoke checks.
  - Added the performance-smoke lane to `docs/ENTERPRISE_TEST_LANES_AND_CI.md`.
  - Existing reliability DB tests verify seeded performance/query-control and quota policy rows.
  - Full representative volume testing remains a future hosted exercise once production-like data and traffic targets exist.

### ENT-1606 Make Clippy Warning-Clean

- Priority: `P1`
- Status: `[x]`
- Owner: Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Clippy warning-clean is added to CI.
  - Any lint allowances are deliberate and documented near the code.
- Verification notes:
  - Removed clippy warnings across frontend, backend, and db helpers.
  - Added documented targeted allowances only for intentional Leptos render/helper signatures.
  - Verified `cargo clippy --workspace --all-targets -- -D warnings`.

### ENT-1607 Add Frontend Release Build To CI

- Priority: `P0`
- Status: `[x]`
- Owner: Frontend/DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Main branch cannot break the Leptos WASM release build.
- Verification notes:
  - Added CI frontend release build with pinned Trunk and wasm target setup.
  - Added frontend runtime config copy-file wiring and deterministic browser bootstrap.
  - Switched Playwright to `trunk serve --release` and refreshed the public landing snapshots.
  - Verified `trunk build --release`.
  - Verified `npx playwright test`: 14 passed.

## Phase 17: Security, Legal, And Enterprise Procurement

### ENT-1701 Perform Threat Model

- Priority: `P0`
- Status: `[x]`
- Owner: Security/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Threats have tracked mitigations.
- Verification notes:
  - Added `docs/ENTERPRISE_THREAT_MODEL.md` covering auth, tenant isolation, payments, documents, TMS/API integrations, admin/support tooling, browser security, secrets, and location privacy.
  - Tracked mitigations, residual risks, and evidence/follow-up references.

### ENT-1702 Add Security Headers And CSP

- Priority: `P1`
- Status: `[x]`
- Owner: Frontend/Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Browser security baseline is enterprise acceptable.
- Verification notes:
  - Added backend security-header middleware and frontend nginx security headers/CSP.
  - Added `docs/ENTERPRISE_SECURITY_HEADERS_AND_CSP.md`.
  - Verified `cargo test -p backend app::tests::enterprise_security_headers_are_defined`: 1 passed.
  - Verified `trunk build --release`.

### ENT-1703 Add Dependency And Secret Scanning

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Known vulnerable dependencies and leaked secrets block release.
- Verification notes:
  - CI runs cargo audit, npm audit, and `scripts/run_ci_security.ps1`.
  - Added `scripts/run_sensitive_output_scan.ps1` and wired it into the security lane.
  - Verified `scripts/run_ci_security.ps1 -SkipCargoAudit`: passed, npm audit reported 0 vulnerabilities.

### ENT-1704 Define Privacy And Data Request Workflow

- Priority: `P1`
- Status: `[x]`
- Owner: Legal/Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Customer privacy requests can be processed consistently.
- Verification notes:
  - Added `docs/ENTERPRISE_PRIVACY_DATA_REQUEST_WORKFLOW.md`.
  - Defined intake, identity/authority verification, export, deletion, correction, restriction, legal hold, timelines, and evidence requirements.
  - Covered location data, documents, chat/support notes, audit ledger, integration records, and payment/finance constraints.

### ENT-1705 Prepare Enterprise Security Packet

- Priority: `P2`
- Status: `[x]`
- Owner: Security/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Sales/support can answer security questionnaires without engineering fire drills.
- Verification notes:
  - Added `docs/ENTERPRISE_SECURITY_PACKET.md`.
  - Created a customer-questionnaire evidence index and standard answers for hosting, tenant isolation, auth/access control, encryption, PCI/payment scope, backups, incident response, vulnerability management, logging, audit, and subprocessors.
  - Documented known pre-launch gaps so external answers stay honest.

### ENT-1706 Define Encryption And Data Classification

- Priority: `P0`
- Status: `[x]`
- Owner: Security/Backend/Legal
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Sensitive data handling is documented and enforced.
  - Logs and support tools do not expose secrets, payment data, private documents, or unnecessary PII.
- Verification notes:
  - Added `docs/ENTERPRISE_DATA_CLASSIFICATION_AND_ENCRYPTION.md`.
  - Documented data classes, encryption baseline, masking, and redaction rules.
  - Wired sensitive-output scanning into the security lane.

### ENT-1706A Define Key Management, Rotation, And PCI Scope

- Priority: `P0`
- Status: `[x]`
- Owner: Security/Backend/DevOps/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Encryption keys and payment secrets have owners, rotation procedures, and audit evidence.
  - PCI/payment-data scope is documented and minimized before enterprise launch.
- Verification notes:
  - Added `docs/ENTERPRISE_KEY_MANAGEMENT_AND_PCI_SCOPE.md`.
  - Defined secret/key owners, rotation rules, emergency rotation, Stripe/PCI boundary, and disallowed raw payment data.
  - Hosted Stripe/smoke scripts now require env-provided credentials.

### ENT-1707 Enforce Secret File Hygiene

- Priority: `P0`
- Status: `[x]`
- Owner: DevOps/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Secret scanning blocks commits and CI merges.
  - Local secret file policy is documented and followed.
- Verification notes:
  - Expanded `.gitignore` for env files, IBM/COS exports, TLS/private keys, SSH private keys, and local secret folders.
  - Added `docs/ENTERPRISE_SECRET_FILE_HYGIENE.md`.
  - Added and verified `scripts/run_pre_commit.ps1`.

### ENT-1708 Add WAF, DDoS, And Bot Protection

- Priority: `P0`
- Status: `[~]`
- Owner: DevOps/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Public endpoints have layered protection beyond application-level rate limiting.
  - Security can respond to abuse without code changes.
- Verification notes:
  - Added `docs/ENTERPRISE_WAF_DDOS_BOT_PROTECTION.md`.
  - Defined public surfaces, required edge controls, baseline route rules, abuse response workflow, change-control requirements, and completion criteria.
  - Installed IBM CIS and DNS CLI plugins and inspected IBM account state on 2026-05-27.
  - Confirmed no IBM CIS service instance is currently configured and IBM DNS Services has no hosted zones, so public hostnames are not yet protected by IBM CIS WAF/bot controls.
  - Partial decision: edge-provider configuration, DNS routing, WAF/DDoS/bot/rate-limit rules, and block/challenge/rollback evidence are still required.

### ENT-1709 Add Vendor, Subprocessor, And Third-Party Risk Management

- Priority: `P1`
- Status: `[x]`
- Owner: Security/Legal/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise security questionnaires can be answered with a maintained vendor inventory.
  - New vendors cannot be introduced without review.
- Verification notes:
  - Added `docs/ENTERPRISE_VENDOR_SUBPROCESSOR_RISK.md`.
  - Inventoried production cloud, Stripe, SMTP, maps/geocoding, ELD/telematics, EDI/TMS, source control, monitoring/SIEM, support/help center, and analytics.
  - Defined approval workflow, review cadence, customer disclosure, and production enforcement rules.

### ENT-1710 Define Data Residency, DPA, And Regional Requirements

- Priority: `P1`
- Status: `[x]`
- Owner: Legal/Security/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Sales can answer where customer data lives and under what privacy terms.
  - Engineering knows which region boundaries must be enforced.
- Verification notes:
  - Added `docs/ENTERPRISE_DATA_RESIDENCY_DPA.md`.
  - Defined first-release US-region posture, contracted-region rules, DPA/privacy commitments, store-by-store residency requirements, privacy applicability checks, and engineering guardrails.

### ENT-1711 Add Penetration Testing And Vulnerability Disclosure

- Priority: `P1`
- Status: `[~]`
- Owner: Security/Engineering/Legal
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - High and critical penetration-test findings are remediated or formally risk-accepted before launch.
  - Security reports have a defined intake, triage, response, and customer evidence workflow.
- Verification notes:
  - Added `docs/ENTERPRISE_PENTEST_AND_VULNERABILITY_DISCLOSURE.md`.
  - Defined testing cadence, first enterprise test scope, severity SLAs, disclosure intake, and customer evidence package.
  - Added execution pack: ROE, vendor RFP, test account/data plan, findings tracker template, and customer evidence template.
  - Ran `scripts/verify_pentest_readiness.ps1 -TargetBaseUrl https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud` on 2026-05-27; required artifacts are present and live IBM target health check passed.
  - Partial decision: vendor scheduling, third-party test execution, retest evidence, and high/critical remediation or risk acceptance are still required.

### ENT-1712 Define SOC 2 Or ISO 27001 Readiness Program

- Priority: `P1`
- Status: `[x]`
- Owner: Security/Ops/Legal/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise procurement can see the compliance roadmap and current evidence posture.
  - Required operational controls are tracked as product/engineering work, not only policy documents.
- Verification notes:
  - Added `docs/ENTERPRISE_SOC2_ISO_READINESS.md`.
  - Defined SOC 2-style readiness posture, certification/deferral decision, control map, evidence owners, evidence repository rules, and procurement answer.

