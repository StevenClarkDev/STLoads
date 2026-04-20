# PHP vs Rust QA Findings Log

Last updated: 2026-04-18

Use this file while running `docs/PHP_RUST_SIDE_BY_SIDE_QA.md`.
Every side-by-side difference should be logged here, even if it is later marked as expected or accepted.

## Current QA Run

| Field | Value |
| --- | --- |
| Run status | `in_progress`: IBM hosted Rust backend redeploy, migration verification, COS validation, backend smoke, Rust-side operator QA account seeding and reseeding, hosted Rust frontend deployment, hosted Rust role-and-lifecycle route-matrix verification, and PHP login verification for all five core roles passed; full manual PHP vs Rust role-based comparison is still pending |
| PHP app URL | `https://portal.stloads.com` |
| Rust frontend URL | `https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud` |
| Rust backend URL | `https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud` |
| Database | IBM PostgreSQL staging / disposable staging dataset |
| Document storage | `ibm_cos`: validated against `stloads-rust-staging-docs` with protected Rust routes |
| Tester | `Codex automated preflight` |
| Started at | `2026-04-15 21:29 PKT` |
| Completed at | `pending`: manual browser QA still open |

## Cutover Gate

Frontend parity can move to `100% cutover-ready` only when:

- [ ] All `P0` findings are fixed.
- [ ] All `P1` findings are fixed.
- [ ] All security and document-access checks pass.
- [x] IBM COS-backed document upload/view survives Code Engine restart or redeploy.
- [ ] Admin, shipper, carrier, broker, and freight-forwarder happy paths pass.
- [ ] Any remaining `P2/P3` findings are explicitly accepted or scheduled.

## Finding Status Key

- `open`: confirmed issue, still needs action
- `in_progress`: fix is underway
- `fixed`: implemented and ready for retest
- `verified`: retested successfully
- `accepted`: known difference accepted for cutover
- `duplicate`: duplicate of another finding
- `not_repro`: could not reproduce during retest

## Findings

| ID | Severity | Status | Role | PHP route | Rust route | Summary | Owner | Fix/Decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| QA-001 | `P1` | `verified` | all | document flows | document flows | IBM COS-backed document validation passed through Rust protected routes for KYC, load, and execution documents. | IBM/runtime | Switched staging to `ibm_cos`, redeployed Code Engine, validated upload/view/deny rules, rolled a new revision, and verified documents still opened. |
| QA-002 | `P1` | `open` | all | all browser routes | all browser routes | Manual side-by-side PHP vs Rust role/session QA is still incomplete. Hosted Rust route and lifecycle-state verification now passes for admin, shipper, carrier, broker, freight-forwarder, pending OTP, pending review, revision requested, and rejected states, but the matching PHP lifecycle-state accounts are still not confirmed. | QA/operator | Use `https://portal.stloads.com` for the PHP app, provide or create the remaining lifecycle-state accounts/sessions, then run `docs/PHP_RUST_SIDE_BY_SIDE_QA.md` against the hosted Rust frontend URL. |

## P0 Stop-Ship Findings

None recorded yet.

## P1 Workflow Blockers

- QA-002: Manual PHP vs Rust browser QA is still open.

## P2 Workarounds Or Parity Gaps

None recorded yet.

## P3 Polish

None recorded yet.

## Retest Log

