# Migration Scoreboard

Last updated: 2026-04-18

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
| Axum backend shell and route groups | `done` | Auth, dispatch, marketplace, execution, payments, TMS, admin, master-data, and realtime route groups exist. |
| Session auth, tokens, RBAC, and route scoping | `partial` | Login, session, OTP continuity, onboarding gating, KYC review access, route scoping, admin-driven role or status updates, and DB-backed role-permission session resolution now work; full broader permission-management parity is still incomplete. |
| Load board read surface | `partial` | Real auth-scoped load board exists with booking integration and execution refresh hooks, but full load-management parity is still missing. |
| Chat and offers | `partial` | Real chat, offer review, read receipts, presence, and scoped realtime exist; still not full Blade parity. |
| Payments and escrow lifecycle | `partial` | Fund, hold, release, real Stripe `Stripe-Signature` verification, platform/connect webhook secrets, real Stripe payment-intent/account payload parsing, Stripe Connect Express account/onboarding-link creation, live PaymentIntent creation, and live Transfer creation now exist. Hosted Stripe test-mode verification passed through Connect link creation, PaymentIntent creation, signed webhook funding, and DB escrow funding on 2026-04-16; final transfer-release verification is waiting on Stripe-hosted Express onboarding to enable the `transfers` capability for the staging carrier account. |
| STLOADS/TMS lifecycle and ops | `partial` | Push, queue, requeue, withdraw, close, status webhooks, admin ops views, and Rust background workers for queued/push-failed retry plus reconciliation scanning now exist. Hosted worker validation and deeper production parity are still pending. |
| Admin STLOADS reconciliation | `partial` | Real Rust screens and DB-backed data exist; broader admin surface is still incomplete. |
| Master-data admin visibility | `done` | Rust now has a DB-backed admin master-data catalog page with row-level create, edit, and safe delete or archive controls for countries, cities, locations, load types, equipments, and commodity types, while workflow statuses stay intentionally read-only. |
| Master-data CRUD | `done` | Rust supports create/update plus delete/archive flows for countries, cities, locations, load types, equipments, and commodity types. Load and offer status masters intentionally remain read-only because they drive canonical workflow state machines. |
| Load create and edit workflow | `partial` | Rust has `/dispatch/load-builder`, `/dispatch/loads/{id}/builder`, `/dispatch/loads`, and `/dispatch/loads/{id}/update` with Leptos `/loads/new` and `/loads/:load_id/edit`. Google-address autocomplete is wired for create and edit, successful saves route back into the Rust load profile, and edit mode safely locks booked or execution-stage loads out of builder rewrites. |
| Load detail, profile, and documents | `partial` | Rust has an auth-scoped `/dispatch/loads/{id}` detail surface plus Leptos `/loads/:load_id` for info, legs, binary document upload, restricted file viewing for admin plus uploader, metadata edits, history, latest STLOADS handoff context, blockchain follow-up controls, and richer admin document actions with preview/download plus hash visibility. |
| Self-service profile and account edit flow | `partial` | Rust now has `/profile` with self-serve account, company, password, protected KYC visibility, a revision-oriented KYC document workspace for add, edit, replace, blockchain anchor, delete actions, and local SHA-256 blockchain verification against stored hashes. Final profile parity and hosted document validation are still pending. |
| Tracking and execution UI | `partial` | Rust now has `/execution/legs/{leg_id}`, execution status actions, GPS ping writes, realtime refresh, timeline, tracking-point views, execution document upload, protected file viewing, POD-oriented document types, execution notes, Google Maps handoff for latest coordinates, execution-note history surfaced back into the UI, a delivery-completion guardrail that requires both a POD and a closing note, an inline OSM live-map embed with point-level map links and quick execution summary cards, plus true start/stop live driver tracking while the page stays open. The execution workspace now also surfaces an approximate route distance, the active tracking session window, start/end route point cards, route-span Google Maps handoff, tracking-health guidance, a stale-tracking recovery hint, a recommended next-step callout, an inline tracking-guidance checklist for thin or unhealthy GPS coverage, and new operator-readiness plus blocker panels that call out GPS, document, closeout, and handoff gaps directly from runtime data. Deeper map polish and broader carrier/operator workflow depth are still pending. |
| Onboarding and registration flows | `partial` | Rust now has role-aware registration, OTP verify and resend, password reset, onboarding continuation, secure KYC file upload, protected uploader-plus-admin document viewing, and admin approve, reject, and revision actions. Final polish, hosted validation, and broader user-management parity are still pending. |
| Admin CRUD for users and roles | `partial` | Rust now has an onboarding review queue, a broader admin user directory with search, role reassignment, account-status updates, user-history writes, session-refresh invalidation events, a DB-backed role-permission matrix at `/admin/roles`, a richer `/admin/users` surface with create, profile detail, KYC visibility, profile-edit, guarded delete flows, in-place approve/reject/revision shortcuts from both the directory cards and selected profile panel, plus directory attention-summary cards, queue guidance, pending-OTP resend support, safer delete cancellation, profile-level next-step guidance by account state, card-level missing-item hints, a profile-side readiness-gap checklist, and a dedicated `/admin/account-lifecycle` workspace for creating or driving Pending OTP, Pending Review, Revision Requested, and Rejected QA accounts. Role-filtered `/admin/users/role/:role_key` pages now replace the old Blade `users_by_role` surface, and there is a dedicated `/admin/change-password` page for the current admin account. Final polish and a few edge-case account-management actions are still pending. |
| Admin load and load-profile surfaces | `partial` | Rust now has `/admin/loads` with PHP-aligned status buckets, pending-load approve/reject/revision actions, direct in-row finance actions for release-ready and escrow-follow-up rows, plus an admin-shell `/admin/loads/{id}` profile route backed by the Rust load profile data and admin-only review/payment shortcuts. Admin profiles now expose per-leg payment state, direct fund/hold/release actions, payments deep links from the leg table itself, a load-level oversight summary, an active-leg tracking shortcut, STLOADS status freshness context, and a new blocker-checklist plus shortcut-handoff panel so admins can move faster between payments, execution, ops, and reconciliation without falling back to Blade instincts. Both admin loads and admin profiles now use a confirm-before-finance flow closer to the old Blade release modal behavior. The list page now also has attention-summary cards and queue guidance so admins can scan release, review, and execution follow-up faster. Final admin-profile and finance parity are still pending. |
| Dispatch desk pages | `partial` | Rust now has DB-backed quote, tender, facility, closeout, and collections desk boards at `/desk/:desk_key` with owner-vs-admin scoping, STLOADS desk counters, realtime refresh, first operator actions for requeue, withdraw, and close flows, inline desk follow-up note capture, finance/archive quick links, direct in-row fund/hold/release actions for closeout and collections, and clearer archive-state guidance. Broader desk workflow parity is still pending. |
| Email templates and outbound mail workflows | `partial` | Rust now has an env-driven outbound mail service with log, disabled, and SMTP modes. Registration OTP, OTP resend, password-reset OTP, account approve/reject/revision, admin account status changes, and load approve/reject/revision notifications are wired through Rust. Durable `email_outbox` persistence, stale lock recovery, and retry worker polling now exist; hosted SMTP provider validation and broader notification coverage are still pending. |
| File and document storage abstraction for IBM | `done` | The Rust upload and view flow includes an IBM Cloud Object Storage-compatible adapter and env contract. IBM staging now runs with `DOCUMENT_STORAGE_BACKEND=ibm_cos`, and hosted KYC, load, and execution document upload/view/deny/durability checks passed on 2026-04-15. |
| Frontend deployment strategy on IBM | `partial` | The Leptos frontend is now hosted as a separate Code Engine app and points at the hosted Rust backend through runtime config; final cutover strategy is still open. |
| Smoke-test tooling for IBM and PostgreSQL | `done` | Seed SQL, smoke script, IBM env template, deployment guide, and IBM staging checklist exist in `rust-port/scripts` and `rust-port/docs`. |
| Side-by-side PHP vs Rust QA process | `done` | `docs/PHP_RUST_SIDE_BY_SIDE_QA.md` now defines the final operator QA pass, severity model, COS validation gates, and completion criteria for calling the frontend cutover-ready; `docs/PHP_RUST_QA_FINDINGS.md` is the findings log for P0/P1/P2/P3 results. |
| IBM Code Engine backend deployment assets | `done` | Dockerfile, ignore files, runtime template, deployment guide, and a UBI-based build path now work on Code Engine. |
| IBM Code Engine staging backend validation | `done` | The hosted staging backend deploys cleanly on Code Engine. The deterministic IBM PostgreSQL smoke dataset was reseeded on 2026-04-15, the full backend smoke pass returned `ok`, and COS-backed document validation passed after a current-source rebuild. |
| Side-by-side QA run status | `partial` | Automated IBM backend preflight and COS-backed document validation are green, the hosted Rust frontend URL is live, and the core PHP role logins are verified in `docs/PHP_RUST_QA_FINDINGS.md`; manual browser-based lifecycle-state QA remains the open P1 gate. |

