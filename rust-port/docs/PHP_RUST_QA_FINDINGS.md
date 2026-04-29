# PHP vs Rust QA Findings Log

Last updated: 2026-04-29

Use this file while running `docs/PHP_RUST_SIDE_BY_SIDE_QA.md`.
Every side-by-side difference should be logged here, even if it is later marked as expected or accepted.

## Current QA Run

| Field | Value |
| --- | --- |
| Run status | `verified`: IBM hosted Rust backend cutover validation, COS validation, hosted Stripe release verification, Rust role-and-lifecycle route verification, PHP role login verification, manual browser QA, hosted PHP lifecycle-state verification, and the custom-domain production cutover all passed. All recorded QA findings QA-001 through QA-010 are now closed as `verified`. |
| PHP app URL | `https://portal.stloads.com` |
| Rust frontend URL | `https://portal.stloads.com` |
| Rust backend URL | `https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud` |
| Database | IBM PostgreSQL live IBM Code Engine runtime / disposable smoke dataset for verification |
| Document storage | `ibm_cos`: validated against `stloads-rust-staging-docs` with protected Rust routes |
| Tester | `Codex automated preflight + manual browser QA operator` |
| Started at | `2026-04-15 21:29 PKT` |
| Completed at | `2026-04-29 23:55 PKT`: custom-domain production cutover passed, backend reports `environment=production`, and the full hosted verification bundle passed against the live portal runtime |

## Cutover Gate

Frontend parity can move to `100% cutover-ready` only when:

