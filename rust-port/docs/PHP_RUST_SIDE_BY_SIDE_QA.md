# PHP vs Rust Side-By-Side QA Checklist

Last updated: 2026-04-15

This checklist is for the final parity pass before treating the Rust/Leptos app as a real replacement for the Laravel Blade app.
Use it with the PHP app open in one browser/session and the Rust app open in another.
Record results and defects in `docs/PHP_RUST_QA_FINDINGS.md`.

## Goal

Find behavioral differences that only appear during real operator use.

At this stage, the major Rust screens exist. The remaining risk is mostly:
- small workflow differences
- missing labels, filters, or shortcuts
- permission edge cases
- hosted document-storage behavior
- confusing copy or next-step guidance

## Test Rules

- Use a staging database or disposable copied dataset.
- Do not run destructive tests against production data.
- Record every issue with the PHP route, Rust route, user role, expected behavior, actual behavior, and severity.
- Keep `docs/PHP_RUST_QA_FINDINGS.md` current as the source of truth for open P0/P1/P2/P3 items.
- Treat security and document-access differences as stop-ship.
- Treat small copy/layout differences as polish unless they change operator decisions.

## Severity

- `P0`: security, data loss, wrong payment, wrong TMS state, unauthorized document access
- `P1`: blocks a core workflow for admin, shipper, carrier, broker, or dispatch
- `P2`: workaround exists, but PHP flow is clearly better or faster
- `P3`: copy, spacing, clarity, or low-risk convenience issue

## Accounts To Use

Prepare one of each:
- admin
- shipper
- carrier
- broker
- freight forwarder
- pending OTP user
- pending review user
- revision-requested user
- rejected user

## Environment Signoff

- [ ] Rust backend `/health` returns healthy.
- [ ] Rust app points to IBM PostgreSQL staging or a disposable local equivalent.
- [ ] PHP app points to comparable staging data.
- [ ] `RUN_MIGRATIONS=false` after the first successful migration run.
- [ ] Document storage backend is known and recorded: `local` or `ibm_cos`.
- [ ] If using `ibm_cos`, bucket, prefix, and HMAC credential are confirmed.
- [ ] Browser geolocation can be tested with both allow and deny paths.
- [ ] Google address autocomplete key is configured for the Rust frontend build/runtime path.

## Auth And Onboarding

Compare PHP auth/onboarding screens with Rust `/auth/*` screens.

- [ ] Login works for all core roles.
- [ ] Invalid login errors are clear.
- [ ] Registration creates the expected role and status.
- [ ] OTP verify moves the user into the correct next state.
- [ ] OTP resend works for self-service users.
- [ ] Admin can resend OTP for pending-OTP users.
- [ ] Forgot/reset password flow completes.
- [ ] Onboarding continuation does not lose the session after OTP.
- [ ] Role-specific onboarding fields match PHP expectations.
- [ ] KYC upload works during onboarding.
- [ ] Revision-requested users are routed back to onboarding/profile repair.
- [ ] Approved users are routed into the product surface.
- [ ] Rejected users see clear locked-state behavior.

Stop-ship checks:
- [ ] A non-admin cannot access another user KYC document.
- [ ] An unauthenticated user cannot access onboarding or KYC files.

## User Profile And KYC Revision

Compare PHP profile/user-profile flows with Rust `/profile`.

- [ ] User can edit account facts.
- [ ] User can edit company facts.
- [ ] Carrier DOT/MC fields are present and save.
- [ ] Broker/freight-forwarder compliance fields are present and save.
- [ ] Password change works and requires valid confirmation.
- [ ] User can add a KYC row.
- [ ] User can edit KYC metadata.
- [ ] User can replace a KYC file.
- [ ] User can delete a KYC row.
- [ ] User can anchor/verify blockchain-style KYC metadata.
- [ ] Local SHA-256 verification works for matching and non-matching files.
- [ ] Document changes move the profile back into pending review when expected.

## Load Board

Compare PHP load board with Rust `/loads`.

- [ ] All/recommended/booked tab counts match expected scoping.
- [ ] Shipper sees only allowed loads.
- [ ] Carrier sees bookable/recommended loads.
- [ ] Admin sees expected broader visibility.
- [ ] Load cards show route, equipment, price, dates, finance/TMS signals, and next actions.
- [ ] Booking works for an available leg.
- [ ] Booking is blocked for already-booked or non-open legs.
- [ ] Realtime refresh updates after booking, offer, payment, or execution events.