## Legacy Surface Still To Replace

- Laravel routes: roughly 184 lines across `routes/web.php` and `routes/api.php`
- PHP controllers: 25
- PHP models: 31
- Blade views: 76

## Current Rust Surface

- Backend route modules: 10
- Leptos page modules: 17
- Rust backend is already meaningful, but frontend parity is still much smaller than the Laravel view surface.

## Current Remaining High-Value Work

1. Run manual browser-based PHP vs Rust side-by-side QA with real operator roles and fix every P0/P1 finding.
2. Deepen tracking and execution parity only where side-by-side QA finds real gaps.
3. Keep IBM COS-backed document validation green during future deploys.
4. Finalize frontend hosting and cutover on IBM.
5. Build broader acceptance and integration coverage before retiring Laravel.
6. Finish the remaining execution/map polish plus the last closeout/collections operator edge cases that still depend on Blade behavior after the new tracking-session summaries and inline guidance checklist.
7. Finish the remaining admin load-profile and admin user-management edge polish that still carries Blade-era behavior after the new per-leg finance controls, confirm flow, in-place review shortcuts, and readiness-gap guidance.
8. Finalize frontend hosting and broader acceptance coverage for cutover confidence.
9. Run the side-by-side PHP vs Rust QA checklist and fix all P0/P1 findings before declaring frontend parity complete.
10. Harden outbound email with provider-specific staging credentials and broader notification coverage now that the durable retry outbox exists.

## Major Updates

### 2026-04-13 - Admin User Directory And Role Or Status Controls

- Added a real Rust admin user directory at `/admin/users` backed by PostgreSQL instead of relying only on the onboarding review queue.
- Added backend account-update actions so admins can change `role_id`, sync the Spatie `model_has_roles` pivot, update account status, and persist review remarks into `user_history`.
- Added a Leptos admin page with directory search, per-user role and status controls, and inline remarks editing so the Rust admin side now covers basic user-operations parity instead of forcing PHP fallbacks.
- Added session-refresh invalidation events after role or status changes so affected users are prompted back through a fresh Rust permission resolution path.

### 2026-04-13 - Dispatch Desk Boards In Rust

