# Master Plan

This is the working blueprint for the Rust + Leptos STLoads platform.

For the current enterprise-readiness execution tracker, use `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`.
For the higher-level roadmap behind that tracker, use `docs/ENTERPRISE_LOADBOARD_ROADMAP.md`.
For the `P0`/`P1` execution board, use `docs/ENTERPRISE_WORK_BOARD.md`.
For measurable alpha, beta, pilot, production, and enterprise-ready gates, use `docs/ENTERPRISE_READINESS_GATES.md`.
For the recommended first-release operating model and pending approval items, use `docs/ENTERPRISE_OPERATING_MODEL.md`.
For repeatable enterprise tenant onboarding, use `docs/ENTERPRISE_CUSTOMER_ONBOARDING.md`.
For enterprise training, help-center, adoption, renewal, and support handoff, use `docs/ENTERPRISE_CUSTOMER_SUCCESS_AND_TRAINING.md`.
For support tiers, SLA targets, maintenance windows, and escalation commitments, use `docs/ENTERPRISE_SLA_SUPPORT_TIERS.md`.
For enterprise customer offboarding, data return, retention, and access shutdown, use `docs/ENTERPRISE_CUSTOMER_OFFBOARDING.md`.
For enterprise support case management, SLA case workflow, reporting, and customer feedback, use `docs/ENTERPRISE_SUPPORT_CASE_MANAGEMENT.md`.
For operator-controlled database migration steps, rollback, and failed-migration handling, use `docs/ENTERPRISE_MIGRATION_RUNBOOK.md`.
For local, CI, staging, pilot, and production environment definitions, use `docs/ENTERPRISE_RELEASE_ENVIRONMENTS.md`.
For backend, frontend, database, object-storage, and release rollback procedure, use `docs/ENTERPRISE_ROLLBACK_RUNBOOK.md`.
For feature flags, emergency kill switches, and production change approval, use `docs/ENTERPRISE_FEATURE_FLAGS_AND_CHANGE_CONTROL.md`.
For customer release notes, UAT, maintenance notices, known issues, and rollout feedback, use `docs/ENTERPRISE_RELEASE_COMMUNICATIONS.md`.
For production data migration, reconciliation, freeze window, rollback point, and business validation, use `docs/ENTERPRISE_DATA_MIGRATION_CUTOVER_PLAN.md`.
For browser bearer-auth, CSRF decision, cookie migration gate, CORS, and custom-domain session controls, use `docs/ENTERPRISE_BROWSER_SESSION_SECURITY.md`.
For rate limiting, brute-force lockout, protected surfaces, and the distributed-store production gap, use `docs/ENTERPRISE_RATE_LIMITING_AND_LOCKOUT.md`.
For privileged MFA, recovery codes, and step-up requirements for high-risk actions, use `docs/ENTERPRISE_MFA_AND_STEP_UP.md`.
For organization tables, memberships, default-tenant migration, and tenant-scoped table ownership, use `docs/ENTERPRISE_ORGANIZATION_TENANT_MODEL.md`.
For current tenant isolation enforcement, cross-tenant tests, and break-glass requirements, use `docs/ENTERPRISE_TENANT_ISOLATION.md`.
For platform-role, organization-role, and route-guard permission mapping, use `docs/ENTERPRISE_PERMISSION_MATRIX.md`.
For test lanes, CI jobs, pinned frontend tooling, Docker gates, and smoke/performance policy, use `docs/ENTERPRISE_TEST_LANES_AND_CI.md`.

## What Exists Now

- The Rust application lives under `rust-port/`.
- The old Laravel/PHP scaffold has been removed from the repository root. Historical Laravel references in docs are migration evidence only.
- Initial workspace scaffold exists for:
  - `crates/backend`
  - `crates/db`
  - `crates/domain`
  - `crates/frontend-leptos`
  - `crates/shared`
- Phase 0 artifacts now exist for:
  - schema inventory in `crates/db`
  - canonical lifecycle notes in `docs/CANONICAL_STATUS_MODEL.md`
  - grouped backend route skeletons in `crates/backend/src/routes`

## Migration Rule

The Rust workspace is the active application source. Historical PHP/Laravel notes should be used only for audit trail, parity history, and legacy data reconciliation.

## Deployment Constraint

- The Rust port is being shaped for IBM-hosted server deployment.
- The target database is PostgreSQL on IBM-hosted infrastructure.
- Runtime behavior must stay environment-driven and reverse-proxy friendly.
- Avoid local-disk assumptions for durable application data.
- Deployment notes live in `rust-port/docs/IBM_DEPLOYMENT_NOTES.md`.
- Database dialect migration notes live in `rust-port/docs/POSTGRES_PIVOT.md`.

## Phase Plan

### Phase 0: Recovery

- recover the real current production schema from the existing MySQL system
- recover the secondary logs schema
- define canonical enums and state machines
- define the target PostgreSQL schema for IBM deployment
- document route parity targets

### Phase 1: Foundations

- auth, sessions, tokens, and RBAC
- shared DTOs and error model
- backend service skeleton
- frontend shells
- begin SQLx dialect migration from MySQL to PostgreSQL

### Phase 2: Dispatch Core

- loads
- load legs
- locations
- documents
- histories
- dashboard queries
- PostgreSQL-ready migrations and repositories for dispatch tables

### Phase 3: Marketplace and Execution

- offers and booking
- conversations and messages
- realtime chat
- tracking, leg events, and leg docs

### Phase 4: Payments and TMS

- Stripe Connect onboarding
- escrow funding and release
- Stripe webhooks
- TMS inbound, queue, requeue, withdraw, close
- reconciliation and sync errors

### Phase 5: Cutover

- migrate user-facing pages from Blade to Leptos
- migrate admin pages
- switch operational traffic feature-by-feature
- cut over to IBM-hosted PostgreSQL-backed Rust services

## Parallel Ownership

- Frontend track
  - shells, components, page modules, interactive islands
  - details in `rust-port/docs/FRONTEND_BLUEPRINT.md`
- Backend track
  - schema, services, APIs, jobs, integrations, state machines
  - details in `rust-port/docs/BACKEND_BLUEPRINT.md`

## Immediate Next Actions

1. Produce an authoritative schema inventory from the live MySQL databases.
2. Translate that inventory into a canonical PostgreSQL target schema for IBM deployment.
3. Turn the current hard-coded statuses into explicit Rust enums and transition rules.
4. Port the SQLx layer from MySQL-specific types, placeholders, and functions to PostgreSQL.
5. Expand `backend` into route groups that mirror:
   - auth
   - dispatch
   - marketplace
   - execution
   - payments
   - tms
6. Expand `frontend-leptos` into shell and page modules for auth, dashboard, and loads.

## Success Criteria For The Next Checkpoint

- Rust workspace structure is stable.
- Current production schema is documented.
- Target PostgreSQL schema is documented.
- Canonical status model is written down.
- Auth shell, dashboard shell, and load shell routes exist in Leptos.
- Backend health endpoint evolves into grouped Axum routers.
- The SQLx migration path away from MySQL is explicit and tracked.
