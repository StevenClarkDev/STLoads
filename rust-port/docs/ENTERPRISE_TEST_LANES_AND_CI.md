# Enterprise Test Lanes And CI

Last updated: 2026-05-27

This document is the Phase 16 quality-gate contract for STLoads. It defines which checks developers run locally, which checks block merge in CI, and which checks are hosted or environment-dependent.

## Lane Summary

| Lane | Owner | Merge blocking | Command |
| --- | --- | --- | --- |
| Fast Rust lane | Engineering | Yes | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_ci_fast.ps1` |
| Backend integration lane | Backend/QA | Yes when database is configured | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_ci_backend_integration.ps1` |
| Frontend release lane | Frontend | Yes | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_ci_frontend_release.ps1` |
| Browser E2E lane | Frontend/QA | Yes | `npx playwright test` |
| Security lane | Security/DevOps | Yes | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_ci_security.ps1` |
| Docker image lane | DevOps | Yes | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_ci_docker_build.ps1` |
| Hosted smoke lane | DevOps/Ops | Required before release promotion | `scripts/verify_backend_cutover_hosted.ps1`, `scripts/verify_smtp_hosted.ps1`, `scripts/verify_stripe_hosted.ps1`, and `scripts/verify_tms_workers_hosted.ps1` |
| Performance smoke lane | QA/Backend | Required before enterprise pilot and major performance-impacting releases | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_performance_smoke.ps1 -BaseUrl <url>` |

## Fast Rust Lane

Purpose: catch normal regressions before a branch leaves a developer machine.

The lane runs:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p domain`
- `cargo test -p shared`

This lane is intentionally smaller than full workspace tests so it stays fast. Domain state-machine tests live here because invalid lifecycle transitions are enterprise P0 failures.

## Backend Integration Lane

Purpose: prove database-backed workflows and migrations still work.

The lane runs:

- `cargo test --workspace`
- `cargo sqlx prepare --workspace --check` when `DATABASE_URL` is configured

Local database tests use `RUST_TEST_DATABASE_URL` or `TEST_DATABASE_URL`. CI uses `CI_DATABASE_URL` as a GitHub secret for SQLx metadata checks. If the database URL is not configured, CI records a clear skip instead of pretending SQLx was verified.

The previous all-in-one workspace path is now split into separate CI jobs for Rust quality, SQLx, frontend release/E2E, security, and Docker. That keeps long checks visible by lane and prevents a single opaque timeout from hiding the failing surface.

## Frontend Release Lane

Purpose: block broken Leptos WASM release artifacts.

The lane pins:

- Rust target: `wasm32-unknown-unknown`
- Trunk: `0.21.14`

The lane runs `trunk build --release` from `crates/frontend-leptos`. Release artifacts are not committed as the deploy source of truth; CI only proves the intended deployment build can be produced.

## Browser E2E Lane

Purpose: catch customer-visible workflow regressions.

The lane runs Playwright against the Leptos dev server defined in `playwright.config.ts`. It covers desktop and mobile Chromium viewports. Browser failures block merge.

## Security Lane

Purpose: catch known vulnerable dependencies and accidental credentials.

The lane runs:

- `cargo audit`
- PowerShell/ripgrep secret scan for common cloud keys, private keys, Stripe live keys, Slack tokens, and generic committed secret assignments
- `npm audit --audit-level=high`

Allowed example-secret references are limited to documented template files. Real runtime files such as `.env.ibm.secret`, `.env.ibm.runtime`, `.cos-*`, and private keys must remain ignored and untracked.

## Docker Image Lane

Purpose: prove production image build inputs are complete.

The lane builds:

- `Dockerfile` as `stloads-backend:ci`
- `Dockerfile.frontend` as `stloads-frontend:ci`

The backend Dockerfile builds all backend package binaries so `backend` and `run_migrations` are both present in the runtime image. The frontend Dockerfile uses the same pinned Trunk version as CI.

## Hosted Smoke Lane

Purpose: prove deployed services can talk to real configured dependencies.

Hosted smoke checks are not run on every pull request because they require deployment credentials and live provider access. They must run before release promotion and after any change that touches runtime config, storage, SMTP, Stripe, TMS workers, migration behavior, or Code Engine routing.

Required scripts:

- `scripts/verify_backend_cutover_hosted.ps1`
- `scripts/verify_smtp_hosted.ps1`
- `scripts/verify_stripe_hosted.ps1`
- `scripts/verify_tms_workers_hosted.ps1`

## Performance Smoke Lane

Purpose: keep performance risks visible before enterprise pilot.

The repository-level smoke script currently verifies live endpoint latency. Database-backed performance expectations are tracked in `query_performance_controls`, including load board search, chat, tracking writes, admin queues, TMS reconciliation, global search, and reporting scorecards. Enterprise pilot readiness requires representative data volume and stored evidence for those query plans.

## CI Workflow

The merge-blocking workflow lives at `.github/workflows/ci.yml` and contains these jobs:

- `rust-quality`
- `sqlx-check`
- `frontend-release`
- `security`
- `docker-build`

Required branch protection should make all jobs required for protected branches. The SQLx job is required as a job but skips the metadata command with an explicit message when `CI_DATABASE_URL` is unavailable.

## Local Batch Command

For broad local verification after a completed phase:

```powershell
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
Push-Location crates/frontend-leptos; trunk build --release; Pop-Location
npx playwright test
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/frontend_page_inventory.ps1
```

Add Docker and hosted smoke checks when the change touches build images or deployed runtime behavior.