- Added a new Rust dispatch-desk surface at `/desk/:desk_key` so quote, tender, facility, closeout, and collections boards no longer depend entirely on Laravel Blade screens.
- Ported the same PHP desk stage filters into PostgreSQL-backed Rust reads: quote uses status `1`, tender uses `1` and `4`, facility uses `4`, `5`, and `6`, closeout uses `9` and `10`, and collections uses `10` and `11`.
- Preserved the legacy access model where admins see all matching desk rows and non-admin users only see loads they own.
- Added the STLOADS-focused desk counters and row signals that matter operationally, including quote eligibility, tender duplicate-risk warnings, facility readiness, closeout withdraw or archive follow-up, and collections archive-state visibility.
- Added a Leptos dispatch-desk page with desk-to-desk navigation and realtime refresh hooks so the Rust workflow can keep moving with booking, TMS, payment, and execution events.

### 2026-04-14 - Desk Actions And Role Permission Depth

- Added first real operator actions to the Rust dispatch desk by wiring desk-side requeue, withdraw, and close actions onto the existing TMS lifecycle routes instead of leaving the desk as a read-only board.
- Broadened Rust TMS lifecycle authorization so dispatch-desk and load-management users can run the desk-side lifecycle actions without pretending everything is admin-only.
- Added a new Rust admin role-permission page at `/admin/roles`, backed by `role_has_permissions` and the `permissions` table.
- Switched Rust session permission resolution to prefer database role permissions first, with the old static role contract kept as a safe fallback for roles that have not been written into the DB yet.
- Added session invalidation fan-out for users affected by role-permission changes so route guards and websocket scopes reconnect against the updated permission set.

### 2026-04-14 - Admin User Surface Deeper Frontend Parity

- Expanded the Rust `/admin/users` page beyond simple role or status edits so it now covers the old PHP admin user-directory depth more directly.
- Added Rust-backed admin user creation with role, status, password, phone, and address fields.
- Added on-demand Rust user profile loading with personal facts, company or regulatory facts, KYC document visibility, and recent review history.
- Added Rust profile-detail editing for name, email, password, phone, and address without leaving the admin user surface.
- Added a guarded Rust delete action that blocks self-delete and keeps the destructive step explicit instead of accidental.

### 2026-04-14 - Dispatch Desk Follow-Up Notes

- Added a frontend completion punch list at `docs/FRONTEND_COMPLETION_TODO.md` so the remaining Blade-to-Leptos work now has a dedicated UI-focused tracker.
- Added a real Rust dispatch-desk follow-up note action that can be submitted inline from quote, tender, facility, closeout, and collections rows.
- Wired desk follow-up notes through shared contracts, a new Rust dispatch route, PostgreSQL load-history persistence, and the Leptos desk page.
- Surfaced the latest activity note back onto each desk row so the new operator note flow is visible immediately instead of disappearing into backend-only history.

### 2026-04-14 - Admin Loads And Admin-Shell Load Profile

- Added a new Rust admin loads screen at `/admin/loads` with the same legacy PHP buckets used by the Blade admin load page: all, pending approval, approved or active, completed, and fund release.
- Wired the admin load list through new PostgreSQL admin-load queries and a dedicated `/admin/loads?tab=...` backend screen contract instead of faking it from the user-facing load board.
- Added an admin-shell `/admin/loads/{load_id}` route that reuses the Rust load-profile data inside the admin frame, so admins no longer need to leave the admin portal to inspect load details, documents, history, or STLOADS context.
- Updated admin navigation and dashboard route inventory so the new load surface is treated as a first-class Rust admin page rather than a side path.

### 2026-04-14 - Admin Load Review Actions

- Added Rust admin load review actions for pending loads so admins can approve, reject, or send back a load for revision directly from `/admin/loads`.
- Mirrored the legacy PHP load-review decisions onto the Rust backend by writing all legs on the load to status `2` for approve, `0` for reject, or `7` for revision, then appending the review into `load_history`.
- Added inline remarks capture in the Leptos admin loads page, with remarks required for reject and revision to preserve the old operational expectations.
- Kept the tracker honest: the review workflow is now in Rust, but email parity and the last finance shortcuts still remain.

### 2026-04-14 - Admin Profile Workflow Shortcuts

- Extended the admin-shell load profile route so admins can now run the same review decisions from `/admin/loads/{id}` instead of having to return to the list view first.
- Added admin-only profile shortcuts that detect pending-approval and release-ready legs from the Rust load profile data contract and surface the appropriate next action.
- Added a direct payments-console handoff from the admin profile when the load contains a release-ready leg, which closes another gap between the Blade admin load profile and the Rust admin shell.

### 2026-04-14 - Admin Finance Handoffs And Role-Filtered User Pages

- Upgraded the Rust payments console so `/admin/payments` can now accept release-context query parameters and prefill the leg, action intent, and source message when opened from admin load workflows.
- Changed release-ready admin load rows and admin-shell load profiles to deep-link into the payments console with the right leg id already selected, which removes another manual Blade-era handoff.
- Added a new Rust `/admin/users/role/:role_key` page that mirrors the old Blade `users_by_role` behavior by starting with approved accounts for the selected role and keeping inline profile, contact, and KYC visibility in the Rust admin shell.
- Added role-specific admin navigation for carriers, shippers, brokers, and freight forwarders so the new Leptos route is a real replacement instead of a hidden side path.

### 2026-04-14 - Self-Service Profile Workspace

- Added `/auth/profile-screen` and `/auth/profile` so authenticated users can load and save their own Rust profile without falling back to the legacy edit-profile controller.
- Added a new Leptos `/profile` page and user-shell navigation entry that combines the old profile view and edit flows into one self-service workspace.
- Ported the main business fields from the Blade edit form, including basic account info, company fields, carrier DOT/MC fields, broker MC/CBSA/USDOT and UCR/HCC fields, password changes, and protected KYC document visibility.
- Kept the tracker honest: this closes most of the legacy `users/edit_profile.blade.php` behavior, but the heavier revision-specific `user_profile.blade.php` document workflow still needs a final Rust replacement.

