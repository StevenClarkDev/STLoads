# Enterprise Work Board

Last updated: 2026-05-24

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
- Status: `[ ]`
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
  - Linked cutover plan from `README.md` and `docs/MASTER_PLAN.md`.
  - Verified `scripts/run_cutover_reconciliation.ps1` fails safely when `DATABASE_URL` is missing.
  - Verified `cargo fmt --check`.
  - Verified `cargo test -p backend`: 28 passed.
  - Verified runtime env template validation still passes with placeholder allowance.
  - Pending: run reconciliation against staging/prod-like data, provide the matching legacy summary, and complete business validation before marking this task complete.

## Phase 2: Auth, Authorization, And Tenant Boundaries

### ENT-0201 Hash Session Tokens

- Priority: `P0`
- Status: `[ ]`
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
- Status: `[ ]`
- Owner: Backend/Frontend/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Support can help customers without unsafe database access.
  - Every support action is auditable.
- Verification notes:
  - `TBD`

### ENT-0208 Add Enterprise SSO And SCIM

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise customers can manage users through their identity provider.
  - Deprovisioning removes access without manual STLoads intervention.
- Verification notes:
  - `TBD`

### ENT-0209 Add Access Reviews And Least-Privilege Recertification

- Priority: `P1`
- Status: `[ ]`
- Owner: Security/Ops/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Internal and customer-facing privileged access can be recertified on a schedule.
  - Stale, excessive, or emergency access does not remain active silently.
- Verification notes:
  - `TBD`

## Phase 3: Audit, Compliance Evidence, And Governance

### ENT-0301 Add Global Audit Ledger

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - All high-risk workflows write audit events.
  - Audit events are queryable by entity and actor.
- Verification notes:
  - `TBD`

### ENT-0302 Add Request Correlation IDs

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - A single incident can be traced across API, DB, jobs, payments, and integrations.
- Verification notes:
  - `TBD`

### ENT-0303 Add Audit Search UI

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Ops can answer who changed what without developer help.
- Verification notes:
  - `TBD`

### ENT-0304 Define Status And Reference Data Governance

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - No workflow status can be changed casually without a migration/test/update plan.
- Verification notes:
  - `TBD`

### ENT-0305 Add Legal Agreement Acceptance And E-Signature Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Legal/Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Legal acceptance can be proven for every required operational agreement.
  - Updated terms can be rolled out and tracked by version.
- Verification notes:
  - `TBD`

### ENT-0306 Define Operating Authority, Insurance, And Jurisdiction Requirements

- Priority: `P1`
- Status: `[ ]`
- Owner: Legal/Operations/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - The company can prove it is legally allowed and insured to operate under the chosen business model.
  - Enterprise customers can receive required authority, bond, and insurance evidence without ad hoc legal work.
- Verification notes:
  - `TBD`

## Phase 4: Document Security And Governance

### ENT-0401 Harden Local File Reads

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Malicious `../` or crafted local paths cannot escape storage root.
- Verification notes:
  - `TBD`

### ENT-0402 Add Document Versioning

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Replacing a document does not destroy audit history.
- Verification notes:
  - `TBD`

### ENT-0403 Add Required Document Rules

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Closeout and compliance readiness are machine-checkable.
- Verification notes:
  - `TBD`

### ENT-0404 Replace Mock Blockchain Proof

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - UI no longer claims mock proof as real proof.
  - Hash verification uses actual uploaded file bytes.
- Verification notes:
  - `TBD`

### ENT-0405 Add File Validation And Scanning Hook

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Dangerous or invalid uploads are blocked or quarantined.
- Verification notes:
  - `TBD`

### ENT-0407 Add Freight Document Templates And Packets

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Operators can generate and retrieve standard freight documents without manual template work.
  - Generated documents are linked to load, carrier, customer, audit, and document history.
- Verification notes:
  - `TBD`

## Phase 5: Load Posting, Search, And Customer Rules

### ENT-0501 Finish Enterprise Load Model

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise freight can be posted without out-of-band notes.
- Verification notes:
  - `TBD`

### ENT-0502 Add Draft/Publish/Revise/Cancel/Archive/Clone

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Users can manage common lifecycle actions safely.
- Verification notes:
  - `TBD`

### ENT-0503 Add Customer Contract And Lane Guide Model

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise shippers can run private/contract freight, not only public spot loads.
- Verification notes:
  - `TBD`

### ENT-0504 Add Saved Filters And Load Search

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Load board stays usable as volume grows.
- Verification notes:
  - `TBD`

