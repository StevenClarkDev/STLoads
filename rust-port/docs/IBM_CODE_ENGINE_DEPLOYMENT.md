# IBM Code Engine Deployment Guide

This is the simplest first deployment path for the current Rust port.

## What We Deploy Right Now

Today, the deployable Rust workload is the Axum backend API in `crates/backend`.

That means this guide gets you to a live backend with:
- auth/session routes
- onboarding and KYC routes
- load board APIs
- load builder and load profile APIs
- tracking and execution APIs
- chat and marketplace APIs
- payments APIs
- STLOADS/TMS APIs
- admin ops APIs
- realtime websocket endpoint
- protected document upload and download routed through IBM Cloud Object Storage-compatible storage

The Leptos frontend crate is not yet packaged as its own deployable IBM workload, so the safest first IBM milestone is: deploy the backend first, point it at IBM PostgreSQL and IBM Cloud Object Storage, then run the smoke script and staging checklist against the live Code Engine URL.

## Files Added For IBM

- `Dockerfile`: backend container image for Code Engine.
- `.dockerignore`: keeps local build contexts small.
- `.ceignore`: keeps Code Engine local-source uploads small.
- `.env.ibm.example`: runtime variable template for IBM.
- `scripts/seed_postgres_smoke_data.sql`: disposable PostgreSQL smoke dataset.
- `scripts/smoke_test_backend.ps1`: end-to-end backend smoke run, now including execution lifecycle checks.
- `docs/IBM_STAGING_SMOKE_CHECKLIST.md`: hosted validation checklist for PostgreSQL plus IBM COS.

## Recommended First Path

Use Code Engine local-source deployment from your workstation.

Why this is the easiest first path:
- you do not need to learn Rust-specific IBM packaging first
- you do not need Docker for the first deploy
- Code Engine can build the image from the local `rust-port` folder and store it in IBM Container Registry automatically
- the same Dockerfile can still be reused later for CI/CD

## Prerequisites

1. IBM Cloud account.
2. Pay-as-you-go billing enabled for Code Engine.
3. IBM Cloud CLI installed.
4. Code Engine plugin installed.
5. IBM PostgreSQL instance created and reachable.
6. IBM Cloud Object Storage bucket created for document uploads.
7. `psql` available locally for the seed step.

Optional now, useful later:
- Docker Desktop if you want to build or test the image locally.
- A real domain and TLS cert if you want `api.yourdomain.com` instead of the default Code Engine URL.

## Step 1: Install The CLI Plugin

```powershell
ibmcloud plugin install code-engine -f
```

If you later decide to push container images yourself, also install Container Registry support:

```powershell
ibmcloud plugin install container-registry -f
```

## Step 2: Log In And Target A Region

`us-south` is the easiest region to start with unless you already use another IBM region.

```powershell
ibmcloud login --sso
ibmcloud target -r us-south -g Default
```

Replace `Default` with your real resource group if needed.

## Step 3: Create And Select A Code Engine Project

A project is the deployment boundary for apps, jobs, secrets, and domain mappings.

```powershell
ibmcloud ce project create --name stloads-rust-staging
ibmcloud ce project select --name stloads-rust-staging
ibmcloud ce project get --name stloads-rust-staging
```

## Step 4: Prepare The Runtime Env File

Copy the template and fill in the real values.

```powershell
Copy-Item rust-port\.env.ibm.example rust-port\.env.ibm.runtime
```

Edit `rust-port\.env.ibm.runtime` and set at minimum:
- `DATABASE_URL`
- `PUBLIC_BASE_URL`
- `APP_ENV`
- `RUN_MIGRATIONS`
- `DOCUMENT_STORAGE_BACKEND`
- `OBJECT_STORAGE_BUCKET`
- `OBJECT_STORAGE_REGION`
- `OBJECT_STORAGE_ENDPOINT`
- `OBJECT_STORAGE_ACCESS_KEY_ID`
- `OBJECT_STORAGE_SECRET_ACCESS_KEY`
- `OBJECT_STORAGE_FORCE_PATH_STYLE`
- `OBJECT_STORAGE_PREFIX`
- `STRIPE_WEBHOOK_SHARED_SECRET`
- `TMS_SHARED_SECRET`

For the very first deploy:
- set `RUN_MIGRATIONS=true`
- keep `PORT=8080`
- keep `DEPLOYMENT_TARGET=ibm-code-engine`
- set `DOCUMENT_STORAGE_BACKEND=ibm_cos`
- point the `OBJECT_STORAGE_*` values at the IBM Cloud Object Storage bucket for this environment

Example PostgreSQL DSN shape:

```text
postgres://USERNAME:PASSWORD@HOST:PORT/DATABASE?sslmode=require
```

Example IBM COS endpoint shape:

```text
https://s3.us-south.cloud-object-storage.appdomain.cloud
```