### 2026-04-13 - Execution Document And POD Workflow Slice

- Extended the Rust execution workspace so `/execution/legs/{leg_id}` now supports binary execution document upload directly from Leptos, without falling back to the Laravel tracking page.
- Added protected execution document download rules so only admin users and the profile that uploaded the leg document can open the binary file, while the broader execution screen remains visible to the usual owner, carrier, and admin audiences.
- Added leg-document persistence on PostgreSQL using the existing `leg_documents` table with structured metadata for original filename, mime type, file size, storage provider, and uploader identity.
- Added first-pass POD-oriented document parity by supporting the same execution document types used in the legacy tracking flow: pickup BOL, pickup photos, delivery POD, delivery photos, and other.
- Kept the execution slice IBM-ready by routing uploads through the same Rust document-storage abstraction already used elsewhere, while still allowing the project to move forward even before final IBM COS validation is closed out.

### 2026-04-13 - Delivery Completion Guardrails And Execution Notes

- Tightened the Rust execution action flow so `Complete delivery` now refuses to advance unless the leg already has a `delivery_pod` document attached.
- Added a required execution note for delivery completion so the final status change carries operator or carrier context instead of being an empty click-through.
- Added map-oriented polish to the execution screen by surfacing a Google Maps handoff link for the latest GPS point plus a quick tracking summary.
- Added an inline execution note editor so operators can attach context to pickup, transit, arrival, and delivery actions from the same Rust workspace.

### 2026-04-13 - Execution Note History Surfaced In Rust

- Extended the execution data contract with a dedicated note-history stream so execution-side remarks no longer disappear into backend-only `load_history`.
- Added PostgreSQL note-history reads for the current leg by filtering the load-history stream down to execution entries that belong to the active leg.
- Updated execution action persistence so Rust execution notes are written with explicit leg markers, making the history stream leg-aware instead of relying on generic load-level remarks.
- Added an execution-note history panel to the Leptos tracking page so carriers and admins can see who left context, what status it belonged to, and when it happened without leaving the Rust execution workspace.

### 2026-04-10 - IBM Code Engine Staging Smoke Pass

- Deployed the Rust backend successfully to IBM Code Engine with the live staging URL at `stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud`.
- Validated `/health` on Code Engine with IBM PostgreSQL connected and migrations applying during startup.
- Ran the full PowerShell backend smoke pass end to end against IBM staging: auth, load board, booking, execution, escrow, Stripe webhook sync, chat, offer review, STLOADS admin ops, reconciliation, and TMS push or webhook or requeue or cancel or close all completed successfully.
- Fixed multiple PostgreSQL runtime issues discovered only in hosted staging, including `NUMERIC` decoding on dispatch, marketplace, tracking, and TMS reads plus a placeholder mismatch in the STLOADS handoff insert query.
- Hardened the Code Engine container build path by switching the Dockerfile off Docker Hub base images to Red Hat UBI images, which avoids Docker Hub unauthenticated pull-rate failures during IBM builds.
- Hosted document validation remains provisional because the current staging runtime is still using `DOCUMENT_STORAGE_BACKEND=local`; final IBM Cloud Object Storage bucket and credential wiring still has to be completed and rechecked.

### 2026-04-08 - IBM Staging Assets And Execution First Pass

- Added the first real Rust execution workspace at `/execution/legs/{leg_id}` with auth-scoped access, sequenced pickup-to-delivery actions, GPS location ping writes, realtime refresh, execution timeline, and tracking-point views.
- Linked execution into the Rust load profile so users can move from load profile to execution without touching Laravel screens.
- Extended shared realtime contracts to include execution topics and targeted execution events.
- Upgraded the PowerShell smoke script so hosted validation now exercises booking, execution lifecycle actions, GPS pings, payments, chat, offers, admin ops, and TMS flows in one run.
- Added `docs/IBM_STAGING_SMOKE_CHECKLIST.md` so IBM PostgreSQL plus IBM Cloud Object Storage validation has a concrete hosted checklist instead of only ad hoc notes.
- Verified the execution slice with `cargo fmt` and `cargo check`; follow-up `cargo test` remains the final verification step for this milestone.

### 2026-04-08 - Rust KYC Intake And Admin Review Flow

- Added secure Rust onboarding KYC upload with multipart handling, protected download routes, and uploader-plus-admin visibility rules on the auth surface.
- Extended the onboarding screen contract so verified users can upload KYC files during onboarding and immediately see the protected document list in Leptos.
- Added a Rust admin onboarding review queue with pending and revision accounts, KYC document review, and approve, reject, or revision-request actions.
- Wired realtime notifications so admin dashboards and affected users receive account review updates from the Rust flow.
- Verified the full slice with cargo fmt, cargo check, and cargo test; the existing 3 lifecycle tests remain green and env-gated DB integration tests still skip unless RUST_TEST_DATABASE_URL is set.

### 2026-04-08 - OTP Continuity And Rust Onboarding Continuation

- Extended the Rust auth flow so successful registration OTP verification now issues a real session token and routes the user directly into the next required step instead of dropping them at a dead end.
- Added backend `GET /auth/onboarding-screen` and `POST /auth/onboarding` with auth-scoped gating, status checks, PostgreSQL user-details persistence, and Pending Review transition writes.
- Added Leptos `/auth/onboarding` with role-aware onboarding fields for shipper, carrier, broker, and freight forwarder accounts, plus read-only behavior when status no longer allows edits.
- Updated session routing so accounts in `EmailVerifiedPendingOnboarding` or `RevisionRequested` keep landing on the Rust onboarding step until they submit the form again.
- Kept the continuity explicit in UI notes so OTP-pending, onboarding-open, pending-review, approved, and revision-requested accounts each get a clear next-step message.
- Verified the slice with cargo fmt, cargo check, and cargo test; the existing 3 lifecycle tests remain green and env-gated DB integration tests still skip unless RUST_TEST_DATABASE_URL is set.