## Load Builder

Compare PHP load add/edit with Rust `/loads/new` and `/loads/:load_id/edit`.

- [ ] Google address autocomplete works when GPS is allowed.
- [ ] Google autocomplete still returns broader/random results when GPS is denied.
- [ ] Pickup and delivery locations save with useful country/city/address metadata.
- [ ] Load type, equipment, commodity, rate, and date fields save correctly.
- [ ] Multi-leg behavior matches the current accepted Rust parity scope.
- [ ] Create routes to the Rust load profile.
- [ ] Edit preloads existing values.
- [ ] Edit saves back to the Rust profile.
- [ ] Edit is blocked once load state makes edits unsafe.

## Load Profile And Documents

Compare PHP load profile with Rust `/loads/:load_id` and `/admin/loads/:load_id`.

- [ ] User view shows load info, legs, documents, history, STLOADS context, and execution links.
- [ ] Admin view shows review controls when load is pending.
- [ ] Admin can approve, reject, and request revision with expected status/history changes.
- [ ] Admin sees oversight summary cards.
- [ ] Admin sees active-leg tracking shortcut.
- [ ] Admin sees STLOADS freshness timestamp.
- [ ] Admin sees per-leg finance state and correct fund/hold/release buttons.
- [ ] Confirm-before-finance behavior prevents accidental payment action.
- [ ] Payments deep links include useful context.
- [ ] Document upload works.
- [ ] Document metadata edit works.
- [ ] Document preview/view works for authorized users.
- [ ] Document download works for authorized users.
- [ ] Blockchain/hash information is visible where expected.

Stop-ship checks:
- [ ] Non-uploader non-admin cannot view load documents.
- [ ] Unauthorized document route access returns a failure, not a file.

## Chat And Offers

Compare PHP chat/offer flow with Rust chat workspace.

- [ ] Conversation list loads for authorized user.
- [ ] Active conversation loads.
- [ ] Messages send and appear without full page confusion.
- [ ] Read receipts update correctly enough for operator use.
- [ ] Presence/realtime state behaves acceptably.
- [ ] Offer accept/decline actions match PHP business rules.
- [ ] Unauthorized users cannot open unrelated conversations.

## Execution And Tracking

Compare PHP tracking page with Rust `/execution/legs/:leg_id`.

- [ ] Driver view, operations view, and customer visibility are clear.
- [ ] Action sequence follows pickup-to-delivery lifecycle.
- [ ] Invalid action order is blocked.
- [ ] GPS one-off ping works when geolocation is allowed.
- [ ] GPS denied path gives useful fallback guidance.
- [ ] Start live tracking works.
- [ ] Stop live tracking works.
- [ ] Tracking health reflects healthy/stale/no-ping states.
- [ ] Inline tracking-guidance checklist gives useful next steps.
- [ ] OSM map appears with current route context.
- [ ] Latest point link opens Google Maps.
- [ ] First point link opens Google Maps.
- [ ] Route-span handoff opens Google Maps directions.
- [ ] Approximate route distance is plausible.
- [ ] Timeline and note history match operator expectations.
- [ ] Pickup/delivery document upload works.
- [ ] Delivery completion requires POD.
- [ ] Delivery completion requires a note.
- [ ] Admin/shipper/carrier visibility matches PHP expectations.

Stop-ship checks:
- [ ] Non-authorized user cannot mutate execution state.
- [ ] Non-authorized user cannot view protected execution documents.

## Dispatch Desks

Compare PHP desk pages with Rust `/desk/:desk_key`.

- [ ] Quote desk rows match expected stage filter.
- [ ] Tender desk rows match expected stage filter.
- [ ] Facility desk rows match expected stage filter.
- [ ] Closeout desk rows match expected stage filter.
- [ ] Collections desk rows match expected stage filter.
- [ ] Admin sees all expected rows.
- [ ] Non-admin sees only owned/allowed rows.
- [ ] STLOADS counters and warnings are useful.
- [ ] Requeue action works where expected.
- [ ] Withdraw action works where expected.
- [ ] Close action works where expected.
- [ ] Follow-up note writes and reappears.
- [ ] Finance shortcuts work.
- [ ] Archive guidance is clear.
- [ ] Realtime refresh works after payment, TMS, booking, or execution updates.

## Admin Users And Roles

