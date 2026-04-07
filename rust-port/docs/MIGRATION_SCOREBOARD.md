# Migration Scoreboard

Last updated: 2026-04-07

This file is the living migration tracker for the Laravel-to-Rust/Leptos port.
It should be updated after every major implementation milestone so the repo itself shows what is done, what is partial, and what still blocks full PHP retirement.

## Status Key

- `done`: migrated into Rust/Leptos and materially usable
- `partial`: real Rust work exists, but not yet full parity
- `not_started`: still effectively Laravel-only
- `blocked`: depends on another unfinished migration area

## Overall Snapshot

- Backend/domain migration: `partial`
- Frontend/Leptos migration: `partial`
- IBM deployment readiness for backend: `partial`
- Full Laravel retirement readiness: `partial`

## Feature Scoreboard

| Area | Status | Notes |
| --- | --- | --- |
| PostgreSQL target and SQLx runtime surface | `done` | Core migrations and heavy runtime SQL now compile and verify on PostgreSQL syntax. |
| Axum backend shell and route groups | `done` | Auth, dispatch, marketplace, payments, TMS, admin, master-data, realtime route groups exist. |
| Session auth, tokens, RBAC, and route scoping | `partial` | Login/session/token flows work; full signup/onboarding parity is not complete yet. |
| Load board read surface | `partial` | Real auth-scoped load board exists with booking integration, but full load-management parity is still missing. |
| Chat and offers | `partial` | Real chat, offer review, read receipts, presence, and scoped realtime exist; still not full Blade parity. |
| Payments and escrow lifecycle | `partial` | Fund/hold/release and Stripe webhook routes exist; production-grade Stripe integration is still incomplete. |
| STLOADS/TMS lifecycle and ops | `partial` | Push/queue/requeue/withdraw/close, status webhooks, and admin ops views exist; still needs deeper production parity. |
| Admin STLOADS reconciliation | `partial` | Real Rust screens and DB-backed data exist; broader admin surface is still incomplete. |
| Master-data admin visibility | `partial` | Rust now has a DB-backed admin master-data catalog page with first-write tooling for key lookup datasets, but full parity is still missing. |
| Master-data CRUD | `partial` | Rust now supports first-write workflows for locations, load types, equipments, and commodity types; countries, cities, deletes, and richer edit UX are still pending. |
| Load create/edit workflow | `partial` | Rust now has a first-pass `/dispatch/load-builder` + `/dispatch/loads` flow and Leptos `/loads/new` form for core multi-leg creation, but edit mode, documents, and address autocomplete are still pending. |
| Load detail/profile/documents | `partial` | Core data models exist, but the Blade detail and document experience is not fully ported. |
| Tracking/execution UI | `not_started` | Execution contracts exist, but live tracking UI and execution flows are not yet migrated. |
| Onboarding and registration flows | `not_started` | Login exists, but multi-role registration, OTP, reset, and onboarding parity are still Laravel-only. |
| Admin CRUD for users and roles | `not_started` | Admin overview exists, but CRUD screens and approval workflows are not yet ported. |
| Dispatch desk pages | `not_started` | Quote, tender, facility, collections, and closeout desk pages are still Laravel-only. |
| Email templates and outbound mail workflows | `not_started` | Laravel Blade mail views still own these flows. |
| File/document storage abstraction for IBM | `not_started` | Needs object storage-backed implementation before full cutover. |
| Frontend deployment strategy on IBM | `partial` | Backend deployment path is prepared; final Leptos hosting/cutover strategy is still open. |
| Smoke-test tooling for IBM/PostgreSQL | `done` | Seed SQL, smoke script, and IBM env template exist in `rust-port/scripts` and `rust-port/.env.ibm.example`. |
| IBM Code Engine backend deployment assets | `done` | Dockerfile, ignore files, and deployment guide now exist. |

## Legacy Surface Still To Replace

- Laravel routes: roughly 184 lines across `routes/web.php` and `routes/api.php`
- PHP controllers: 25
- PHP models: 31
- Blade views: 76

## Current Rust Surface

- Backend route modules: 10
- Leptos page modules: 12
- Rust backend is already meaningful, but frontend parity is still much smaller than the Laravel view surface.

## Current Remaining High-Value Work

1. Finish auth/onboarding parity.
2. Deepen load detail/profile/document workflows now that first-pass load creation exists.
3. Port tracking/execution pages and map/browser integrations.
4. Port remaining master-data CRUD depth and admin user/role CRUD.
5. Port dispatch desk workflows.
6. Add object storage-backed document handling for IBM.
7. Finalize frontend hosting/cutover on IBM.
8. Build broader acceptance/integration coverage before retiring Laravel.

## Major Updates

### 2026-04-07 - Master Data Admin Read Surface

- Added this scoreboard as the living migration tracker.
- Added a DB-backed Rust master-data screen contract and admin page.
- Added backend `GET /master-data/screen` with admin permission checks.
- Added Leptos `/admin/master-data` and linked it from the admin shell.
- Updated the migration score so master-data admin visibility is now `partial` instead of effectively missing.

### 2026-04-07 - Load Builder First Pass

- Added shared Rust contracts for load-builder screen data and create-load actions.
- Added backend GET /dispatch/load-builder and POST /dispatch/loads for auth-scoped load creation backed by PostgreSQL.
- Added SQLx create-load persistence with generated RUST-LD-###### load numbers, multi-leg inserts, and load history writes.
- Added Leptos /loads/new with core load fields, dynamic leg rows, shared auth-session gating, and save flow wired to the Rust backend.
- Updated the user shell so accounts with manage_loads can navigate directly into the Rust load builder.
- Load creation is now partial; document uploads, Google-style address autocomplete, edit mode, and richer parity with the Blade workflow are still pending.

### 2026-04-07 - Master Data First Write Flows

- Added shared Rust contracts for master-data write actions and DB-backed option payloads.
- Added backend write routes for `POST /master-data/load-types`, `POST /master-data/equipments`, `POST /master-data/commodity-types`, and `POST /master-data/locations`.
- Added SQLx upsert helpers for the first writable master-data tables on PostgreSQL.
- Upgraded the Leptos master-data page so operators can create or update those records from the Rust admin surface.
- Updated the migration score so master-data CRUD is now `partial` instead of `not_started`.

## Next Recommended Slice

- Move into load detail/profile and document handling so the newly created Rust loads have a real follow-up workflow after `/loads/new` saves them.
- After that, deepen the load builder with document upload rows and address autocomplete, then return to richer master-data edit/delete coverage for countries, cities, and the remaining admin catalogs.