### 2026-04-08 - Rust Auth Self-Serve Slice

- Added shared Rust contracts for registration, OTP verification and resend, forgot-password, and reset-password flows so backend and Leptos stay aligned.
- Added backend auth routes for `POST /auth/register`, `POST /auth/verify-otp`, `POST /auth/otp/resend`, `POST /auth/forgot-password`, and `POST /auth/reset-password` with PostgreSQL-backed persistence.
- Added database helpers for registered-user creation, OTP refresh and consumption, password reset token issuance, and password update consumption on PostgreSQL.
- Added Leptos auth pages for login, register, OTP verify, forgot password, and reset password, turning the first Laravel-only auth surface into a real Rust user flow.
- Kept secrets IBM-safe by exposing OTP and reset tokens only outside production so development and staging can move quickly without baking local-only assumptions into production runtime.
- Verified the slice with cargo fmt, cargo check, and cargo test; the existing 3 lifecycle tests remain green and env-gated DB integration tests still skip unless RUST_TEST_DATABASE_URL is set.

### 2026-04-08 - IBM Cloud Object Storage Adapter

- Replaced the temporary local-only production assumption with an IBM Cloud Object Storage-compatible adapter built on the S3 API for document upload and protected file reads.
- Added object storage runtime configuration for bucket, region, endpoint, credentials, path-style behavior, and key prefix so Code Engine secrets can drive deployment cleanly.
- Kept local-backed storage available for development fallback while making `DOCUMENT_STORAGE_BACKEND=ibm_cos` the intended IBM runtime path.
- Updated the IBM deployment env template and deployment guide so PostgreSQL plus IBM Cloud Object Storage are now the documented durable targets for the Rust port.
- Verified the adapter swap with cargo fmt, cargo check, and cargo test; the lifecycle tests remain green and env-gated DB integration tests still skip unless RUST_TEST_DATABASE_URL is set.

### 2026-04-08 - Secure Document Upload And View Flow

- Added backend multipart upload and protected file-view routes on the Rust dispatch surface so `/loads/{id}` can store and serve real document binaries without falling back to Laravel.
- Added storage-backed document persistence metadata including uploader identity and storage provider tracking, with file visibility limited to admin users plus the profile that uploaded the file.
- Added browser-side authenticated upload and protected file-open helpers for the Leptos load profile, including inline upload form wiring and View file actions for authorized rows.
- Updated the load profile UI so operators can upload documents, edit metadata, see uploader labels, and keep blockchain follow-up on the same Rust screen.
- Kept the storage layer IBM-ready by routing everything through a document storage abstraction; the current adapter is local-backed for development, while IBM object storage remained the next production-hardening step at that milestone.
- Verified the slice with cargo fmt, cargo check, and cargo test; the existing 3 lifecycle tests remain green and env-gated live DB tests still skip unless RUST_TEST_DATABASE_URL is set.

### 2026-04-08 - Rust Load Builder Edit Mode

- Added backend `GET /dispatch/loads/{load_id}/builder` and `POST /dispatch/loads/{load_id}/update` so the same Rust builder now supports safe load revision as well as creation.
- Extended the shared load-builder contract with draft payloads, mode metadata, and submit labels so backend and Leptos stay aligned for create vs edit state.
- Added PostgreSQL-backed builder-leg hydration and load update persistence that soft-replaces editable legs while preserving the existing load number.
- Upgraded Leptos so `/loads/new` and `/loads/:load_id/edit` share one Google-address form, prefill existing load data, and save back into the Rust load profile.
- Added a direct `Edit load` handoff from the Rust load profile and intentionally blocked builder edits once a load has booked or execution-stage legs.
- Verified the slice with cargo fmt, cargo check, and cargo test; the existing 3 lifecycle tests remain green and env-gated live DB tests still skip unless RUST_TEST_DATABASE_URL is set.

## Next Recommended Slice

- Finish IBM Cloud Object Storage wiring and rerun the hosted document-specific checklist items in `docs/IBM_STAGING_SMOKE_CHECKLIST.md`.
- After that, deepen dispatch desk parity further with broader closeout, collections, and finance workflow actions that still exist only in Laravel.
- Keep pushing frontend-only parity by porting the remaining Blade-heavy admin and operator screens that still do not have a strong Leptos counterpart, starting with finance-only edge cases and the last admin/profile holdouts.

### 2026-04-14 - Self-Service Profile Revision Workspace

- Extended the Rust `/profile` page so the old revision-heavy `user_profile.blade.php` document workflow is no longer just a passive holdout.
- Added profile-side KYC row create, metadata edit, file replacement, blockchain anchoring, protected file viewing, and delete actions on the Rust auth surface.
- Wired those actions through PostgreSQL-backed KYC mutations plus review-cycle status writes, so profile-side document changes now move the account back into pending review the same way the PHP revision flow does.
- Preserved the document access rule: uploaded KYC binaries are still viewable only by admin users and the profile that uploaded them.

### 2026-04-14 - Finance Shortcuts On Closeout And Collections Desks

- Extended the Rust dispatch-desk contract with finance labels plus secondary action links so closeout and collections rows can guide operators into the right next system instead of stopping at a status badge.
- Added PostgreSQL-backed escrow status visibility onto dispatch-desk rows, which lets the Rust UI show finance status directly on closeout and collections boards.
- Added desk-specific quick links for reconciliation, STLOADS operations, and the payments console to mirror the Blade-era operator shortcuts more closely.
- Added row-level payments-console handoffs for release/fund follow-up and row-level archive/reconciliation shortcuts for closeout and collections workflows.