## Step 5: Create A Runtime Secret In Code Engine

This stores your environment values inside the project.

```powershell
ibmcloud ce secret create --name stloads-rust-runtime --from-env-file .\rust-port\.env.ibm.runtime
ibmcloud ce secret get --name stloads-rust-runtime
```

If the secret already exists, update it by deleting and recreating it:

```powershell
ibmcloud ce secret delete --name stloads-rust-runtime -f
ibmcloud ce secret create --name stloads-rust-runtime --from-env-file .\rust-port\.env.ibm.runtime
```

## Step 6: Deploy From Local Source

Run this from the repo root (`e:\Projects\STLoads`).

```powershell
ibmcloud ce app create `
  --name stloads-rust-backend `
  --build-source .\rust-port `
  --build-dockerfile Dockerfile `
  --env-from-secret stloads-rust-runtime `
  --port 8080 `
  --cpu 1 `
  --memory 2G `
  --min-scale 1 `
  --max-scale 2 `
  --request-timeout 600
```

Notes:
- `--build-source .\rust-port` tells Code Engine to upload the Rust workspace from your local machine.
- `--build-dockerfile Dockerfile` tells Code Engine to use the Dockerfile in `rust-port`.
- `--request-timeout 600` is important because this app includes websocket and realtime behavior and Code Engine caps app connections at 10 minutes.
- `--min-scale 1` avoids cold-start pain during the first admin, chat, document, and execution testing pass.

## Step 7: Get The Live URL And Logs

```powershell
ibmcloud ce app get --name stloads-rust-backend
ibmcloud ce app logs --name stloads-rust-backend --follow
```

The `app get` output includes the default Code Engine public URL.

## Step 8: Seed IBM PostgreSQL And Run The Hosted Smoke Pass

After the app is up, seed the database from your workstation.

```powershell
psql "YOUR_DATABASE_URL" -f "rust-port\scripts\seed_postgres_smoke_data.sql"
```

Then run the smoke test against the live Code Engine URL:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\smoke_test_backend.ps1" -BaseUrl "https://YOUR-CODE-ENGINE-URL"
```

Important notes:
- the smoke script now validates execution lifecycle and GPS ping endpoints in addition to auth, board, chat, payments, and TMS flows
- document upload, protected file reads, onboarding KYC, and Google-address browser behavior should still be checked manually in the browser
- use `docs/IBM_STAGING_SMOKE_CHECKLIST.md` as the hosted signoff checklist

## Step 9: Turn Off Startup Migrations After The First Healthy Deploy

After the first successful deployment and smoke pass:

1. Edit `rust-port\.env.ibm.runtime`
2. change `RUN_MIGRATIONS=true` to `RUN_MIGRATIONS=false`
3. recreate the secret
4. update the app

```powershell
ibmcloud ce secret delete --name stloads-rust-runtime -f
ibmcloud ce secret create --name stloads-rust-runtime --from-env-file .\rust-port\.env.ibm.runtime

ibmcloud ce app update `
  --name stloads-rust-backend `
  --build-source .\rust-port `
  --build-dockerfile Dockerfile `
  --env-from-secret stloads-rust-runtime `
  --port 8080 `
  --cpu 1 `
  --memory 2G `
  --min-scale 1 `
  --max-scale 2 `
  --request-timeout 600
```

## Step 10: Custom Domain Later

When the backend is stable, map your real domain.

1. Obtain a publicly trusted TLS certificate and private key.
2. Create a TLS secret:

```powershell
ibmcloud ce secret create --name stloads-api-tls --format tls --cert-chain-file .\tls\fullchain.pem --private-key-file .\tls\privkey.pem
```

3. Create the domain mapping:

```powershell
ibmcloud ce domainmapping create --domain-name api.example.com --target stloads-rust-backend --tls-secret stloads-api-tls
```

4. Add the CNAME record that Code Engine gives you at your DNS provider.

## Updating The App Later

Every time you want to redeploy new backend code:

```powershell
ibmcloud ce app update `
  --name stloads-rust-backend `
  --build-source .\rust-port `
  --build-dockerfile Dockerfile `
  --env-from-secret stloads-rust-runtime `
  --port 8080 `
  --cpu 1 `
  --memory 2G `
  --min-scale 1 `
  --max-scale 2 `
  --request-timeout 600
```

## If You Prefer Docker Later

The included `Dockerfile` also supports a manual image workflow.

That path is useful later for CI/CD, but for the first IBM deployment the local-source Code Engine flow is simpler and less error-prone.

## Important Current Limitation

The backend is ready for this first IBM deployment path.

The frontend is not yet packaged as its own finalized IBM workload, so treat this as:
- backend deployment first
- IBM PostgreSQL plus IBM Cloud Object Storage validation second
- frontend hosting and cutover after that