Compare PHP admin user screens with Rust `/admin/users`, `/admin/users/role/:role_key`, `/admin/roles`, and `/admin/change-password`.

- [ ] User search works.
- [ ] Role filter works.
- [ ] Role-specific pages replace `users_by_role` behavior.
- [ ] Create user works.
- [ ] Edit profile facts works.
- [ ] Role/status update works.
- [ ] Role/status update invalidates active session when needed.
- [ ] Approve/reject/revision actions work from card and profile panel.
- [ ] Pending OTP resend works.
- [ ] Delete requires explicit confirmation and can be cancelled.
- [ ] Readiness-gap checklist catches missing phone, company, KYC, carrier, and broker fields.
- [ ] KYC file viewing works for admin.
- [ ] Role-permission matrix saves and affects new sessions.
- [ ] Admin password change works.

Stop-ship checks:
- [ ] Admin cannot accidentally delete self.
- [ ] Non-admin cannot access admin users or roles pages.

## Admin Loads And Finance

Compare PHP admin load screens with Rust `/admin/loads`, `/admin/loads/:load_id`, and `/admin/payments`.

- [ ] Status buckets match PHP expectations.
- [ ] Pending approval bucket shows expected rows.
- [ ] Fund release bucket shows expected rows.
- [ ] Admin load review actions write history.
- [ ] Finance buttons are only enabled when state allows.
- [ ] Fund action works.
- [ ] Hold action works.
- [ ] Release action works.
- [ ] Stripe webhook simulation updates expected state.
- [ ] Payments console deep links are prefilled from load/desk context.
- [ ] Admin load profile shows per-leg escrow state.
- [ ] STLOADS operations/reconciliation shortcuts are useful.

Stop-ship checks:
- [ ] Finance actions cannot be run by unauthorized users.
- [ ] Release cannot run twice for the same completed state.

## STLOADS/TMS Operations

Compare PHP STLOADS operations/reconciliation screens with Rust admin pages.

- [ ] Operations dashboard loads live backend data.
- [ ] Reconciliation dashboard loads live backend data.
- [ ] Push/queue payload works.
- [ ] Requeue works.
- [ ] Withdraw works.
- [ ] Close works.
- [ ] Status webhook updates handoff and local projection.
- [ ] Cancel webhook withdraws local projection.
- [ ] Rate update webhook marks requeue-required where expected.
- [ ] Sync error resolve works.
- [ ] Targeted realtime refresh updates admin pages.

## Master Data

Compare PHP master-data screens with Rust `/admin/master-data`.

- [ ] Load types create/update as expected.
- [ ] Equipment create/update as expected.
- [ ] Commodity types create/update as expected.
- [ ] Locations create/update as expected.
- [ ] Countries/cities remaining gap is explicitly accepted or ticketed.
- [ ] Delete/deactivate behavior is explicitly accepted or ticketed.

## Hosted COS Document Validation

Run only when `DOCUMENT_STORAGE_BACKEND=ibm_cos`.

- [ ] Upload onboarding KYC document.
- [ ] Upload self-profile KYC document.
- [ ] Upload load document.
- [ ] Upload execution POD document.
- [ ] Confirm object exists in bucket under expected prefix.
- [ ] Confirm object read goes through Rust protected route.
- [ ] Confirm admin can open each uploaded file.
- [ ] Confirm uploader can open their file.
- [ ] Confirm unrelated user cannot open the file.
- [ ] Restart or redeploy Code Engine app.
- [ ] Confirm previously uploaded file still opens after restart.

Stop-ship checks:
- [ ] File disappears after Code Engine restart.
- [ ] Direct object URL leaks private file without Rust auth.
- [ ] Protected Rust route allows unauthorized file access.

## Findings Log Template

Use this format for each issue:

```text
ID:
Date:
Tester:
Role:
PHP route:
Rust route:
Dataset/load/user:
Expected PHP behavior:
Actual Rust behavior:
Severity:
Screenshot/video:
Decision:
Owner:
Status:
```

The repo also includes a structured findings file at `docs/PHP_RUST_QA_FINDINGS.md`.

## Completion Criteria

Frontend parity can be treated as `100% cutover-ready` only when:
- every `P0` and `P1` finding is fixed
- all document checks pass on IBM COS
- admin, shipper, carrier, broker, and freight-forwarder happy paths pass
- PHP-only route dependencies are either removed or explicitly accepted for post-cutover
- operations signs off on desk, finance, execution, and admin account workflows
