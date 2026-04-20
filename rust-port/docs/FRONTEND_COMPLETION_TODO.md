# Frontend Completion TODO

Last updated: 2026-04-18

This file is the frontend-only punch list for finishing the Laravel Blade to Leptos migration.
It is intentionally more detailed than the broader implementation queue so we can work through the remaining UI surfaces one by one without losing momentum.

## Status Key

- `[x]` done in Rust/Leptos
- `[-]` partially done and still needs parity work
- `[ ]` not done yet

## Core User App

- [x] Rust auth login page
- [x] Rust register, OTP verify, OTP resend, forgot-password, reset-password pages
- [x] Rust onboarding continuation page
- [x] Rust onboarding KYC upload and protected file access
- [-] onboarding UX polish, stronger validation, and hosted document validation
- [x] Rust load board
- [x] Rust load builder create flow
- [x] Rust load builder edit flow
- [x] Google-address autocomplete in load builder
- [x] Rust self-service profile page
- [x] Rust profile-side KYC revision workspace with row add, edit, replace, anchor, delete, and protected viewing
- [x] local SHA-256 blockchain verification flow for profile KYC rows
- [x] Rust load profile page
- [x] Rust load-profile document upload and protected view flow
- [-] deeper load-profile parity and document lifecycle polish after the new preview/download/hash admin workflow improvements
- [x] Rust chat workspace
- [-] deeper chat polish and full Blade-level interaction parity
- [x] Rust execution workspace
- [x] execution notes, POD guardrails, and execution document upload
- [-] map polish and remaining execution UX depth after the new inline OSM map, tracking summary cards, approximate route distance and session summaries, start/end route point cards, point-level map links, route-span handoff, tracking-health cues, stale-route recovery guidance, next-step guidance, true start/stop live driver tracking, the inline tracking-guidance checklist, and the new operator readiness plus blocker panels

## Dispatch Desks

- [x] quote desk page
- [x] tender desk page
- [x] facility desk page
- [x] closeout desk page
- [x] collections desk page
- [x] realtime refresh on desk boards
- [x] desk-side requeue, withdraw, and close actions
- [x] desk follow-up note action from the Rust UI
- [-] broader closeout workflow parity
- [-] broader collections and finance workflow parity
- [x] closeout and collections quick links plus row-level payments/archive shortcuts
- [x] direct in-row finance actions and archive-state guidance on closeout/collections rows
- [-] deeper desk-specific operator notes, summaries, and last finance edge shortcuts

## Admin App

- [x] Rust admin dashboard
- [x] STLOADS operations page
- [x] STLOADS reconciliation page
- [x] payments operations page
- [x] onboarding review queue
- [x] admin role-permission matrix
- [x] admin user directory
- [x] admin lifecycle QA workspace for Pending OTP, Pending Review, Revision Requested, and Rejected account handling
- [x] admin user create flow
- [x] admin user profile detail view
- [x] admin user profile edit flow
- [x] guarded admin user delete flow
- [x] in-place approve, revision, and reject shortcuts from admin user directory and profile
- [x] role-filtered admin user pages replacing `users_by_role`
- [x] admin loads page with PHP-aligned status buckets and review actions
- [x] admin-shell load profile route with review, per-leg finance actions, payment shortcuts, and confirm-before-finance UX
- [x] direct admin load-list finance actions for release-ready and escrow-follow-up rows
- [x] confirm-before-finance UX on admin load rows
- [-] last admin account-management edge cases and polish after the new directory attention-summary cards, queue guidance, resend-OTP support, safer delete confirmation flow, profile-level next-step guidance, card-level missing-item hints, and profile-side readiness-gap checklist
- [-] full admin load listing parity after the new attention-summary cards and queue guidance
- [-] full admin load-profile parity after the new per-leg finance controls, payments deep links, confirm flow, load-level oversight summary cards, active-leg tracking shortcut, STLOADS status freshness context, and the new admin blocker plus shortcut handoff panel
- [x] admin change-password page parity

## Master Data And Back Office

- [x] master-data admin catalog page
- [x] first-write workflows for countries, cities, locations, load types, equipments, and commodity types
- [-] better edit UX and broader admin polish
- [x] countries management page
- [x] cities management page
- [x] delete/deactivate flows where business-safe

## Remaining Blade-Heavy Surfaces To Replace

- [ ] `resources/views/admin/load.blade.php`
- [ ] `resources/views/admin/load_profile.blade.php`
- [x] `resources/views/admin/users_by_role.blade.php`
- [x] `resources/views/admin/change_password.blade.php`
- [-] `resources/views/user_profile.blade.php` local verification and KYC row lifecycle parity now exist; remaining oddities should be treated as final polish only if still discovered
- [x] `resources/views/users/edit_profile.blade.php`
- [ ] any remaining user-profile variants that still carry business behavior not covered by Rust screens

## Cutover And IBM Hosting UI Work

- [-] backend-first IBM deployment path exists
- [ ] final frontend hosting shape on IBM
- [ ] public routing and cutover plan for Leptos
- [ ] same-origin or split-origin browser behavior signoff
- [x] final COS-backed hosted document validation in frontend flows
- [x] side-by-side PHP vs Rust frontend QA checklist exists at `docs/PHP_RUST_SIDE_BY_SIDE_QA.md`
- [x] side-by-side findings log exists at `docs/PHP_RUST_QA_FINDINGS.md`
- [-] side-by-side PHP vs Rust QA is in progress: automated IBM backend preflight and COS-backed document validation passed, manual browser QA still open

## Suggested Completion Order

1. Finish dispatch desk workflow depth
2. Deepen admin load-profile parity and finance shortcuts
3. Finish remaining admin account-management polish
4. Finish execution and map polish
5. Finish remaining user-profile and back-office pages
6. Run side-by-side PHP vs Rust QA and fix P0/P1 findings
7. Finalize IBM frontend hosting and document validation

## Current Working Slice

The current active slice is:

1. the last execution/operator polish after the new inline tracking-guidance checklist and operator blocker panels
2. then the final admin load-profile and account edge cases after the new readiness-gap, missing-item guidance, master-data CRUD pass, and admin blocker handoff panel
3. then a side-by-side PHP vs Rust QA pass using `docs/PHP_RUST_SIDE_BY_SIDE_QA.md`

This file should be updated after every major frontend milestone.
