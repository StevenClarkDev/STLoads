# Master Plan

This is the working blueprint for the Rust + Leptos copy of STLoads.

## What Exists Now

- The current production system remains the Laravel application at the repo root.
- The Rust port lives under `rust-port/`.
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

The PHP system stays authoritative until each feature area is rebuilt and validated inside `rust-port/`.

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