### 2026-04-14 - Admin Profile Per-Leg Finance Controls

- Extended load-profile leg reads so Rust now carries escrow state for each leg instead of treating finance as a load-level afterthought.
- Added admin-only per-leg finance controls on the Rust admin load profile, including direct fund, hold, or release actions wherever the current escrow state allows them.
- Added payments-console deep links directly from each admin load-profile leg row so finance exceptions can still move into `/admin/payments` with the right context prefilled.
- Surfaced payment status directly inside the profile leg table, which closes another Blade-era operational gap between admin load detail and finance follow-up.

### 2026-04-14 - Execution Session Summaries And In-Place Admin Reviews

- Extended the Rust execution page with tracking-session summaries so operators can see the GPS window and approximate route distance without leaving the screen.
- Kept those execution improvements in the Rust UI layer, which removes one more practical reason to bounce back to the Blade tracking page during active loads.
- Added in-place approve, revision, and reject shortcuts directly to the Rust admin user directory and selected user profile panel for accounts still sitting in onboarding review states.
- Reused the same Rust onboarding-review endpoint behind those shortcuts, so the admin directory and profile surfaces now carry more of the real account-operations workload instead of forcing a switch to the separate review queue for every decision.

### 2026-04-15 - Confirm-First Admin Finance Actions

- Added confirm-before-finance behavior to the Rust admin loads board so `fund`, `hold`, and `release` actions no longer fire on the first click.
- Added the same confirm-first flow to per-leg finance actions on the Rust admin load profile, which makes the Rust UX much closer to the old Blade release modal behavior without leaving the screen.
- Kept the payments-console deep links in place for exception handling, while making the in-place Rust actions feel more deliberate and safer for operators.

### 2026-04-15 - Admin Document Actions And Route Endpoint Map Cards

- Extended the Rust load-profile document table with richer admin-side document behavior by adding inline preview/download actions and visible blockchain hash previews.
- That closes more of the old Blade admin load-profile document workflow, where operators could preview, download, and inspect document state directly from the profile page.
- Extended the Rust execution map pane with route-endpoint cards so operators can quickly open both the first and latest plotted points while reviewing the active tracking session.

### 2026-04-15 - Execution Health Guidance And Admin Attention Summaries

- Extended the Rust execution workspace with backend-driven tracking-health guidance so operators can immediately see whether location updates are healthy, stale, awaiting first ping, or simply parked outside a live-tracking stage.
- Added a recommended next-step callout to the Rust execution page so the current status leads directly into the right pickup, transit, POD, or finance follow-up action.
- Added attention-summary cards and queue guidance to `/admin/loads`, which makes review queues, release-ready rows, execution-active shipments, and follow-up-heavy rows much easier to scan before diving into the table itself.

### 2026-04-15 - Route Span Handoff Plus Admin Oversight Summaries

- Extended the Rust execution map pane again with a direct Google Maps handoff for the full first-to-latest GPS span, which gives operators a quicker route-context check than opening single points one by one.
- Added a load-level admin oversight summary onto the Rust load profile so pending review, release-ready, execution-active, and document-anchor follow-up counts are visible before an admin starts working through legs and documents.
- Added matching attention-summary cards and queue guidance to the Rust admin user directory, making pending-review, pending-OTP, revision-requested, KYC-missing, and approved account queues easier to scan from one place.

### 2026-04-15 - Pending OTP Admin Support And Stale Tracking Recovery Hint

- Added admin-side resend-OTP support to both the Rust user directory cards and selected profile panel, which closes a real account-operations edge for users stuck in the `pending_otp` state.
- Added a safer delete flow in the Rust user directory by surfacing an explicit cancel action once a delete confirmation has been armed.
- Added a stale-tracking recovery hint to the Rust execution workspace so operators are nudged toward sending a fresh GPS ping or restarting live tracking when the route health drifts into warning or danger.

### 2026-04-15 - Admin Next-Step Guidance And STLOADS Freshness Context

- Added profile-level next-step guidance to the Rust admin user profile so account states like pending OTP, pending review, revision requested, approved, and rejected all explain the likely next admin action.
- Extended the Rust admin load profile with an active-leg tracking shortcut right from the load-level oversight area, reducing the number of clicks between oversight and execution follow-up.
- Extended the Rust STLOADS handoff card with an explicit upstream-status timestamp so admins can see not just the TMS status label but also how fresh that upstream state is.

### 2026-04-15 - Tracking Guidance Checklist And Admin Readiness Gaps

- Added an inline tracking-guidance checklist to the Rust execution workspace so operators get explicit recovery steps when GPS coverage is thin, stale, or blocked from their current role.
- Added card-level missing-item hints to the Rust admin user directory so KYC, phone, and company gaps are visible before opening the full profile.
- Added a profile-side readiness-gap checklist to the Rust admin user profile so reviewers can immediately see what is still incomplete for carriers, brokers, freight forwarders, and other account types.

### 2026-04-15 - Side-By-Side PHP vs Rust QA Checklist

- Added `docs/PHP_RUST_SIDE_BY_SIDE_QA.md` as the final parity checklist for comparing Laravel Blade workflows against the Rust/Leptos replacement with real admin, shipper, carrier, broker, freight-forwarder, and edge-state users.
- Added `docs/PHP_RUST_QA_FINDINGS.md` as the structured findings log for side-by-side runs, with cutover gates, severity buckets, retest tracking, and the required role/account matrix.
- Included stop-ship checks for document access, finance actions, execution mutations, admin authorization, and IBM COS-backed document durability.
- Linked the checklist from the IBM staging smoke checklist and frontend tracker so the remaining work is now validation-driven instead of blind feature-building.

