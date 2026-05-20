# STLoads Production Runbook

## Purpose

This runbook gives operators the minimum reliable procedure set for running STLoads in production or staging. STLoads is a standalone Rust and Leptos load-board product with its own backend, frontend, middleware, database, Stripe flow, document storage, and IBM Code Engine deployment path.

## Production Surfaces

- Backend health: `https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud/health`
- Backend readiness: `https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud/readiness`
- Frontend: `https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud/`
- IBM Code Engine project: `stloads-rust-staging`
- Source repository: `https://github.com/StevenClarkDev/STLoads.git`
- Working branch: `codex/production-readiness-lock`

## Deploy Process

1. Confirm the repository is clean.
2. Run the local production readiness gate from `rust-port`.
3. Run a deployment dry run with `scripts/deploy_code_engine_staging.ps1 -WhatIf`.
4. Deploy backend and frontend to the `stloads-rust-staging` Code Engine project.
5. Confirm both Code Engine apps report `Application deployed successfully`.
6. Smoke test backend health, backend readiness, payments health, dispatch health, admin health, and frontend root.
7. Record the backend and frontend revision numbers in the release notes or work log.

Canonical deployment command:

```powershell
cd "C:\New folder\STLoads-api-review\rust-port"
powershell -ExecutionPolicy Bypass -File ".\scripts\deploy_code_engine_staging.ps1" `
  -ProjectName "stloads-rust-staging" `
  -RuntimeSecretName "stloads-rust-runtime" `
  -BackendAppName "stloads-rust-backend" `
  -FrontendAppName "stloads-rust-frontend" `
  -BackendOutboundBaseUrl "https://dispatch-api.268io0zej89v.us-south.codeengine.appdomain.cloud" `
  -FrontendPublicUrl "https://portal.stloads.com" `
  -GoogleMapsApiKey "<browser-key>"
```

## Rollback Process

1. Identify the last healthy backend and frontend revisions from Code Engine.
2. Set traffic back to the last healthy revision.
3. Confirm health and readiness return HTTP 200.
4. Run the hosted smoke checks.
5. Document the failed revision, symptom, and rollback time.

Useful inspection commands:

```powershell
ibmcloud ce app get -n stloads-rust-backend
ibmcloud ce app get -n stloads-rust-frontend
ibmcloud ce revision list --app stloads-rust-backend
ibmcloud ce revision list --app stloads-rust-frontend
```

## Incident Response

Severity levels:

- SEV1: marketplace unavailable, login unavailable, payments broken, or data isolation suspected.
- SEV2: core workflow degraded, document upload/read failure, dispatch sync delayed, or booking stuck.
- SEV3: single-user issue, cosmetic defect, slow page, or non-critical support issue.

Initial response:

1. Check `/health` and `/readiness`.
2. Check Code Engine app status and latest logs.
3. Confirm database connectivity and runtime secret availability.
4. Confirm Stripe webhook health for payment incidents.
5. Confirm object storage access for document incidents.
6. Freeze deployments until the incident is triaged.
7. Capture affected tenant, user, load, booking, payment, and document IDs.

## Queue Replay

Use queue replay only for idempotent events or events with a clear deduplication key.

1. Identify the failed queue family: dispatch sync, document event, notification, payment reconciliation, or marketplace booking.
2. Confirm the original event payload and tenant scope.
3. Confirm no successful downstream write already exists.
4. Replay one event first.
5. Confirm audit and downstream state.
6. Replay the remaining batch in small groups.

Never replay a payment or booking event without checking the current database state and Stripe state first.

## DLQ Recovery

1. Export the dead-letter entries before modification.
2. Group failures by error type.
3. Fix configuration or data problems before replay.
4. Replay deterministic validation failures only after correcting the source record.
5. For poison messages, preserve the payload and mark the work item manually resolved with an audit note.

Required audit fields:

- tenant ID
- source event ID
- operator
- recovery action
- before state
- after state
- timestamp

## Stripe Webhook Recovery

1. Confirm the webhook endpoint and signing secret are configured in Code Engine runtime secrets.
2. Find the Stripe event in the Stripe dashboard.
3. Confirm whether STLoads already recorded the event.
4. If missing, resend the event from Stripe or run the approved hosted webhook recovery script.
5. Confirm escrow, payment intent, transfer, and booking state are consistent.
6. Add an audit note with Stripe event ID and operator.

Do not manually mark payments funded or released unless Stripe state proves the money movement.

## Object Storage Recovery

1. Confirm the document metadata row exists.
2. Confirm object key, bucket, tenant, owner, and document type.
3. Check whether the object exists in storage.
4. If metadata exists but object is missing, mark the document as recovery required and ask the user to re-upload.
5. If object exists but metadata is missing, restore metadata only when tenant and owner are proven.
6. Re-run protected document access checks after recovery.

## Required Post-Deploy Smoke Checks

```powershell
$backend = "https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud"
$frontend = "https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud"
Invoke-WebRequest "$backend/health" -UseBasicParsing
Invoke-WebRequest "$backend/readiness" -UseBasicParsing
Invoke-WebRequest "$backend/payments/health" -UseBasicParsing
Invoke-WebRequest "$backend/dispatch/health" -UseBasicParsing
Invoke-WebRequest "$backend/admin/health" -UseBasicParsing
Invoke-WebRequest "$frontend/" -UseBasicParsing
```

## Release Signoff

A release is ready to hand to partners only when:

- health and readiness pass
- frontend root loads
- production readiness gate passes or skipped checks are documented
- Code Engine revisions are recorded
- no demo load, fake payment, fake carrier, or placeholder production path is present
- rollback path is known