- [x] All `P0` findings are fixed.
- [x] All `P1` findings are fixed.
- [x] All security and document-access checks pass.
- [x] IBM COS-backed document upload/view survives Code Engine restart or redeploy.
- [x] Admin, shipper, carrier, broker, and freight-forwarder happy paths pass.
- [x] Any remaining `P2/P3` findings are explicitly accepted or scheduled.

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
| QA-002 | `P1` | `verified` | all | all browser routes | all browser routes | Manual side-by-side PHP vs Rust role/session QA is now backed by hosted route and lifecycle-state verification on both stacks. Hosted Rust route and lifecycle-state verification passes for admin, shipper, carrier, broker, freight-forwarder, pending OTP, pending review, revision requested, and rejected states, and the matching hosted PHP lifecycle accounts now also pass those same state checks after the live `AuthController::otp()` fix was uploaded and caches were cleared on 2026-04-27. | QA/operator + PHP app | Uploaded the fixed PHP `AuthController.php` through cPanel, cleared Laravel caches on the hosted app, reran `scripts/verify_php_lifecycle_states.ps1`, and verified all four PHP lifecycle states including pending OTP. |
| QA-003 | `P2` | `verified` | admin | `/admin_dashboard` | `/dashboard` | Admin QA hit the Rust user dashboard first when opening `/dashboard`, which made the admin landing feel wrong even though the admin session itself was valid. | frontend | Browser retest on 2026-04-25 against frontend revision `stloads-rust-frontend-00017` confirmed that an admin session opening `/dashboard` stays inside the admin shell and shows admin-only navigation like `Lifecycle QA`, `Roles & Permissions`, `STLOADS Operations`, and `Reconciliation`. |
| QA-004 | `P2` | `verified` | admin | onboarding review, loads action modal, escrow console | `/admin/onboarding-reviews`, `/admin/loads`, `/admin/payments` | Rust admin action buttons for Request Revision, Reject, Send Back, and escrow shortcuts appeared inactive during browser QA because the flow stayed inline and the feedback/confirmation pattern was too subtle. | frontend | Browser retest on 2026-04-25 against frontend revision `stloads-rust-frontend-00019` confirmed that `/admin/onboarding-reviews` now shows the inline guidance copy, keeps the queue visible, and displays an immediate card-level notice such as `Submitting revision request for Rust QA Revision Requested inside the Rust review queue...` when `Request Revision` is clicked. Together with the earlier loads/payments retest, this closes the affordance issue as verified. |
| QA-005 | `P3` | `verified` | admin | user approval modal/edit flow | `/admin/users` | The Rust user-directory `Edit details` and `Close edit` flow did not feel responsive during QA because the inline form change was too subtle. | frontend | Browser retest on 2026-04-25 against frontend revision `stloads-rust-frontend-00018` confirmed that `/admin/users` now loads all 27 directory rows again and the first `Edit details` interaction expands into a visible `Hide edit form` state. |
| QA-006 | `P2` | `verified` | admin | onboarding review / reconciliation load | `/admin/onboarding-reviews`, `/admin/stloads/reconciliation` | Some Rust admin deep pages intermittently failed with `Failed to decode GET ... expected value at line 1 column 1` until a hard refresh. | frontend/runtime | Browser retest on 2026-04-25 against revision `00018` verified the fix: both admin pages now fetch JSON successfully from `/api/stloads/...`, no decode banner appears, onboarding reviews populate, and reconciliation shows live log data again. |
| QA-007 | `P3` | `verified` | admin | onboarding reviews | `/admin/onboarding-reviews` | The Rust onboarding review queue had no search affordance and no explicit way to open more detail before approving, requesting revision, or rejecting. | frontend | Browser retest on 2026-04-25 against revision `00018` verified that the onboarding review page now shows the search box, returns populated review rows, supports `View Details`, and opens the embedded `KYC Documents` detail panel correctly. |
| QA-008 | `P3` | `verified` | admin | master-data / lifecycle helper screens | `/admin/master-data`, `/admin/account-lifecycle` | QA reported that the main search experience on Master Data and Lifecycle QA did not work, making large admin lists harder to use. | frontend | Browser retest on 2026-04-25 verified that `/admin/master-data` now exposes `Search master data rows` and filters the visible catalog rows, and `/admin/account-lifecycle` now exposes `Search lifecycle accounts` and narrows the QA workspace correctly by email and lifecycle state. |
| QA-009 | `P2` | `verified` | public/auth + user roles | auth/logout flow | `/auth/login`, `/auth/forgot-password`, `/auth/verify-otp`, `/auth/reset-password`, user/admin shells | Public auth flow lost the password-reset token handoff, logout behavior could leave users visually stranded on dashboard routes, and freight-forwarder/user pages still exposed admin-oriented navigation too broadly. | frontend | Browser retest on 2026-04-25 against frontend revision `stloads-rust-frontend-00019` confirmed the remaining dashboard leak is gone: the freight-forwarder dashboard no longer renders `STLOADS Board Status`, `STLOADS routes linked`, `STLOADS Ops`, or `Reconciliation`, and instead shows the role-safe `Role workspace aligned` copy with only user-facing dashboard cards. |
| QA-010 | `P2` | `verified` | admin | user directory | `/admin/users` | The Rust admin user directory is loading the page shell but returning `No user directory data is available yet.` even though the admin dashboard shows 27 visible accounts. | frontend/runtime | Browser retest on 2026-04-25 against revision `00018` confirmed the fix: `/admin/users` now fetches JSON successfully from `/api/stloads/admin/users`, displays all 27 rows, and no longer shows the empty-directory state. |

## P0 Stop-Ship Findings

None recorded yet.

## P1 Workflow Blockers

None currently recorded.

## P2 Workarounds Or Parity Gaps

None currently recorded.

## P3 Polish

