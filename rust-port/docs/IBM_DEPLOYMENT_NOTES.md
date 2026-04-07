# IBM Deployment Notes

These notes capture the deployment assumptions being baked into the Rust port for IBM-hosted infrastructure.

## Current Runtime Shape

- `backend` binds with `BIND_ADDR` and `PORT` instead of assuming localhost-only development.
- Default bind address is `0.0.0.0` so the app can sit behind IBM ingress and reverse proxies without code changes.
- Startup is environment-driven through:
  - `DEPLOYMENT_TARGET`
  - `APP_ENV`
  - `BIND_ADDR`
  - `PORT`
  - `DATABASE_URL`
  - `RUN_MIGRATIONS`
  - `PUBLIC_BASE_URL`
  - `STRIPE_WEBHOOK_SHARED_SECRET`
  - `TMS_SHARED_SECRET`
- `/health` reports deployment target, environment, public base URL, and database connectivity state.

## Database Target

- The target production database is PostgreSQL on IBM-hosted infrastructure.
- `DATABASE_URL` should point to a PostgreSQL DSN.
- The Rust workspace now compiles and verifies against PostgreSQL SQLx support.
- The next live checkpoint is not more SQL dialect cleanup; it is running the app end-to-end against a real IBM-style PostgreSQL instance.

## Deployment Assets Now Present

- `Dockerfile` for the Rust backend.
- `.dockerignore` for local container builds.
- `.ceignore` for Code Engine local-source deployments.
- `.env.ibm.example` as the starter runtime contract.
- `docs/IBM_CODE_ENGINE_DEPLOYMENT.md` as the beginner-first runbook.
- Smoke tooling:
  - `scripts/seed_postgres_smoke_data.sql`
  - `scripts/smoke_test_backend.ps1`

## Deployment Principles

- Keep the Rust services stateless.
  - Do not rely on local disk for durable uploads, documents, or reconciliation artifacts.
  - Plan object storage behind a storage abstraction before full cutover.
- Assume TLS terminates upstream.
  - The backend should remain reverse-proxy friendly and not require direct public TLS termination in-process.
- Prefer environment configuration over machine-specific paths or hard-coded hostnames.
- Treat database connectivity as optional during boot.
  - If `DATABASE_URL` is missing or temporarily unavailable, the backend still starts and exposes health plus fallback screen data.
- Treat migrations as operator-controlled.
  - `RUN_MIGRATIONS=true` is explicit so production startup stays predictable.
- Treat test databases as disposable.
  - Integration tests should run against a dedicated PostgreSQL test database, never the production IBM database.

## Known IBM Runtime Considerations

- Code Engine is a strong fit for the current backend API.
- The websocket/realtime surface works on Code Engine, but client connections must tolerate reconnects because Code Engine caps app connections at 10 minutes.
- The current first IBM milestone is backend deployment first, frontend cutover second.

## Near-Term Follow-Up

- Add object storage configuration for uploaded documents and generated exports.
- Add structured logging and request correlation suitable for IBM-hosted observability pipelines.
- Add readiness/liveness split once more write-heavy production flows are online.
- Package the frontend as its own IBM-ready workload or fold it into a final SSR hosting strategy.

## Constraint For Ongoing Work

Every new backend and frontend slice should assume IBM-hosted deployment from the start:

- no localhost-only assumptions
- no hard-coded filesystem dependencies
- no direct coupling to Laravel-era asset hosting patterns
- explicit env-driven configuration for ports, URLs, credentials, and runtime behavior
- PostgreSQL as the target durable relational store