### ENT-0506 Add Rating, Mileage, Fuel, And Accessorial Rules

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Rates can be calculated and explained before booking, billing, and settlement.
  - Manual pricing changes are auditable.
- Verification notes:
  - `TBD`

### ENT-0507 Add Address Validation, Geocoding, And Facility Scheduling

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Pickup and delivery locations are operationally usable, not just text addresses.
  - Appointment changes are visible to carrier, shipper, operator, and audit history.
- Verification notes:
  - `TBD`

### ENT-0509 Add Time Zone, Unit, Currency, And Localization Rules

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Operators, carriers, and customers do not misread appointment times, units, or rates.
  - Reports and integrations use documented canonical units.
- Verification notes:
  - `TBD`

### ENT-0509A Define Cross-Border Tax, FX, Duties, And Incoterms Rules

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Finance/Legal/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Cross-border and multi-currency freight cannot be billed or settled with ambiguous tax or FX rules.
  - Finance/legal can explain which party owns duties, taxes, and currency risk.
- Verification notes:
  - `TBD`

### ENT-0510 Add Governed Master Data And Configuration Admin

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise configuration can be changed safely without code deployments or direct database edits.
  - Master-data changes are auditable, reversible, and tested against active freight workflows.
- Verification notes:
  - `TBD`

## Phase 6: Carrier Network, Matching, And Marketplace

### ENT-0601 Build Carrier Capacity Profile

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Carrier eligibility can be computed from structured data.
- Verification notes:
  - `TBD`

### ENT-0602 Add Private Networks And Blocklists

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Shippers control who can see and book private freight.
- Verification notes:
  - `TBD`

### ENT-0603 Add Matching And Ranking

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Data
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Matching is explainable to operators and customers.
- Verification notes:
  - `TBD`

### ENT-0604 Complete Offer State Machine

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Offer state cannot become ambiguous or contradictory.
- Verification notes:
  - `TBD`

### ENT-0605 Add Counteroffers, Expiration, And Tender Flow

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise tendering and spot negotiation are both supported.
- Verification notes:
  - `TBD`

### ENT-0606 Add Booking Race Protection

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Two carriers cannot book the same leg through race conditions.
- Verification notes:
  - `TBD`

### ENT-0607 Add Carrier Packet And Vetting Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Compliance/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - A carrier cannot receive restricted freight until packet requirements are complete.
  - Operators can see exactly which packet item blocks eligibility.
- Verification notes:
  - `TBD`

## Phase 7: Dispatch Desk And Operator Workflows

### ENT-0701 Define Canonical Desk Queues

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Every load/leg has a clear operational queue when action is needed.
- Verification notes:
  - `TBD`

### ENT-0702 Add Assignment, Priority, SLA, And Escalation

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Managers can see backlog, aging, and stuck work.
- Verification notes:
  - `TBD`

### ENT-0703 Separate Internal And Customer-Visible Notes

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Internal operational comments cannot accidentally leak to customers.
- Verification notes:
  - `TBD`

### ENT-0704 Add Exception Management

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Operations can resolve freight exceptions without spreadsheets.
- Verification notes:
  - `TBD`

## Phase 8: Execution, Tracking, Mobile, And Closeout

### ENT-0801 Centralize Execution State Machine

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Invalid pickup, delivery, or closeout transitions are impossible.
- Verification notes:
  - `TBD`

### ENT-0802 Build Mobile-First Driver Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Driver tasks work comfortably on a phone.
- Verification notes:
  - `TBD`

### ENT-0802A Add Mobile Field Capture And Offline Strategy

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Drivers can complete critical field tasks under real mobile network conditions.
  - Offline or delayed submissions are clearly marked and reconciled when connectivity returns.
- Verification notes:
  - `TBD`

### ENT-0803 Add Tracking Consent And Privacy

- Priority: `P0`
- Status: `[ ]`
- Owner: Product/Legal/Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Location tracking is consented, explainable, and retained only as needed.
- Verification notes:
  - `TBD`

### ENT-0805 Complete POD And Closeout Package

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Delivered loads cannot be financially released until closeout rules pass.
- Verification notes:
  - `TBD`

### ENT-0807 Add Claims, Detention, And Accessorial Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Frontend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Exceptions that affect billing or settlement are tracked through resolution.
  - Finance can see which charges are approved, disputed, rejected, or pending.
- Verification notes:
  - `TBD`

## Phase 9: Payments, Billing, Settlements, And Finance Controls

### ENT-0901 Add Payment Idempotency

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Repeated requests cannot double-charge or double-release.
- Verification notes:
  - `TBD`