### 2026-04-15 - IBM Backend QA Preflight Rerun

- Reseeded the IBM PostgreSQL staging smoke dataset with `scripts/seed_postgres_smoke_data.sql` after a first smoke attempt found the deterministic leg already mutated to `Paid Out`.
- Reran `scripts/smoke_test_backend.ps1` against `https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud`; the final result was `ok`.
- Updated `docs/PHP_RUST_QA_FINDINGS.md` with the preflight pass plus the remaining P1 gates: COS-backed document validation and manual browser-based PHP vs Rust side-by-side QA.

### 2026-04-15 - IBM COS Hosted Document Validation

- Created a fresh HMAC-enabled Writer credential for `Cloud Object Storage-st` and validated direct COS PUT/GET/DELETE against `stloads-rust-staging-docs`.
- Switched staging to `DOCUMENT_STORAGE_BACKEND=ibm_cos`, updated the Code Engine secret, rebuilt the backend from the current Rust source, and redeployed it to Code Engine.
- Reseeded IBM PostgreSQL staging, uploaded KYC, load, and execution documents through the hosted Rust API, confirmed admin/uploader access, confirmed unrelated-user `403` denial, rolled a new Code Engine revision, and confirmed documents still opened afterward.
- Marked QA-001 verified in `docs/PHP_RUST_QA_FINDINGS.md`; the remaining P1 validation gate is manual PHP vs Rust browser QA.

### 2026-04-15 - Rust Outbound Email Service

- Added an env-driven Rust mail service with `MAIL_MAILER=log`, `disabled`, and `smtp` modes so IBM staging can run safely while production can use a real SMTP provider without code changes.
- Ported the core Laravel mail triggers into Rust: registration OTP, OTP resend, password-reset OTP, onboarding approve/reject/revision, admin account status changes, and load approve/reject/revision notifications.
- Added Rust HTML email templates for OTP and review/update decisions, plus `MAIL_FAIL_OPEN` behavior so staging workflows can continue while still surfacing delivery warnings.
- Exposed `mailer_mode` on `/health` and documented the new mail env contract in `.env.ibm.example`.
- Verified the slice with `cargo fmt`, `cargo check -p backend`, and focused backend mail tests for log, disabled, and fail-closed SMTP modes.

### 2026-04-15 - Stripe Webhook Signature Hardening

- Upgraded the Rust payments webhook from a smoke-test-only shared-secret header to real Stripe `Stripe-Signature` HMAC verification over the raw request body.
- Added support for separate platform and Connect webhook secrets while keeping the existing `x-stripe-webhook-secret` fallback for deterministic staging smoke scripts.
- Added real Stripe payload parsing for `payment_intent.succeeded`, `payment_intent.payment_failed`, and `account.updated` so hosted Stripe events can map into the existing Rust escrow and Connect-state sync contract.
- Verified the slice with `cargo fmt`, `cargo check -p backend`, and backend tests for valid signatures, rejected signatures, and real payment-intent payload parsing.

### 2026-04-15 - Live Stripe Connect Backend Wiring

- Matched the Rust runtime to the PHP `.env` Stripe variable names without hardcoding secrets: `STRIPE_SECRET`, `STRIPE_WEBHOOK_SECRET_PLATFORM`, and `STRIPE_WEBHOOK_SECRET_CONNECT`.
- Added Rust Stripe API service support for Express account creation, account onboarding links, PaymentIntent creation, and Transfer creation.
- Added carrier self-service and admin-assisted Stripe Connect onboarding-link backend routes.
- Upgraded escrow funding so Rust can create a live Stripe PaymentIntent and persist the escrow as unfunded until Stripe confirms `payment_intent.succeeded`.
- Upgraded escrow release so Rust can create a live Stripe Transfer from the captured charge into the carrier connected account, while keeping manual deterministic release available unless `STRIPE_LIVE_TRANSFERS_REQUIRED=true`.
- Verified the slice with `cargo fmt`, `cargo check -p backend`, and the existing backend test suite; hosted Stripe verification remains the next gate.

### 2026-04-16 - Hosted Stripe Staging Verification

- Rebuilt and deployed the current Rust backend to IBM Code Engine as revision `stloads-rust-backend-stripe-verify-20260416010035`, pinned to image digest `95d26d`.
- Updated the Code Engine runtime secret with Stripe test-mode values copied from the PHP `.env` names without printing secrets.
- Reseeded the IBM PostgreSQL smoke dataset and cleared the seeded fake carrier Stripe account so the hosted Connect route could create a real Stripe test Express account.
- Verified the hosted Rust backend can create a Stripe Connect onboarding link, book the smoke leg, create a live test PaymentIntent, confirm it with Stripe test card `pm_card_visa`, and apply a signed `payment_intent.succeeded` webhook to fund escrow.
- Transfer release reached Stripe and was blocked by Stripe account readiness: the new Express account still needs Stripe-hosted onboarding to activate the `transfers` capability. A fresh onboarding link was opened in the browser so transfer release can be retried after onboarding is completed.

### 2026-04-16 - Durable Email Outbox And Retry Worker

- Added a PostgreSQL-backed `email_outbox` table for every Rust-generated OTP, account-review, account-status, and load-review email.
- Updated the Rust mailer so email workflows persist the message before delivery, mark sent/logged/skipped outcomes, and retain retryable records when SMTP delivery fails.
- Added stale lock recovery, exponential retry scheduling, configurable max attempts, and a startup background worker controlled by `MAIL_OUTBOX_*` environment variables.
- Exposed `mail_outbox` on backend `/health` and documented the new IBM runtime settings in `.env.ibm.example`.
- Verified the slice with `cargo fmt`, `cargo check -p db`, `cargo check -p backend`, and `cargo test -p backend`.

