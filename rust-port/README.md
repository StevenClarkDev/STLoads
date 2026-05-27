# STLoads Rust Port

This workspace is the starting point for the Rust + Leptos copy of the current Laravel application.

## Workspace Layout

- `crates/backend`: Axum-based HTTP API, webhooks, background entry points, and current deployable backend workload.
- `crates/db`: Schema inventory, migrations, and SQLx-facing persistence layer.
- `crates/domain`: Core business modules for auth, dispatch, marketplace, tracking, payments, TMS sync, and reference-data contracts.
- `crates/frontend-leptos`: Leptos UI layer for the user and admin portals.
- `crates/shared`: Shared DTOs, API responses, and cross-cutting types used by backend and frontend.

## Foundation Docs

- `docs/MASTER_PLAN.md`: migration phases and immediate checkpoints.
- `docs/BACKEND_BLUEPRINT.md`: backend domain split and blocker inventory.
- `docs/FRONTEND_BLUEPRINT.md`: page inventory and Leptos module plan.
- `docs/ENTERPRISE_LOADBOARD_ROADMAP.md`: enterprise-readiness roadmap for the Rust platform.
- `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`: production task tracker for following the roadmap to enterprise readiness.
- `docs/ENTERPRISE_WORK_BOARD.md`: execution board for `P0`/`P1` enterprise-readiness tasks.
- `docs/ENTERPRISE_READINESS_GATES.md`: measurable alpha, beta, pilot, production, and enterprise-ready gates.
- `docs/ENTERPRISE_OPERATING_MODEL.md`: recommended first enterprise-release market, role, and operating-model scope.
- `docs/ENTERPRISE_CUSTOMER_ONBOARDING.md`: repeatable enterprise tenant onboarding checklist and owner handoff.
- `docs/ENTERPRISE_CUSTOMER_SUCCESS_AND_TRAINING.md`: enterprise training, help-center, support handoff, adoption, and renewal playbook.
- `docs/ENTERPRISE_SLA_SUPPORT_TIERS.md`: enterprise support tiers, SLA targets, maintenance windows, and escalation commitments.
- `docs/ENTERPRISE_CUSTOMER_OFFBOARDING.md`: enterprise customer offboarding, data-return, retention, and access-shutdown process.
- `docs/ENTERPRISE_SUPPORT_CASE_MANAGEMENT.md`: enterprise support-case model, SLA case workflow, reporting, and customer feedback loop.
- `docs/ENTERPRISE_MIGRATION_RUNBOOK.md`: operator-controlled database migration command, rollback, and failed-migration procedure.
- `docs/ENTERPRISE_RELEASE_ENVIRONMENTS.md`: local, CI, staging, pilot, and production environment matrix plus runtime validation policy.
- `docs/ENTERPRISE_ROLLBACK_RUNBOOK.md`: backend, frontend, database, object-storage, and release rollback procedure.
- `docs/ENTERPRISE_FEATURE_FLAGS_AND_CHANGE_CONTROL.md`: feature-flag strategy, emergency kill switches, and production change approval checklist.
- `docs/ENTERPRISE_RELEASE_COMMUNICATIONS.md`: customer release notes, UAT, maintenance notices, known issues, and rollout feedback loop.
- `docs/ENTERPRISE_DATA_MIGRATION_CUTOVER_PLAN.md`: production data migration, reconciliation, freeze window, rollback point, and business validation plan.
- `docs/ENTERPRISE_BROWSER_SESSION_SECURITY.md`: browser bearer-auth, CSRF decision, cookie migration gate, CORS, and custom-domain session controls.
- `docs/ENTERPRISE_RATE_LIMITING_AND_LOCKOUT.md`: rate limiting, brute-force lockout, protected surfaces, and the distributed-store production gap.
- `docs/ENTERPRISE_MFA_AND_STEP_UP.md`: privileged MFA, recovery codes, and step-up requirements for high-risk actions.
- `docs/ENTERPRISE_ORGANIZATION_TENANT_MODEL.md`: organization tables, memberships, default-tenant migration, and tenant-scoped table ownership.
- `docs/ENTERPRISE_TENANT_ISOLATION.md`: current tenant isolation enforcement, tests, and remaining break-glass requirements.
- `docs/ENTERPRISE_PERMISSION_MATRIX.md`: platform-role and organization-role permission matrix plus route guard map.
- `docs/ENTERPRISE_TEST_LANES_AND_CI.md`: Phase 16 test lanes, CI jobs, pinned frontend build tooling, and smoke/performance verification policy.
- `docs/CANONICAL_STATUS_MODEL.md`: Rust-side status model and legacy PHP code mapping.
- `docs/SCHEMA_BASELINE.md`: draft SQL baseline inferred from Laravel write paths.
- `docs/POSTGRES_PIVOT.md`: PostgreSQL migration checklist for the Rust port.
- `docs/IBM_DEPLOYMENT_NOTES.md`: IBM-hosting assumptions and runtime constraints.
- `docs/IBM_CODE_ENGINE_DEPLOYMENT.md`: step-by-step beginner deployment guide for IBM Code Engine.

## Migration Approach

The current Laravel application remains the production source while we rebuild feature parity inside this workspace.

Recommended order:

1. Recover and normalize the real current production schema from MySQL.
2. Translate that schema into the target PostgreSQL shape for IBM deployment.
3. Port authentication, roles, and onboarding.
4. Port loads, load legs, documents, and dashboard queries.
5. Port offers, chat, tracking, and realtime flows.
6. Port Stripe escrow and payout flows.
7. Port STLOADS/TMS inbound, webhook, and reconciliation flows.
8. Cut over page-by-page from Blade to Leptos.

## Current Status

- The Rust workspace now targets PostgreSQL through SQLx.
- The heavy runtime SQL surface has been moved into PostgreSQL-compatible query syntax.
- The backend can be containerized and deployed to IBM Code Engine with the included `Dockerfile`.
- The current fastest IBM milestone is backend deployment first, then PostgreSQL smoke validation using the included seed and smoke scripts.

## IBM Starter Assets

- `Dockerfile`
- `.dockerignore`
- `.ceignore`
- `.env.ibm.example`
- `scripts/seed_postgres_smoke_data.sql`
- `scripts/smoke_test_backend.ps1`
