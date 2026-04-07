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
