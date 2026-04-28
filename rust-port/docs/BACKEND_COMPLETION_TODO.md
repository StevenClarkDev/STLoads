# Backend Completion Todo

Last updated: 2026-04-27

This checklist tracks the backend-only work required before the Rust service can replace the PHP backend with confidence. Frontend side-by-side QA stays separate.

## Done In Current Backend Slice

- [x] Add Rust outbound email service with env-driven `log`, `disabled`, and `smtp` modes.
- [x] Wire registration OTP email from Rust.
- [x] Wire OTP resend email from Rust.
- [x] Wire password-reset OTP email from Rust.
- [x] Wire account approve, reject, and revision emails from Rust admin review.
- [x] Wire admin account status-change emails for approved, rejected, and revision-requested states.
- [x] Wire load approve, reject, and revision emails from Rust admin load review.
- [x] Document IBM-safe mail env settings in `.env.ibm.example`.
- [x] Expose `mailer_mode` in backend `/health`.
- [x] Verify the backend compiles with `cargo check -p backend`.
- [x] Add focused backend mail tests for log mode, disabled mode, and fail-closed SMTP config.
- [x] Add real Stripe `Stripe-Signature` verification for platform/connect webhook secrets.
- [x] Parse real Stripe `payment_intent.*` and `account.updated` webhook payloads into the Rust escrow/account sync contract.
- [x] Add focused backend tests for Stripe signature verification, signature rejection, and real payment-intent payload parsing.
- [x] Read live Stripe keys using the same PHP env names: `STRIPE_SECRET`, `STRIPE_WEBHOOK_SECRET_PLATFORM`, and `STRIPE_WEBHOOK_SECRET_CONNECT`.
- [x] Add Rust Stripe Connect Express account creation and onboarding-link routes.
- [x] Add live Stripe PaymentIntent creation for escrow funding when `STRIPE_SECRET` is configured.
- [x] Add live Stripe Transfer creation for escrow release when `STRIPE_SECRET` is configured.
- [x] Keep deterministic/manual escrow behavior available when live Stripe is not configured, unless `STRIPE_LIVE_TRANSFERS_REQUIRED=true`.
- [x] Deploy current Stripe backend wiring to IBM Code Engine staging and verify hosted health against PostgreSQL.
- [x] Run hosted Stripe test-mode verification through Connect account creation/onboarding-link, PaymentIntent creation, card confirmation, and signed `payment_intent.succeeded` webhook funding.
- [x] Add durable `email_outbox` persistence with retry state, stale lock recovery, startup worker polling, and IBM-safe env controls.
- [x] Add Rust TMS retry and reconciliation workers with env-driven schedules, queued/push-failed handoff retry, stale-handoff detection, delivered-still-open warnings, and auto-withdraw/archive reconciliation.
- [x] Complete backend master-data CRUD parity for countries, cities, locations, load types, equipments, and commodity types, while keeping workflow status masters read-only.
- [x] Broaden DB-backed acceptance coverage for escrow, TMS webhooks, TMS retry/reconciliation workers, durable email outbox, and master-data CRUD.
- [x] Add route-level DB-backed acceptance tests for registration OTP, password reset OTP, account review emails, load review emails, documents, and execution lifecycle.
- [x] Add a hosted TMS worker validation script for IBM staging using the live backend plus IBM PostgreSQL.

## Remaining Backend Cutover Work

- [x] Configure a real IBM Code Engine email secret once the final sender/provider is chosen.
- [x] Run hosted staging with `MAIL_MAILER=smtp` and `MAIL_FAIL_OPEN=false`.
- [x] Run hosted SMTP staging validation against the new durable email outbox worker.
- [x] Complete hosted Stripe transfer-release verification after the test Express account finishes Stripe-hosted onboarding for the `transfers` capability.
- [x] Run final side-by-side PHP vs Rust backend behavior QA after the current backend source is redeployed to hosted staging.

## Current Backend Readiness

The backend is strong enough for cutover planning work: PostgreSQL, Code Engine health, COS-backed document storage, auth/session flows, load/desk/execution/payments/TMS/admin/master-data routes, durable mail outbox and retry workers, and route-level DB-backed acceptance coverage for auth, admin review, execution, and master-data now exist in Rust. The backend-only hosted hardening gates are closed, and the previously open manual PHP-vs-Rust QA blocker has also been verified in the staging findings log.

## Latest Hosted Validation Note

- On 2026-04-24, the current Rust backend source was rebuilt and redeployed to IBM Code Engine as revision `stloads-rust-backend-00020`.
- On the same day, `cargo check -p backend` and `cargo test -p backend` both passed locally before the hosted rerun.
- We hardened `scripts/verify_rust_role_matrix.ps1` so it refreshes the disposable lifecycle-state QA accounts before validation and tests frontend routes with a browser-like `Accept: text/html` header, which avoids false `404` results from the frontend API proxy rules.
- After that hardening, `scripts/verify_backend_cutover_hosted.ps1` passed end to end again on IBM staging, including smoke flow, hosted Rust role and lifecycle matrix, SMTP, TMS workers, and Stripe release.
- The 2026-04-24 hosted Stripe rerun released escrow successfully with transfer `tr_3TPUIpLZLVGhpopD0YPIQ4Zr` for carrier account `acct_1TOTnoLMsudLt19f`.
- On 2026-04-21, the hosted SMTP gate closed. We copied the live PHP SMTP provider settings into the Code Engine runtime secret, rebuilt the current Rust backend, and ran `scripts/verify_smtp_hosted.ps1` against IBM staging.
- The hosted registration flow returned success with `Email notification sent.`, the `email_outbox` row for the validation recipient settled in `status=sent`, and delivery completed on the first attempt with `MAIL_FAIL_OPEN=false`.
- On 2026-04-21, the hosted Stripe verifier first moved further than before: Rust auto-recovered stale stored Stripe account ids, created a fresh Express account, created a live test PaymentIntent, confirmed it, and applied the signed `payment_intent.succeeded` webhook on IBM staging.
- Later on 2026-04-21, after Stripe-hosted onboarding was completed for staging carrier account `acct_1TOTnoLMsudLt19f`, `scripts/verify_stripe_hosted.ps1` passed end to end on IBM staging, including live test PaymentIntent confirmation, signed webhook funding, and transfer release with Stripe transfer `tr_3TOhX4LZLVGhpopD1igiLh76`.
- On 2026-04-21, `scripts/verify_tms_workers_hosted.ps1` plus direct IBM PostgreSQL inspection proved the hosted IBM retry worker picked up queued handoff `#9641` and republished it as load `#9343`.
- The same staging revision also proved the hosted reconciliation worker path: cancelled drift handoff `#9642` was auto-withdrawn, and `stloads_reconciliation_log` recorded `action=auto_withdraw` with `triggered_by=rust_tms_reconciliation_worker`.
- On 2026-04-22, `scripts/verify_backend_cutover_hosted.ps1` reseeded the IBM staging dataset, reran `smoke_test_backend.ps1`, `verify_rust_role_matrix.ps1`, `verify_smtp_hosted.ps1`, `verify_tms_workers_hosted.ps1`, and `verify_stripe_hosted.ps1`, restored the known onboarded staging carrier Stripe account after reseed, and finished with `result = ok`.
- The backend-only hosted gate is closed, and the hosted PHP-vs-Rust QA blocker was also closed on 2026-04-27 after the live PHP `AuthController.php` fix was deployed and `scripts/verify_php_lifecycle_states.ps1` passed for pending OTP, pending review, revision requested, and rejected states.