None currently recorded.

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
| 2026-04-20 | php-lifecycle-script | blocked | Added `scripts/verify_php_lifecycle_states.ps1` and ran it against the PHP accounts that had been supplied earlier as lifecycle-state candidates. All four still landed on `/dashboard`, so they are confirmed active accounts rather than true pending-OTP, pending-review, revision-requested, or rejected PHP users. |
| 2026-04-21 | tms-workers-hosted | passed | Redeployed the current Rust backend to Code Engine revision `stloads-rust-backend-00014`, tuned staging worker intervals, hardened `scripts/verify_tms_workers_hosted.ps1` with PostgreSQL `hostaddr` fallback, and validated both hosted worker paths against IBM PostgreSQL. Queued handoff `#9641` republished as load `#9343`, and cancelled-drift handoff `#9642` auto-withdrew with `stloads_reconciliation_log.action=auto_withdraw` and `triggered_by=rust_tms_reconciliation_worker`. |
| 2026-04-21 | smtp-hosted | passed | Redeployed the current Rust backend to Code Engine revision `stloads-rust-backend-00016`, switched IBM staging to `MAIL_MAILER=smtp` with `MAIL_FAIL_OPEN=false`, fixed the Rust SMTP transport to use STARTTLS for Laravel-style `MAIL_ENCRYPTION=tls` on port `587`, and ran `scripts/verify_smtp_hosted.ps1`. The hosted registration OTP path returned `Email notification sent.`, and the matching `email_outbox` row settled in `status=sent` on the first attempt. |
| 2026-04-21 | stripe-hosted-release | passed | Redeployed the current Rust backend to Code Engine revision `stloads-rust-backend-00018`, hardened the payments route to recreate stale Stripe Connect account ids automatically, completed Stripe-hosted onboarding for staging carrier account `acct_1TOTnoLMsudLt19f`, and reran `scripts/verify_stripe_hosted.ps1`. Connect onboarding-link creation, PaymentIntent creation, signed webhook funding, DB escrow funding, and transfer release all passed on IBM staging, returning Stripe transfer `tr_3TOhX4LZLVGhpopD1igiLh76`. |
| 2026-04-22 | backend-cutover-hosted | passed | Ran `scripts/verify_backend_cutover_hosted.ps1` against IBM staging. The aggregate bundle reseeded the disposable PostgreSQL smoke dataset, reran `smoke_test_backend.ps1`, `verify_rust_role_matrix.ps1`, `verify_smtp_hosted.ps1`, `verify_tms_workers_hosted.ps1`, and `verify_stripe_hosted.ps1`, restored staged carrier Stripe account `acct_1TOTnoLMsudLt19f` after reseed, and finished with `result = ok`. |
| 2026-04-23 | qa-admin-parity-followup | fixed | Logged the latest manual admin QA notes, patched the frontend-only admin dashboard route handling, made onboarding/loads/payments actions much more explicit inline, improved the user-directory edit-form visibility, and reran `cargo check -p frontend-leptos` successfully. Staging deploy and browser retest are still pending. |
| 2026-04-24 | qa-admin-parity-deploy | passed | Confirmed the frontend-only IBM Code Engine deploy completed as revision `stloads-rust-frontend-00014`, verified the live root HTML is serving the updated frontend bundle, and confirmed the staged wasm contains the latest admin-dashboard, payments, and user-directory parity strings. Browser retest is still required to move QA-003, QA-004, and QA-005 from `fixed` to `verified`. |
| 2026-04-24 | backend-cutover-rerun | passed | Rebuilt the Rust backend to IBM Code Engine revision `stloads-rust-backend-00020`, hardened `scripts/verify_rust_role_matrix.ps1` to refresh lifecycle QA accounts and test frontend routes with a browser-style HTML `Accept` header, then reran `scripts/verify_backend_cutover_hosted.ps1`. Smoke, role matrix, SMTP, TMS worker, and Stripe release validation all returned `ok`; the latest Stripe release transfer was `tr_3TPUIpLZLVGhpopD0YPIQ4Zr`. |
| 2026-04-24 | qa-admin-auth-followup | fixed | Logged the next manual QA batch, then implemented local frontend fixes for JSON-first GET requests, onboarding-review search/detail toggles, master-data search, lifecycle search, admin-shell logout, user-shell logout redirects, password-reset query handoff, and tighter user-shell admin navigation. `cargo check -p frontend-leptos` passed. IBM staging redeploy is still pending because the IBM CLI session expired in this shell. |
| 2026-04-25 | qa-003-to-qa-009-browser-retest | mixed | Browser retest against live frontend revision `stloads-rust-frontend-00017` verified QA-003 and QA-008, partially verified QA-004, reproduced QA-006 on `/admin/onboarding-reviews` and `/admin/stloads/reconciliation`, left QA-007 blocked behind that decode error, and showed that QA-009 still has non-admin admin-link leakage even though logout and reset routes now render correctly. The same pass also surfaced QA-010 because `/admin/users` returned no directory rows while the admin dashboard still reported 27 visible accounts. |
| 2026-04-25 | qa-006-009-010-fix-batch | mixed | Frontend revision `stloads-rust-frontend-00018` went live from Code Engine build `stloads-rust-frontend-run-260425-05281195`. Browser retest verified the `/api/stloads/...` proxy fix end to end: onboarding reviews, reconciliation, and the admin user directory now load JSON and render populated data again, and QA-005/QA-006/QA-007/QA-010 are now verified. The remaining miss is QA-009 because the freight-forwarder dashboard still renders STLOADS-oriented cards even though auth/logout behavior is improved. |
| 2026-04-25 | qa-009-dashboard-retest | passed | Frontend revision `stloads-rust-frontend-00019` went live from Code Engine build `stloads-rust-frontend-run-260425-074951936`. Browser retest of the freight-forwarder dashboard confirmed that STLOADS-only dashboard cards and copy are gone, `Active Views` dropped to `2`, and the page now shows the role-safe `Role workspace aligned` message instead. |
| 2026-04-25 | qa-004-onboarding-action-retest | passed | Browser retest against frontend revision `stloads-rust-frontend-00019` confirmed the onboarding-review affordance is now obvious: the queue renders action guidance inline, `Approve`/`Request Revision`/`Reject` are present on each card, and clicking `Request Revision` shows an immediate card-level notice while keeping the queue visible. |
| 2026-04-25 | php-lifecycle-state-rerun | blocked | Reran `scripts/verify_php_lifecycle_states.ps1` against `https://portal.stloads.com` using the current QA lifecycle credentials (`pending.otp.qa@stloads.test`, `pending.review.qa@stloads.test`, `revision.requested.qa@stloads.test`, `rejected.qa@stloads.test`). All four checks failed and returned to `/normal-login`, so matching PHP lifecycle-state accounts were still not available at that point. |
| 2026-04-25 | php-lifecycle-account-provisioning | mixed | Created real hosted PHP lifecycle accounts directly on `https://portal.stloads.com` for pending OTP, pending review, revision requested, and rejected states using the live registration, OTP, onboarding, and admin-review flows. Pulled OTPs from the PHP admin recent-activity feed, moved carrier to pending review, broker to revision requested, and freight-forwarder to rejected, then reran `scripts/verify_php_lifecycle_states.ps1`. Pending review, revision requested, and rejected passed; pending OTP still failed because the live PHP `otp()` route throws and bounces the user back to `/normal-login` with a generic error instead of rendering the OTP screen. |
| 2026-04-27 | QA-002 | verified | Uploaded the fixed PHP `app/Http/Controllers/AuthController.php` through cPanel, cleared Laravel caches on the hosted app, and reran `scripts/verify_php_lifecycle_states.ps1` with the hosted lifecycle credentials. All four PHP lifecycle-state checks passed: pending OTP now lands on `/otp`, and pending review, revision requested, and rejected continue to route to their expected login/dashboard states. |

