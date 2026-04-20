# Implementation Queue

Last updated: 2026-04-15

This file turns the remaining Laravel-to-Rust/Leptos migration work into an execution queue.
Priority labels are practical cutover labels, not abstract severity labels.

## Priority Key

- `critical`: blocks safe retirement of the PHP app or IBM staging validation
- `important`: needed for strong product parity, but not the first hard blocker
- `later`: valuable, but can follow after the main Rust cutover path is stable

## Remaining Estimate

- Critical path to serious IBM staging readiness: about `3 to 6 weeks`
- Broader parity after that: about `6 to 9 more weeks`
- Total remaining work to retire PHP safely: about `9 to 15 weeks`

These are solo-engineer estimates for focused implementation, not calendar promises.

## Critical

### 1. Final IBM Hosted Validation And Rollout Hardening

- Priority: `critical`
- Estimate: `1 to 3 days`
- Why it matters: the Rust backend is deployed on IBM Code Engine, the full backend smoke pass is green against IBM PostgreSQL, and COS-backed document durability is now verified. Manual browser side-by-side QA remains the last cutover validation gate.
- Remaining scope:
  - keep the deterministic IBM PostgreSQL smoke preflight green; last rerun on 2026-04-15 returned `ok`
  - keep the IBM COS-backed document validation green; last rerun on 2026-04-15 passed upload/view/deny/revision durability checks
  - capture and fix any remaining storage or proxy surprises from hosted staging
- Main files likely involved:
  - `rust-port/.env.ibm.example`
  - `rust-port/docs/IBM_CODE_ENGINE_DEPLOYMENT.md`
  - `rust-port/docs/IBM_STAGING_SMOKE_CHECKLIST.md`
  - `rust-port/scripts/smoke_test_backend.ps1`
- Depends on:
  - current backend deploy assets, already present

### 2. Tracking And Execution Workflow Depth

- Priority: `critical`
- Estimate: `3 to 7 days`
- Why it matters: Rust now has the first real execution workspace, but the execution domain is still not deep enough to retire the Laravel tracking experience.
- Remaining scope:
  - finish the map polish after the new inline OSM embed, point-level links, live driver tracking controls, tracking-session summaries, route-endpoint cards, route-span handoff, tracking-health guidance, stale-tracking recovery guidance, next-step callouts, and inline tracking-guidance checklist
  - stronger execution timeline parity beyond the new summary/tone pass
  - deeper admin and shipper visibility polish
  - follow-up execution workflow polish after the new leg-document upload slice and POD completion guardrail
- Main files likely involved:
  - `crates/backend/src/routes/execution.rs`
  - `crates/db/src/tracking.rs`
  - `crates/frontend-leptos/src/pages/execution.rs`
  - `crates/frontend-leptos/src/device_location.rs`
  - `crates/shared/src/execution.rs`
- Depends on:
  - canonical status model in `docs/CANONICAL_STATUS_MODEL.md`

### 3. Admin User And Role CRUD

- Priority: `critical`
- Estimate: `1 to 2 days`
- Why it matters: the Rust admin surface now has user create, profile detail, profile edit, guarded delete, onboarding reviews, a role-permission matrix, and stronger review guidance, but a few production ops edges still remain.
- Missing scope:
  - last edge-case account controls and polish after the new standalone admin change-password page, in-place review shortcuts, directory attention-summary layer, admin-side resend-OTP support, profile-level next-step guidance, and readiness-gap hints
  - any remaining admin account actions that still force a PHP fallback
- Main files likely involved:
  - new backend admin route handlers
  - `crates/db/src/auth.rs`
  - new Leptos admin user and role pages
- Depends on:
  - current RBAC and session base, already present

### 4. Auth And Onboarding Final Polish

- Priority: `critical`
- Estimate: `1 to 3 days`
- Why it matters: Rust now covers registration, OTP continuity, onboarding submission, KYC intake, and admin review, but the last polish and hosted validation still matter before this domain is called done.
- Remaining scope:
  - richer onboarding polish after submission
  - stronger validation and review remarks UX
  - hosted smoke validation on IBM PostgreSQL plus IBM COS
- Main files likely involved:
  - `crates/backend/src/routes/auth.rs`
  - `crates/db/src/auth.rs`
  - `crates/frontend-leptos/src/pages/auth.rs`
  - `crates/frontend-leptos/src/pages/onboarding_reviews.rs`
- Depends on:
  - current session and token layer, already present

## Important

### 5. Dispatch Desk Workflow Port

- Priority: `important`
- Estimate: `2 to 5 days`
- Why it matters: the read-side desk boards, lifecycle actions, and inline follow-up notes now exist in Rust, but the deeper operator workflows and parity are still incomplete.
- Missing scope:
  - the last closeout and collections exceptions after the new direct desk-side finance actions and archive guidance
  - richer desk-specific workflow summaries, operator notes, and finance exception follow-up controls
  - broader desk navigation polish after the first `/desk/:desk_key` cutover
- Legacy source surface:
  - `resources/views/desk`
- Depends on:
  - admin and user access clarity plus execution flow depth

### 6. Deeper Document Workflow Parity