### 2026-04-16 - TMS Retry And Reconciliation Workers

- Added env-driven Rust background workers for STLOADS/TMS queued or push-failed retry handling and reconciliation scans, replacing the PHP `stloads:reconcile` cron pattern with Code Engine-safe in-process scheduling.
- Added DB-backed retry processing for queued and push-failed handoffs, including retry-count limits, failure recording, and sync-error creation when a retry cannot publish.
- Added reconciliation scanning for TMS financial terminal statuses, TMS cancellations, delivered-but-still-open handoffs, and stale published handoffs with no recent webhook activity.
- Tightened archive behavior so terminal TMS/close transitions soft-delete published load projections the way the PHP reconciler did.
- Exposed TMS worker state on backend `/health`, documented the new `TMS_*` env controls in `.env.ibm.example`, and verified with `cargo fmt`, `cargo check -p db`, `cargo check -p backend`, and `cargo test -p backend`.

### 2026-04-16 - Master Data CRUD Parity

- Added Rust backend create/update routes for countries and cities, completing the missing write side around the location dependency graph.
- Added delete/archive routes for countries, cities, locations, load types, equipments, and commodity types, matching the Blade-era admin master-data maintenance behavior.
- Fixed malformed SQL placeholders in the Rust country/city lookup helpers used by load and TMS location materialization.
- Updated the master-data screen contract so countries and cities are marked editable while load and offer status masters remain read-only by design.
- Verified the slice with `cargo fmt`, `cargo check -p shared`, `cargo check -p db`, `cargo check -p backend`, and `cargo test -p backend`.

### 2026-04-16 - Broader DB-Backed Acceptance Coverage

- Expanded `crates/db/tests/lifecycle_integration.rs` beyond the original escrow and TMS webhook tests.
- Added durable email outbox acceptance coverage for enqueue, retry, claim, delivery marking, and pending-count behavior.
- Added master-data CRUD acceptance coverage for country, city, location, load type, equipment, and commodity type create/delete flows.
- Added TMS reconciliation acceptance coverage for terminal-status auto-archive, stale-handoff sync-error creation, and queued handoff retry publishing.
- Verified with `cargo fmt`, `cargo test -p db`, and `cargo test -p backend`; the DB-backed suite ran successfully in this environment.

### 2026-04-14 - Direct Desk Finance Actions And Archive Guidance

- Extended the Rust dispatch-desk contract again so closeout and collections rows can now carry a recommended finance action and explicit archive guidance instead of leaving both as implied operator judgment.
- Added direct in-row `fund`, `hold`, and `release` desk actions on the Leptos closeout and collections boards by wiring them to the existing Rust payments routes.
- Kept the manual fallback path by preserving the payments-console deep links, so operators can still jump into `/admin/payments` for exceptions that need the full finance console.
- Added stronger archive-state badges and notes for closeout and collections rows so operators can see whether a load is still live, ready to archive, ready to close, or blocked on finance before archive.
- Expanded the payments-console prefill flow so desk-driven `fund`, `hold`, and `release` handoffs now open with the right context note, not just release-only context.

### 2026-04-14 - Admin Finance Actions And Profile Ops Shortcuts

- Extended the Rust admin loads screen so rows can now declare a direct finance action, not just a deep link into the payments console.
- Added in-row admin finance buttons for release-ready and escrow-follow-up rows by wiring `/admin/loads` straight into the existing Rust fund, hold, and release payments routes.
- Extended the Rust admin load profile so release-ready loads can now trigger a direct release from the profile, while still keeping the full payments console as the manual fallback.
- Added admin-only STLOADS ops and reconciliation shortcuts onto the profile-side handoff card so the Rust admin profile behaves more like the old Blade operations surface during finance and cleanup follow-up.

### 2026-04-14 - Admin Password Parity And Local Blockchain Verification

- Added a dedicated Rust `/admin/change-password` page inside the admin shell, replacing the old standalone Blade password screen with the same current-password confirmation flow.
- Added a real authenticated password-change endpoint on the Rust auth surface so the admin security page is backed by actual verification, not just frontend form polish.
- Extended the Rust profile-side KYC workspace so blockchain documents can verify a locally selected file with SHA-256 directly in the browser, closing one of the last high-visibility `user_profile.blade.php` behaviors.
- Kept the blockchain verification result visible in-row so users can immediately see whether the selected file matched the anchored hash without falling back to the old modal-based PHP flow.

### 2026-04-14 - Execution Map And Timeline Polish

- Added an inline OpenStreetMap embed to the Rust execution workspace so live tracking context now stays on the execution page instead of relying on the old Blade map view.
- Added execution summary cards for tracking points, timeline events, note volume, and delivery readiness to make the page read more like an operator console than a raw data dump.
- Added point-level map links from the tracking table and stronger timeline labeling so operators can jump to exact coordinates and parse event progression faster.
- Kept the existing Google Maps handoff for the latest point while broadening the execution page itself into a stronger tracking workspace.

### 2026-04-14 - Live Driver Tracking In Rust Execution

- Added a true start/stop live tracking flow to the Rust execution workspace so the booked carrier can keep sending GPS updates while the page stays open, much closer to the old PHP tracking behavior.
- Added an operator-mode banner that distinguishes driver view, operations view, and customer visibility on the execution page.
- Added live tracking status badges and explicit controls so the user can see when Rust is actively streaming location vs waiting for a manual GPS ping.
- Kept cleanup on component teardown so the browser watcher is stopped when the execution page is left.
