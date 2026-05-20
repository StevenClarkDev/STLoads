# STLoads Production Readiness Completion Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete STLoads as a production-grade, market-ready enterprise load board that works hand in hand with ATMP-OS while remaining its own product.

**Architecture:** ATMP-OS Dispatch remains the TMS and system of record. STLoads remains a standalone Rust/Leptos marketplace/load-board product with its own backend, frontend, middleware, database, deployment, security envelope, and operations surfaces. The GitHub STLoads UI/UX is locked as the visual contract, and all API/middleware/backend work must support that experience without redesigning it.

**Tech Stack:** Rust, Axum, SQLx, PostgreSQL, Leptos, IBM Code Engine, IBM Container Registry, object storage, Stripe Connect, WebSocket/SSE realtime, ATMP Dispatch API contract.

---

## Non-Negotiable Product Boundaries

- **STLoads UI/UX source of truth:** `https://github.com/StevenClarkDev/STLoads.git`
- **Approved visual commit:** `a258e74082ae147f12c17ab793d9fffc236174a7`
- **Clean working source:** `C:\New folder\STLoads-api-review`
- **ATMP system of record:** `C:\New folder\atmp-os`
- **Do not use:** `C:\New folder\atmp-os-core-rebuild`
- **Do not treat as visual authority:** `C:\New folder\atmp-os\STLoads`

STLoads must not become an ATMP screen. ATMP can launch or integrate with it, but STLoads remains its own board.

---

## Agent Work Lanes

### Lane A: Backend And Data Core

Owns database, domain models, API handlers, event state, and persistence.

Primary areas:

- `rust-port/crates/backend/src/routes`
- `rust-port/crates/domain/src`
- `rust-port/crates/db/src`
- `rust-port/crates/shared/src`
- `rust-port/crates/db/migrations`

### Lane B: Frontend And Visual Contract

Owns the Leptos UI, wiring real data into the developer's existing UX, removing placeholders, and preserving styling.

Primary areas:

- `rust-port/crates/frontend-leptos/src`
- `rust-port/crates/frontend-leptos/index.html`
- `rust-port/crates/frontend-leptos/assets`

### Lane C: Middleware, Security, Integrations, Deployment

Owns auth middleware, RBAC, rate limits, signed webhooks, observability, IBM deploy, smoke tests, and go-to-market readiness.

Primary areas:

- `rust-port/crates/backend/src/auth_session.rs`
- `rust-port/crates/backend/src/config.rs`
- `rust-port/crates/backend/src/app.rs`
- `rust-port/scripts`
- `rust-port/Dockerfile`
- `rust-port/Dockerfile.frontend`
- `rust-port/docs`

---

## Completion Rules

- Remove or check off a task only after backend, frontend, middleware, and tests for that task are complete.
- Commit after every completed task.
- Commit and push STLoads-owned work from `C:\New folder\STLoads-api-review` to `https://github.com/StevenClarkDev/STLoads.git`.
- Commit and push ATMP-owned work from `C:\New folder\atmp-os` to `https://github.com/sabertech-development/atmp-os.git` only when the task changes ATMP Dispatch, the ATMP launcher, or the ATMP side of the STLoads API contract.
- If a task touches both products, split the work into separate commits in the correct repository.
- Do not deploy to IBM until all Critical and Launch Gate tasks pass.
- Do not redesign the GitHub UI/UX.
- Do not leave demo loads, placeholder carriers, fake payments, fake compliance statuses, or static dashboards in production paths.
- Every production write path must be tenant-scoped, role-gated, idempotent where repeatable, audited, and observable.

---

## Critical Tasks

## Launch Gate

Do not call STLoads market-ready until every item below is true:

- [ ] GitHub developer UI/UX is preserved.
- [ ] No production-visible demo or placeholder data remains.
- [ ] ATMP publish/update/withdraw/close works end to end.
- [ ] STLoads outbound events reconcile back to ATMP.
- [ ] Tenant isolation tests pass.
- [ ] RBAC tests pass.
- [ ] Signed integration tests pass.
- [ ] Carrier search uses real data with real filters and pagination.
- [ ] Eligibility gates block noncompliant carriers.
- [ ] Offer/counter/tender/book-now flows work.
- [ ] Booking concurrency lock prevents double booking.
- [ ] Documents are versioned, reviewed, and synced.
- [ ] Tracking and exception events sync to ATMP.
- [ ] Stripe webhook replay is idempotent.
- [ ] Admin reconciliation can replay or dead-letter failed events.
- [ ] Health/readiness checks are live.
- [ ] Backend image builds.
- [ ] Frontend image builds.
- [ ] IBM staging smoke passes.
- [ ] Production runbook exists.
- [ ] Security overview exists.
- [ ] Go-to-market overview exists.

---

## Recommended Execution Order

1. P7 Tenant/RBAC/session hardening.
2. P8 Carrier search and alerts.
3. Final launch-gate verification and partner signoff.

## Definition Of Done

STLoads is done when a clean tenant can receive a board-ready ATMP load, expose it only to eligible carriers, accept an offer or booking, prevent duplicate awards, track execution, process documents, handle payment events, reconcile every important event back to ATMP, and provide operators with enough admin tooling to recover from failures without database access.