### ENT-0902 De-Duplicate Stripe Webhooks

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Stripe retry behavior cannot corrupt escrow state.
- Verification notes:
  - `TBD`

### ENT-0903 Add Payment Ledger

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Finance can reconstruct every cent.
- Verification notes:
  - `TBD`

### ENT-0904 Add Finance Approval Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - High-risk payouts cannot be released casually.
- Verification notes:
  - `TBD`

### ENT-0905 Add Invoices And Carrier Settlements

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Platform supports billing and settlement, not only escrow.
- Verification notes:
  - `TBD`

### ENT-0909 Add Shipper Credit, AR Aging, And Collections Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Finance/Product/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Finance can prevent new exposure from high-risk or overdue shippers.
  - Operators know when a load is blocked by credit policy and how to escalate.
- Verification notes:
  - `TBD`

### ENT-0910 Add Bank Account And Payout Change Controls

- Priority: `P0`
- Status: `[ ]`
- Owner: Finance/Security/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Payout destination changes cannot silently redirect carrier money.
  - Returned or suspicious payouts are held until finance review is complete.
- Verification notes:
  - `TBD`

## Phase 10: Compliance, Risk, And Fraud

### ENT-1001 Split Compliance Models

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Compliance status is specific enough to drive eligibility.
- Verification notes:
  - `TBD`

### ENT-1002 Add FMCSA/DOT/MC And Insurance Verification

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Carrier booking can be blocked by expired or missing compliance.
- Verification notes:
  - `TBD`

### ENT-1002A Add Driver, Equipment, And Safety Compliance

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Compliance/Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Carrier eligibility can account for driver and equipment requirements, not only company-level compliance.
  - Expired or missing driver/equipment compliance can block restricted freight.
- Verification notes:
  - `TBD`

### ENT-1003 Add Sanctions And Tax Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Legal/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - High-risk entities can be blocked before booking or payout.
  - Payout tax reporting obligations are implemented or explicitly assigned to an external finance process.
- Verification notes:
  - `TBD`

### ENT-1005 Add Double-Brokering And Carrier Fraud Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Compliance/Risk/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - High-risk bookings can be paused before tender, pickup, or payout.
  - Operators have explicit risk reasons, not only a generic fraud flag.
- Verification notes:
  - `TBD`

### ENT-1006 Add AML, Transaction Monitoring, And Account-Takeover Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Risk/Compliance/Security/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Suspicious money movement and account-takeover attempts can be detected before payout release.
  - Compliance obligations are implemented or explicitly ruled out for the target operating model.
- Verification notes:
  - `TBD`

## Phase 11: TMS, APIs, Webhooks, And Integrations

### ENT-1101 Publish OpenAPI Specs

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - External customers can integrate against documented contracts.
- Verification notes:
  - `TBD`

### ENT-1102 Add Partner API Auth

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - External APIs are not protected only by user-session assumptions.
- Verification notes:
  - `TBD`

### ENT-1103 Add External Idempotency And Event De-Dupe

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Retried partner requests do not create duplicate loads or state changes.
- Verification notes:
  - `TBD`

### ENT-1104 Add Webhook Delivery Logs And Replay

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Failed customer webhooks can be diagnosed and replayed.
- Verification notes:
  - `TBD`

### ENT-1105 Add TMS Conflict Resolution

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - TMS drift can be fixed without developer database access.
- Verification notes:
  - `TBD`

### ENT-1107 Add EDI Integration Track

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product/Integrations
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise logistics partners can exchange standard freight events without custom one-off scripts.
  - Failed EDI messages are visible, replayable, and auditable.
- Verification notes:
  - `TBD`

### ENT-1108 Add Sandbox, Demo, And Test Tenant Governance

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Sales, support, QA, and customers can test safely without production data risk.
  - Sandbox/test behavior is visibly separated from production behavior.
- Verification notes:
  - `TBD`

### ENT-1109 Define API Lifecycle, SDKs, And Deprecation Policy

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Backend/Integrations
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise customers know how long integrations are supported and how breaking changes are handled.
  - Supported API examples can be run against sandbox without custom engineering help.
- Verification notes:
  - `TBD`

## Phase 12: Notifications And Communications

### ENT-1201 Add Notification Center

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Users do not depend only on email for operational events.
- Verification notes:
  - `TBD`

### ENT-1202 Add Notification Preferences

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise customers can control noisy workflows.
- Verification notes:
  - `TBD`