| Date | Finding ID | Result | Notes |
| --- | --- | --- | --- |
| 2026-04-15 | preflight | passed | Reseeded IBM PostgreSQL staging with `seed_postgres_smoke_data.sql`, then ran `smoke_test_backend.ps1` against `https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud`; final result was `ok`. |
| 2026-04-15 | QA-001 | verified | Created a fresh HMAC-enabled COS Writer key, validated direct COS PUT/GET/DELETE, updated Code Engine to `DOCUMENT_STORAGE_BACKEND=ibm_cos`, rebuilt/redeployed the backend, uploaded KYC/load/execution documents through hosted Rust routes, confirmed admin/uploader access, confirmed unrelated user `403` denial, rolled a new revision, and confirmed files still opened afterward. |
| 2026-04-16 | backend-redeploy | passed | Redeployed the current Rust backend source to IBM Code Engine as revision `stloads-rust-backend-backend-hardening-20260416044401`, verified `/health` exposes `mail_outbox`, `tms_retry_worker`, and `tms_reconciliation_worker`, confirmed the `email_outbox` table exists in IBM PostgreSQL, reseeded staging, updated `smoke_test_backend.ps1` for execution POD upload, and reran the hosted backend smoke pass with final result `ok`. |
| 2026-04-16 | qa-accounts | passed | Ran `scripts/seed_operator_qa_accounts.ps1` against the hosted IBM Rust backend and verified disposable QA accounts for broker approved, freight-forwarder approved, pending OTP, pending review, revision requested, and rejected states. |
| 2026-04-17 | php-logins | passed | Ran `scripts/verify_php_role_logins.ps1` against `https://portal.stloads.com`. Admin, shipper, carrier, broker, and freight-forwarder logins all reached their expected dashboards with the corrected shipper email. |
| 2026-04-17 | php-state-accounts | blocked | Tested the provided PHP accounts that were labeled as pending OTP, pending review, revision requested, and rejected. All four landed on `/dashboard`, so they are currently active logins rather than true lifecycle-state accounts for side-by-side QA. |
| 2026-04-18 | qa-accounts-refresh | passed | Re-ran `scripts/seed_operator_qa_accounts.ps1` against the hosted IBM Rust backend and re-verified the exact Pending OTP, Pending Review, Revision Requested, and Rejected Rust QA states. |
| 2026-04-18 | rust-frontend-hosting | passed | Rust frontend is now hosted at `https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud`, so browser QA can target a real IBM Code Engine Leptos deployment. |
| 2026-04-18 | rust-role-matrix | passed | Ran `scripts/verify_rust_role_matrix.ps1` against hosted IBM staging. Frontend health/routes passed, approved role access passed, admin-vs-non-admin route scoping passed, and pending-OTP/pending-review/revision-requested/rejected lifecycle-state onboarding behavior matched the Rust contract. |

## Test Account Matrix

| Role / State | Email | PHP verified | Rust verified | Notes |
| --- | --- | --- | --- | --- |
| Admin | `admin@stloads.com` | [x] | [x] | PHP admin login reached `/admin_dashboard`. Rust staging admin smoke login and session passed. |
| Shipper | `steven333clark.developer@gmail.com` | [x] | [x] | PHP shipper login reached `/dashboard`. Rust shipper smoke login and session passed. |
| Carrier | `crankin.carrier@yahoo.com` | [x] | [x] | PHP carrier login reached `/dashboard`. Rust backend smoke login, session, booking, execution, and chat flows passed. |
| Broker | `crankin.broker@yahoo.com` | [x] | [x] | PHP broker login reached `/dashboard`. Disposable Rust staging broker account verified in approved state through register, OTP, onboarding, and admin approval flow. |
| Freight forwarder | `oglyguy@yahoo.com` | [x] | [x] | PHP freight-forwarder login reached `/dashboard`. Disposable Rust staging freight-forwarder account verified in approved state through register, OTP, onboarding, and admin approval flow. |
| Pending OTP | `pending.otp.qa@stloads.test` | [ ] | [x] | Disposable Rust staging pending-OTP shipper account verified; login/session and OTP resend path both worked. |
| Pending review | `pending.review.qa@stloads.test` | [ ] | [x] | Disposable Rust staging carrier account verified in pending-review state after OTP and onboarding submission. |
| Revision requested | `revision.requested.qa@stloads.test` | [ ] | [x] | Disposable Rust staging shipper account verified in revision-requested state after admin review action. |
| Rejected | `rejected.qa@stloads.test` | [ ] | [x] | Disposable Rust staging broker account verified in rejected state after admin review action. |

## Manual Run Notes

Use this section for observations that are not findings yet.

- Automated backend preflight passed after reseeding the deterministic smoke dataset.
- Initial smoke attempt failed because the seeded leg had already been mutated to `Paid Out`; reseeding fixed the state and the rerun passed.
- Local `rust-port/.env.ibm.runtime` had an unresolved clean Code Engine URL. It was corrected locally to the healthy generated Code Engine URL.
- COS validation passed after switching staging to `ibm_cos` and redeploying the current backend image.
- PHP app URL is now recorded from local config as `https://portal.stloads.com`.
- Rust-side disposable QA operator accounts are now seeded and verified through the hosted IBM backend using `scripts/seed_operator_qa_accounts.ps1`.
- PHP login verification now passes for admin, shipper, carrier, broker, and freight-forwarder through `scripts/verify_php_role_logins.ps1`.
- Hosted Rust role and lifecycle-state verification now also passes through `scripts/verify_rust_role_matrix.ps1`, so the remaining QA-002 risk is on the PHP lifecycle-state side and true browser comparison rather than on Rust route uncertainty.
- The PHP accounts later provided for pending OTP, pending review, revision requested, and rejected do not currently behave as those states; all four still log into `/dashboard`.
- Manual PHP vs Rust role-based QA is still pending because the remaining PHP lifecycle-state accounts are not yet available even though the Rust frontend is now hosted.
