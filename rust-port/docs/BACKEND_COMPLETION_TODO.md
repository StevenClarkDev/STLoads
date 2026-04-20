# Backend Completion Todo

Last updated: 2026-04-21

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

- [ ] Configure a real IBM Code Engine email secret once the final sender/provider is chosen.
- [ ] Run hosted staging with `MAIL_MAILER=smtp` and `MAIL_FAIL_OPEN=false`.
- [ ] Run hosted SMTP staging validation against the new durable email outbox worker.
- [ ] Complete hosted Stripe transfer-release verification after the test Express account finishes Stripe-hosted onboarding for the `transfers` capability.
- [ ] Run hosted TMS worker validation against IBM PostgreSQL with scheduled retry/reconciliation enabled.
- [ ] Run final side-by-side PHP vs Rust backend behavior QA after the current backend source is redeployed to hosted staging.

## Current Backend Readiness

The backend is strong enough for continued IBM staging work: PostgreSQL, Code Engine health, COS-backed document storage, auth/session flows, load/desk/execution/payments/TMS/admin/master-data routes, durable mail outbox and retry workers, and route-level DB-backed acceptance coverage for auth, admin review, execution, and master-data now exist in Rust. It is not yet safe to retire PHP until the remaining hosted hardening and final acceptance gates are complete.

## Latest Hosted Validation Note

- On 2026-04-21, `scripts/verify_tms_workers_hosted.ps1` proved that the hosted IBM retry worker picked up queued handoff `#9639` and republished it as load `#9341`.
- The same hosted run created cancelled drift handoff `#9640`, but the reconciliation worker did not auto-withdraw it within the validation window, and no `rust_tms_reconciliation_worker` reconciliation row appeared for that handoff.
- The remaining TMS worker gate is now precise: inspect or lower the staging reconciliation interval on Code Engine, redeploy if needed, and rerun `scripts/verify_tms_workers_hosted.ps1` until the cancelled drift scenario is reconciled automatically.