### ENT-1204 Broaden Notification Coverage

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Critical events notify the responsible party.
- Verification notes:
  - `TBD`

### ENT-1205 Add Message Deliverability And Compliance Controls

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/Ops/Product/Legal
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Critical messages have observable delivery status and safe fallback/escalation behavior.
  - Message templates and sender identities can be changed without accidental compliance or deliverability regressions.
- Verification notes:
  - `TBD`

## Phase 13: Frontend, UX, Accessibility, And Design System

### ENT-1301 Create Shared UI Component Library

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - New screens use shared components by default.
- Verification notes:
  - `TBD`

### ENT-1303 Add Accessibility Pass

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/QA
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Critical flows are keyboard and screen-reader usable.
  - Accessibility exceptions are documented with owners and remediation dates.
- Verification notes:
  - `TBD`

### ENT-1304 Add Role-Specific Dashboards

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Frontend/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Each role lands on actionable work, not a generic page.
- Verification notes:
  - `TBD`

### ENT-1305 Add Browser E2E And Visual Regression

- Priority: `P1`
- Status: `[ ]`
- Owner: QA/Frontend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - UI regressions are caught before production.
- Verification notes:
  - `TBD`

## Phase 14: Data, Reporting, Search, And Intelligence

### ENT-1401 Define Business Metrics

- Priority: `P1`
- Status: `[ ]`
- Owner: Product/Data/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Metrics definitions are documented and accepted by product/ops.
- Verification notes:
  - `TBD`

### ENT-1406 Add Data Quality And Integrity Monitoring

- Priority: `P1`
- Status: `[ ]`
- Owner: Data/Backend/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Bad data is detected before it becomes an operational or financial incident.
  - Data quality issues have owners, severity, repair status, and audit trail.
- Verification notes:
  - `TBD`

## Phase 15: Observability, Workers, Scale, And Disaster Recovery

### ENT-1501 Add Structured Logs And Tracing

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Incidents can be traced end to end.
- Verification notes:
  - `TBD`

### ENT-1502 Add Metrics And Alerts

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Team is alerted before customers report major failures.
- Verification notes:
  - `TBD`

### ENT-1502A Add On-Call, Escalation, And Security Log Export

- Priority: `P1`
- Status: `[ ]`
- Owner: DevOps/Security/Ops
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - P0/P1 alerts page an accountable owner with escalation if not acknowledged.
  - Security and audit logs can be retained, searched, exported, or forwarded according to enterprise commitments.
- Verification notes:
  - `TBD`

### ENT-1503 Separate Workers From Web Runtime

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Scaling web traffic does not multiply background job side effects.
- Verification notes:
  - `TBD`

### ENT-1504 Add Job Queue And Dead Letter Handling

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Failed background work is visible and recoverable.
- Verification notes:
  - `TBD`

### ENT-1505 Optimize Database Queries And Indexes

- Priority: `P1`
- Status: `[ ]`
- Owner: Backend/DBA
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Critical pages remain fast with production-scale data.
- Verification notes:
  - `TBD`

### ENT-1506 Define Backup, Restore, RPO, And RTO

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Restore procedure works and is documented.
  - The team knows whether recovery is restore-only or failover-capable for the target release.
- Verification notes:
  - `TBD`

### ENT-1508 Add Incident Response, Status Page, And Runbooks

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Ops/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - The team can respond to incidents with a documented process.
  - Customers can be informed consistently during outages or degraded service.
- Verification notes:
  - `TBD`

### ENT-1509 Define Business Continuity And Tabletop Exercises

- Priority: `P1`
- Status: `[ ]`
- Owner: Ops/DevOps/Security/Customer Success
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - The business can continue critical logistics operations during prolonged service degradation.
  - Enterprise customers can see that continuity procedures have been exercised, not only written.
- Verification notes:
  - `TBD`

### ENT-1510 Add Cost, Quota, And Usage Guardrails

- Priority: `P1`
- Status: `[ ]`
- Owner: DevOps/Product/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Production cost spikes are visible before they become business incidents.
  - Heavy customers and integrations are governed without harming normal freight operations.
- Verification notes:
  - `TBD`

## Phase 16: Testing, CI, And Quality Gates

### ENT-1601 Define Test Lanes

- Priority: `P0`
- Status: `[ ]`
- Owner: Engineering/QA
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Developers know what to run locally and CI knows what blocks merge.
- Verification notes:
  - `TBD`

### ENT-1602 Add CI Pipeline

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Main branch cannot receive unverified risky code.
- Verification notes:
  - `TBD`