- Priority: `important`
- Estimate: `3 to 6 days`
- Why it matters: secure upload and protected viewing exist now, but the broader document lifecycle is still shallow.
- Missing scope:
  - richer document typing and validation
  - better operator review UX
  - document replacement and versioning rules after the new preview/download/hash workflow
  - stronger audit annotations
  - final COS-specific validation in staging
- Main files likely involved:
  - `crates/backend/src/routes/dispatch.rs`
  - `crates/backend/src/document_storage.rs`
  - `crates/frontend-leptos/src/pages/load_profile.rs`
  - `crates/frontend-leptos/src/pages/execution.rs`

### 7. Remaining Master-Data Depth

- Priority: `important`
- Estimate: `3 to 6 days`
- Why it matters: first-write support exists, but parity and operator polish are incomplete.
- Missing scope:
  - countries and cities management
  - better edits
  - deactivation and deletes
  - broader admin UX polish
- Main files likely involved:
  - `crates/backend/src/routes/master_data.rs`
  - `crates/db/src/master_data.rs`
  - `crates/frontend-leptos/src/pages/master_data.rs`

### 8. Frontend Hosting And Cutover Strategy

- Priority: `important`
- Estimate: `3 to 5 days`
- Why it matters: backend-first deploy is ready, but final frontend hosting and cutover path is still unresolved.
- Missing scope:
  - choose SSR vs separate frontend deployment path
  - define public routing strategy
  - define same-origin vs split-origin auth behavior
  - finalize IBM hosting shape for Leptos
- Depends on:
  - staging validation and remaining page coverage

### 9. Remaining Admin And Operator Blade Screen Port

- Priority: `important`
- Estimate: `2 to 5 days`
- Why it matters: frontend parity still has a few high-visibility Blade holdouts, especially the admin load and admin load-profile surfaces.
- Missing scope:
  - finish the last admin-only finance and load-profile polish after the new direct row/profile and per-leg finance actions plus confirm-first finance UX, the new admin load attention-summary layer, the new profile-level oversight summary, and the STLOADS freshness plus active-leg tracking improvements
  - finish the remaining admin-only post-review polish after the Rust review actions and STLOADS/payment handoffs
  - final profile-oriented admin or user screens that still carry operational behavior after the new admin password page and local blockchain verification slice
- Main files likely involved:
  - new backend admin read models
  - new Leptos admin load pages
  - `resources/views/admin/load.blade.php`
  - `resources/views/admin/load_profile.blade.php`

## Later

### 10. Email And Notification Port

- Priority: `important`
- Estimate: `2 to 4 days`
- Why it matters: Rust now owns the core outbound email trigger points that PHP previously handled, but production cutover still needs provider credentials, retry hardening, and broader notification coverage.
- Missing scope:
  - configure a real SMTP/provider credential set in IBM Code Engine once the production sender is chosen
  - add queued retry/outbox persistence for failed delivery instead of only immediate send/log behavior
  - broaden notification coverage beyond OTP, account review, account status changes, and load review decisions
  - run hosted staging tests with `MAIL_MAILER=smtp` and `MAIL_FAIL_OPEN=false`

### 12. Live Stripe Verification

- Priority: `important`
- Estimate: `1 to 2 days`
- Why it matters: Rust now has live Stripe API wiring for Connect onboarding, PaymentIntent creation, webhook verification, and transfers, but those flows still need hosted verification with the configured Stripe account before PHP payments can be retired.
- Missing scope:
  - run Connect onboarding-link creation for a carrier on staging
  - create a Stripe PaymentIntent through the Rust escrow funding route
  - replay/receive a signed `payment_intent.succeeded` webhook and confirm escrow funding
  - release a funded escrow and confirm the Stripe Transfer id is stored
  - decide whether IBM staging should set `STRIPE_LIVE_TRANSFERS_REQUIRED=true`

### 11. Broader Acceptance Coverage

- Priority: `later`
- Estimate: `2 to 5 days`
- Why it matters: stronger confidence before final PHP shutdown.
- Missing scope:
  - run the side-by-side PHP vs Rust QA checklist in `docs/PHP_RUST_SIDE_BY_SIDE_QA.md`
  - record every finding in `docs/PHP_RUST_QA_FINDINGS.md`
  - fix every P0/P1 side-by-side finding before production cutover
  - auth onboarding tests
  - execution flow tests
  - document and COS tests
  - admin workflow tests
  - broader end-to-end happy path coverage

## Suggested Execution Order

1. Final IBM hosted validation with PostgreSQL plus IBM COS
2. Tracking and execution workflow depth
3. Dispatch desk workflow depth
4. Admin user and role CRUD polish
5. Remaining admin and operator Blade screen port
6. Auth and onboarding final polish
7. Deeper document workflow parity
8. Remaining master-data depth
9. Frontend hosting and cutover strategy
10. Email and notification port
11. Broader acceptance coverage

## Best Fast-Cutover Sequence

If the goal is to retire PHP as quickly as possible with acceptable risk, the best sequence is:

1. finish IBM hosted validation end to end
2. run side-by-side PHP vs Rust QA and fix P0/P1 findings
3. deepen tracking and execution only where the side-by-side pass shows real gaps
4. finish the remaining desk finance and closeout edge cases
5. deepen the new admin loads surface and remaining admin finance polish
6. finish auth and onboarding polish

That sequence gets us from a strong Rust foundation to a real operational replacement the fastest.
