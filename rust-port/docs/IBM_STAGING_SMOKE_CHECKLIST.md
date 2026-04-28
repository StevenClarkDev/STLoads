# IBM Staging Smoke Checklist

Last updated: 2026-04-24

This checklist is the operational companion to the Rust backend deployment guide.
Use it when the backend is deployed on IBM Code Engine with IBM PostgreSQL and IBM Cloud Object Storage.

## Goal

Prove that the current Rust stack works on IBM-hosted services end to end before deeper cutover work continues.

A successful pass means:
- the Rust backend boots on Code Engine
- PostgreSQL migrations apply cleanly
- login and session resolution work
- booking, execution, chat, payments, TMS, and admin ops work against hosted services
- protected document uploads and reads work against IBM Cloud Object Storage

## Preconditions

Before running this checklist, make sure all of these are true:
- Code Engine app is deployed and healthy
- `RUN_MIGRATIONS=true` has been used at least once successfully
- IBM PostgreSQL is reachable from Code Engine
- IBM Cloud Object Storage bucket exists and the app has valid credentials
- smoke seed SQL has been applied to the staging database
- browser-side `GOOGLE_MAPS_API_KEY` is configured if you want to validate load builder address autocomplete

## Required Files

- `rust-port/.env.ibm.example`
- `rust-port/docs/IBM_CODE_ENGINE_DEPLOYMENT.md`
- `rust-port/docs/PHP_RUST_SIDE_BY_SIDE_QA.md`
- `rust-port/scripts/seed_postgres_smoke_data.sql`
- `rust-port/scripts/smoke_test_backend.ps1`
- `rust-port/scripts/verify_backend_cutover_hosted.ps1`

## Step 1. Confirm App Health

Run:

```powershell
Invoke-RestMethod https://YOUR-CODE-ENGINE-URL/health
```

Pass criteria:
- `status` is `ok`
- `deployment_target` is `ibm-code-engine`
- `database_state` is not reporting a missing database connection

## Step 2. Seed The Disposable Staging Dataset

Run from your workstation:

```powershell
psql "YOUR_DATABASE_URL" -f "rust-port\scripts\seed_postgres_smoke_data.sql"
```

Pass criteria:
- seed script finishes without PostgreSQL errors
- smoke users, seeded load leg, offers, handoff rows, and sync issue rows exist

## Step 3. Run The Automated API Smoke Pass

Run:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\smoke_test_backend.ps1" -BaseUrl "https://YOUR-CODE-ENGINE-URL"
```

The current script validates:
- `/health`
- login and `/auth/session`
- shipper and carrier load board reads
- carrier booking on a seeded leg
- execution lifecycle on the booked leg
- GPS location pings during execution
- escrow fund, hold, and release
- Stripe webhook ingestion
- chat send and read
- offer review
- admin STLOADS operations and reconciliation reads
- sync-error resolve
- TMS withdraw, close, push, status webhook, requeue, cancel, and close flows

Pass criteria:
- script exits cleanly
- final JSON summary shows `result: ok`

Optional hosted bundle:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\verify_backend_cutover_hosted.ps1"
```

The bundle script reseeds the disposable staging dataset, runs the core API smoke pass, reruns the hosted Rust role matrix, SMTP validation, TMS worker validation, and hosted Stripe release verification, then emits one final `result: ok` summary for the backend-only cutover gate.
The hosted role matrix now also refreshes the disposable lifecycle-state QA accounts before validation and checks frontend routes with a browser-style HTML `Accept` header so the IBM frontend proxy rules do not create false `404` failures for SPA pages like `/admin/account-lifecycle`.

## Step 4. Run Manual Browser Validation

The script is API-first. These browser checks still matter.

### Auth and onboarding

- register a new non-admin user from the Rust auth flow
- verify OTP
- continue onboarding without losing session continuity
- upload KYC from the Rust onboarding screen
- confirm admin can review that KYC file

### Load builder and profile

- create a new load in Rust using Google address autocomplete
- open the Rust load profile
- upload at least one load document
- confirm the uploader can open the document
- confirm a different non-admin non-uploader cannot open it
- confirm admin can open it

### Execution

- open `/execution/legs/{leg_id}` from the Rust load profile
- verify action sequence is gated correctly
- send GPS from the browser if device geolocation is available
- verify realtime refresh updates the page after execution actions

### Admin ops

- open STLOADS operations, reconciliation, onboarding review, master-data, and payments pages
- confirm page loads are real backend/API driven, not sample-backed

## Step 5. Confirm IBM Cloud Object Storage Behavior

Because Code Engine is stateless, object storage validation is mandatory.

Check:
- newly uploaded document keys are written under the configured prefix
- object reads work through the protected Rust route
- access still respects uploader-plus-admin visibility rules
- no local-filesystem dependency is required in staging

## Step 6. Capture Findings

Record the following after the pass:
- Code Engine URL used
- PostgreSQL instance name
- COS bucket name and prefix
- smoke script result
- any manual browser issues
- any env vars that needed clarification
- any route that still depends on Laravel

## What To Fix Immediately If Found

Treat these as stop-ship staging failures:
- migrations fail on IBM PostgreSQL
- document upload works locally but not on IBM COS
- session cookies or bearer-token resolution break behind Code Engine
- protected documents are accessible by unauthorized users
- execution actions or GPS pings fail for the seeded leg
- admin pages still depend on sample payloads or Laravel routes

## Next Step After A Clean Pass

Once this checklist is green:
1. set `RUN_MIGRATIONS=false`
2. redeploy the backend
3. run `docs/PHP_RUST_SIDE_BY_SIDE_QA.md` with real operator roles
4. fix every P0/P1 finding before any production cutover