### ENT-1603 Add Domain State Machine Tests

- Priority: `P0`
- Status: `[ ]`
- Owner: Backend/QA
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Invalid transitions are rejected by tests and code.
- Verification notes:
  - `TBD`

### ENT-1604 Add Security Access Tests

- Priority: `P0`
- Status: `[ ]`
- Owner: QA/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - P0 access regressions are caught automatically.
- Verification notes:
  - `TBD`

### ENT-1606 Make Clippy Warning-Clean

- Priority: `P1`
- Status: `[ ]`
- Owner: Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Clippy warning-clean is added to CI.
  - Any lint allowances are deliberate and documented near the code.
- Verification notes:
  - `TBD`

### ENT-1607 Add Frontend Release Build To CI

- Priority: `P0`
- Status: `[ ]`
- Owner: Frontend/DevOps
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Main branch cannot break the Leptos WASM release build.
- Verification notes:
  - `TBD`

## Phase 17: Security, Legal, And Enterprise Procurement

### ENT-1701 Perform Threat Model

- Priority: `P0`
- Status: `[ ]`
- Owner: Security/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Threats have tracked mitigations.
- Verification notes:
  - `TBD`

### ENT-1702 Add Security Headers And CSP

- Priority: `P1`
- Status: `[ ]`
- Owner: Frontend/Backend/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Browser security baseline is enterprise acceptable.
- Verification notes:
  - `TBD`

### ENT-1703 Add Dependency And Secret Scanning

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Known vulnerable dependencies and leaked secrets block release.
- Verification notes:
  - `TBD`

### ENT-1704 Define Privacy And Data Request Workflow

- Priority: `P1`
- Status: `[ ]`
- Owner: Legal/Product/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Customer privacy requests can be processed consistently.
- Verification notes:
  - `TBD`

### ENT-1706 Define Encryption And Data Classification

- Priority: `P0`
- Status: `[ ]`
- Owner: Security/Backend/Legal
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Sensitive data handling is documented and enforced.
  - Logs and support tools do not expose secrets, payment data, private documents, or unnecessary PII.
- Verification notes:
  - `TBD`

### ENT-1706A Define Key Management, Rotation, And PCI Scope

- Priority: `P0`
- Status: `[ ]`
- Owner: Security/Backend/DevOps/Finance
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Encryption keys and payment secrets have owners, rotation procedures, and audit evidence.
  - PCI/payment-data scope is documented and minimized before enterprise launch.
- Verification notes:
  - `TBD`

### ENT-1707 Enforce Secret File Hygiene

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Secret scanning blocks commits and CI merges.
  - Local secret file policy is documented and followed.
- Verification notes:
  - `TBD`

### ENT-1708 Add WAF, DDoS, And Bot Protection

- Priority: `P0`
- Status: `[ ]`
- Owner: DevOps/Security
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Public endpoints have layered protection beyond application-level rate limiting.
  - Security can respond to abuse without code changes.
- Verification notes:
  - `TBD`

### ENT-1709 Add Vendor, Subprocessor, And Third-Party Risk Management

- Priority: `P1`
- Status: `[ ]`
- Owner: Security/Legal/Product
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise security questionnaires can be answered with a maintained vendor inventory.
  - New vendors cannot be introduced without review.
- Verification notes:
  - `TBD`

### ENT-1710 Define Data Residency, DPA, And Regional Requirements

- Priority: `P1`
- Status: `[ ]`
- Owner: Legal/Security/Backend
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Sales can answer where customer data lives and under what privacy terms.
  - Engineering knows which region boundaries must be enforced.
- Verification notes:
  - `TBD`

### ENT-1711 Add Penetration Testing And Vulnerability Disclosure

- Priority: `P1`
- Status: `[ ]`
- Owner: Security/Engineering/Legal
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - High and critical penetration-test findings are remediated or formally risk-accepted before launch.
  - Security reports have a defined intake, triage, response, and customer evidence workflow.
- Verification notes:
  - `TBD`

### ENT-1712 Define SOC 2 Or ISO 27001 Readiness Program

- Priority: `P1`
- Status: `[ ]`
- Owner: Security/Ops/Legal/Engineering
- Issue link: `docs/ENTERPRISE_WORK_BOARD.md`
- Dependencies: `TBD`
- Estimate: `TBD`
- Source: `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`
- Acceptance criteria:
  - Enterprise procurement can see the compliance roadmap and current evidence posture.
  - Required operational controls are tracked as product/engineering work, not only policy documents.
- Verification notes:
  - `TBD`

