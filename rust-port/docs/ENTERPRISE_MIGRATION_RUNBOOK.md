# Enterprise Migration Runbook

This runbook defines the operator-controlled migration path for the Rust STLoads backend. It is the operating contract for `ENT-0103`.

## Rule

The web runtime must not apply database migrations during startup.

Migrations are applied through an explicit operator command before web traffic is promoted.

## Command

Run migrations from the Rust workspace with the target database environment loaded:

```powershell
cd rust-port
$env:DATABASE_URL = "postgres://USER:PASSWORD@HOST:PORT/DB?sslmode=require"
$env:STLOADS_DATABASE_SCHEMA = "stloads"
cargo run -p backend --bin run_migrations
```

For IBM Code Engine, run the same binary as a one-off migration job or from a trusted operator workstation that can reach the target PostgreSQL service.

## Pre-Migration Checklist

- Confirm the target environment: staging, pilot, or production.
- Confirm the exact `DATABASE_URL` and schema.
- Confirm the web runtime uses `RUN_MIGRATIONS=false`.
- Take a database snapshot or provider backup.
- Record the current deployed backend image/revision.
- Review pending SQL files in `crates/db/migrations`.
- Confirm no unapproved destructive schema changes are included.
- Confirm the release owner, database owner, and rollback owner are online.

## Migration Steps

1. Put the release in a change window if the migration touches large tables, payments, auth, tenant isolation, documents, or integrations.
2. Run the migration command against staging first.
3. Run smoke tests against staging.
4. Run the migration command against production only after staging passes.
5. Verify the command exits successfully and prints `database migrations completed`.
6. Start or update the web runtime with `RUN_MIGRATIONS=false`.
7. Check `/health/ready`.
8. Run production smoke checks approved for the release window.
9. Record the migration result in the release notes and enterprise work board.

## Failed Migration Procedure

- Stop the deployment promotion immediately.
- Keep the previous web revision serving traffic if it is still healthy.
- Capture the migration error output, target database, schema, migration file, and timestamp.
- Do not rerun blindly.
- Determine whether the failure happened before or after any schema mutation.
- If no mutation happened, fix the migration or environment and rerun through staging first.
- If a partial mutation happened, decide between forward-fix SQL and database restore.
- For production restore, use the latest verified database snapshot and keep the previous web revision active until restore is complete.
- After recovery, rerun `/health/ready` and the release smoke tests.

## Rollback Strategy

The migration files in this slice are forward SQL migrations. Treat rollback as an operational recovery process:

- Prefer keeping the previous web revision live while the migration issue is investigated.
- Use database snapshots for destructive or incompatible migration rollback.
- Use forward-fix migrations for additive migration mistakes when data is safe and the release owner approves.
- Never deploy application code that expects a failed migration to have succeeded.

## Verification Checklist

- `RUN_MIGRATIONS=false` in the web runtime env.
- Migration command succeeds in staging before production.
- Production migration command output is saved.
- `/health/ready` returns 200 after deploy promotion.
- Smoke tests pass after migration and deploy.
- Release notes include migration version, operator, timestamp, and verification results.