## Test Account Matrix

| Role / State | Email | PHP verified | Rust verified | Notes |
| --- | --- | --- | --- | --- |
| Admin | `admin@stloads.com` | [x] | [x] | PHP admin login reached `/admin_dashboard`. Rust staging admin smoke login and session passed. |
| Shipper | `steven333clark.developer@gmail.com` | [x] | [x] | PHP shipper login reached `/dashboard`. Rust shipper smoke login and session passed. |
| Carrier | `crankin.carrier@yahoo.com` | [x] | [x] | PHP carrier login reached `/dashboard`. Rust backend smoke login, session, booking, execution, and chat flows passed. |
| Broker | `crankin.broker@yahoo.com` | [x] | [x] | PHP broker login reached `/dashboard`. Disposable Rust staging broker account verified in approved state through register, OTP, onboarding, and admin approval flow. |
| Freight forwarder | `oglyguy@yahoo.com` | [x] | [x] | PHP freight-forwarder login reached `/dashboard`. Disposable Rust staging freight-forwarder account verified in approved state through register, OTP, onboarding, and admin approval flow. |
| Pending OTP | `qa.lifecycle.shipper.20260425040645@example.com` | [x] | [x] | Real hosted PHP pending-OTP shipper account now passes verification after the live `AuthController.php` fix and cache clear. |
| Pending review | `qa.lifecycle.carrier.20260425040650@example.com` | [x] | [x] | Real hosted PHP carrier lifecycle account now passes pending-review verification; password `QaPass123!`. |
| Revision requested | `qa.lifecycle.broker.20260425040655@example.com` | [x] | [x] | Real hosted PHP broker lifecycle account now passes revision-requested verification; password `QaPass123!`. |
| Rejected | `qa.lifecycle.freight-forwarder.20260425040700@example.com` | [x] | [x] | Real hosted PHP freight-forwarder lifecycle account now passes rejected verification; password `QaPass123!`. |

