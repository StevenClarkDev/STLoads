# Enterprise Release Environments

This document defines the STLoads Rust release environments for `ENT-0104`. It is the source of truth for runtime shape, required secrets, and deployment validation expectations.

## Environment Matrix

| Environment | Purpose | Data | Deploy Target | Required Strictness |
| --- | --- | --- | --- | --- |
| Local | Developer iteration and unit checks | Disposable local or env-gated test data | Developer workstation | Permissive defaults allowed |
| CI | Automated formatting, tests, builds, and static checks | No production data | CI runner | Secrets should not be required except explicit integration jobs |
| Staging | Production-like verification before release | Dedicated staging PostgreSQL and object-storage bucket | IBM Code Engine staging app | Must mirror production shape with test credentials |
| Enterprise Pilot | Limited customer/UAT rollout before full production | Pilot tenant data only | Isolated pilot deployment or production with guarded pilot flags | Must use production-like security and observability |
| Production | Live enterprise customer operations | Live customer data | IBM Code Engine production app | Strict config, no placeholders, no local storage, no startup migrations |

## Required Runtime Values

Every staging, pilot, and production runtime file must define:

- `APP_ENV`
- `DEPLOYMENT_TARGET`
- `BIND_ADDR`
- `PORT`
- `PUBLIC_BASE_URL`
- `CORS_ALLOWED_ORIGINS`
- `RUN_MIGRATIONS=false`
- `DATABASE_URL`
- `STLOADS_DATABASE_SCHEMA`
- `DOCUMENT_STORAGE_BACKEND`
- `OBJECT_STORAGE_BUCKET`
- `OBJECT_STORAGE_REGION`
- `OBJECT_STORAGE_ENDPOINT`
- `OBJECT_STORAGE_ACCESS_KEY_ID`
- `OBJECT_STORAGE_SECRET_ACCESS_KEY`
- `OBJECT_STORAGE_FORCE_PATH_STYLE`
- `OBJECT_STORAGE_PREFIX`
- `STRIPE_SECRET`
- `STRIPE_WEBHOOK_SECRET_PLATFORM`
- `STRIPE_WEBHOOK_SECRET_CONNECT`
- `STRIPE_CONNECT_REFRESH_URL`
- `STRIPE_CONNECT_RETURN_URL`
- `TMS_SHARED_SECRET`
- `TMS_RETRY_WORKER_ENABLED`
- `TMS_RECONCILIATION_WORKER_ENABLED`
- `MAIL_MAILER`
- `MAIL_HOST`
- `MAIL_FROM_ADDRESS`
- `MAIL_FAIL_OPEN`
- `PORTAL_URL`
- `FRONTEND_PUBLIC_URL`
- `KILL_SWITCH_PAYMENTS`
- `KILL_SWITCH_BOOKING`
- `KILL_SWITCH_TMS_PUSHES`
- `KILL_SWITCH_NOTIFICATIONS`
- `KILL_SWITCH_DOCUMENT_UPLOADS`

## Environment Rules

### Local

- `APP_ENV=development`.
- `DATABASE_URL` may be omitted for frontend/API shape work.
- `DOCUMENT_STORAGE_BACKEND=local` is allowed.
- `MAIL_MAILER=log` and `MAIL_FAIL_OPEN=true` are allowed.
- Secrets must stay in ignored local files.

### CI

- CI must run without live customer secrets by default.
- DB integration jobs may use disposable `RUST_TEST_DATABASE_URL`.
- Production runtime secrets must not be required for normal unit tests.
- CI should run the env validator against templates with placeholder allowance.

### Staging

- `APP_ENV=staging`.
- Must use PostgreSQL, object storage, Stripe test-mode credentials, SMTP test/provider credentials, and TMS shared secret.
- Must keep `RUN_MIGRATIONS=false` in the web runtime.
- Migrations must run through `cargo run -p backend --bin run_migrations`.
- Staging should fail validation if it uses local document storage or permissive CORS.

### Enterprise Pilot

- `APP_ENV=pilot` or `APP_ENV=production` with pilot tenants isolated by flags and tenant controls.
- Must use production-like database, object storage, SMTP, Stripe, TMS, CORS, and public URL settings.
- Must not use production customer data unless the pilot is contractually approved.
- Must have support, onboarding, rollback, and incident owners assigned before launch.

### Production

- `APP_ENV=production`.
- Must not use placeholders, local document storage, permissive CORS, log-only mailer, or fail-open mail.
- Must keep `RUN_MIGRATIONS=false`.
- Must pass `/health/ready` before traffic promotion.
- Must have rollback owner, migration owner, support owner, and incident owner identified for every release.

## Validation

Use the runtime environment validator before deployment:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\validate_runtime_env.ps1" -EnvFile "rust-port\.env.ibm.runtime" -TargetEnvironment staging
```

For templates only:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\validate_runtime_env.ps1" -EnvFile "rust-port\.env.ibm.example" -TargetEnvironment staging -AllowPlaceholders
```

Validation must be run before every staging, pilot, and production deployment.

## Drift Signals

Treat any of these as release blockers:

- Runtime file has placeholder values for a real deployment.
- `RUN_MIGRATIONS=true` appears in a web runtime file.
- Production or staging uses `DOCUMENT_STORAGE_BACKEND=local`.
- Production has empty or wildcard CORS.
- Production has `MAIL_MAILER=log` or `MAIL_FAIL_OPEN=true`.
- Staging does not use the same dependency classes as production.
- Frontend and backend public URLs do not match the intended environment.

## Task Mapping

- `ENT-0104` defines this environment model and validator.
- `ENT-0105` owns rollback.
- `ENT-0106` owns feature flags and change approval.
- `ENT-0106A` owns release notes and UAT rollout.
- `ENT-0107` owns production data migration and cutover.