## Manual Run Notes

Use this section for observations that are not findings yet.

## Production Launch Notes

- `portal.stloads.com` now maps directly to the Rust frontend on IBM Code Engine.
- IBM domain mapping status reached `Ready` on 2026-04-29.
- The backend runtime now reports `environment=production` and `public_base_url=https://portal.stloads.com`.
- The final hosted verification rerun still passed end to end after the custom-domain cutover.

- Automated backend preflight passed after reseeding the deterministic smoke dataset.
- Initial smoke attempt failed because the seeded leg had already been mutated to `Paid Out`; reseeding fixed the state and the rerun passed.
- Local `rust-port/.env.ibm.runtime` had an unresolved clean Code Engine URL. It was corrected locally to the healthy generated Code Engine URL.
- COS validation passed after switching staging to `ibm_cos` and redeploying the current backend image.
- PHP app URL is now recorded from local config as `https://portal.stloads.com`.
- Rust-side disposable QA operator accounts are now seeded and verified through the hosted IBM backend using `scripts/seed_operator_qa_accounts.ps1`.
- PHP login verification now passes for admin, shipper, carrier, broker, and freight-forwarder through `scripts/verify_php_role_logins.ps1`.
- Hosted Rust role and lifecycle-state verification now also passes through `scripts/verify_rust_role_matrix.ps1`, so the remaining QA-002 risk is on the PHP lifecycle-state side and true browser comparison rather than on Rust route uncertainty.
- The hosted role-matrix script now refreshes the disposable Rust lifecycle-state QA accounts before asserting their states, so reruns stay stable even after staging data drifts during manual QA.
- `scripts/verify_php_lifecycle_states.ps1` now exists to verify PHP pending-OTP, pending-review, revision-requested, and rejected behavior with real hosted credentials.
- The earlier PHP accounts that had been labeled as pending OTP, pending review, revision requested, and rejected did not actually behave as those states; both manual checks and the verifier confirmed they logged into `/dashboard`.
- Fresh hosted PHP lifecycle accounts were created directly on 2026-04-25 through the live Laravel registration, OTP, onboarding, and admin-review flows, and the verifier now passes for pending review, revision requested, and rejected.
- The last PHP-side blocker is the pending-OTP redirect path: `AuthController::otp()` logs an undefined `$fromAddress`, catches its own exception, and sends the user back to `/normal-login` with the generic error banner instead of showing the OTP form.
- Manual PHP vs Rust role-based QA is still pending because the remaining PHP lifecycle-state accounts are not yet available even though the Rust frontend is now hosted.
- Hosted SMTP validation now also passes on IBM staging through `scripts/verify_smtp_hosted.ps1`, hosted Stripe release verification passes through `scripts/verify_stripe_hosted.ps1`, and the aggregate hosted backend bundle now passes through `scripts/verify_backend_cutover_hosted.ps1`, so the remaining cutover blocker is the manual PHP-vs-Rust browser parity pass rather than backend-hosting uncertainty.
- Hosted Stripe verification now also passes end to end on IBM staging, including connected-account recovery, onboarding-link creation, PaymentIntent confirmation, signed webhook funding, and transfer release after Stripe-hosted onboarding was completed for the staging carrier account.
- Latest manual admin QA noted that Rust often shows more linked or visible data than PHP on Dashboard, Users, and Loads, but the inline feedback for some admin actions was unclear enough that Request Revision, Reject, Send Back, Escrow actions, and Edit Details looked inactive to the tester.
