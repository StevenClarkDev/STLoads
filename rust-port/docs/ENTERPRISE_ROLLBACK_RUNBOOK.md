# Enterprise Deployment Rollback Runbook

This runbook defines the recovery path for failed STLoads Rust deployments. It is the operating document for `ENT-0105`.

## Rule

Rollback is a release operation, not an improvised incident response.

Every release must identify:

- Release owner.
- Backend rollback owner.
- Frontend rollback owner.
- Database/migration owner.
- Object-storage owner.
- Support/customer-communication owner.
- Decision point for rollback versus forward fix.

## Rollback Decision Triggers

Rollback or stop promotion when any of these occur:

- `/health/ready` fails after deploy.
- Login, load posting, booking, tracking, document upload/view, payments, TMS handoff, or notification smoke checks fail.
- Cross-tenant data exposure is suspected.
- Duplicate booking, duplicate payout, or incorrect financial state is suspected.
- A migration fails or produces unexpected schema/data drift.
- Object-storage reads/writes fail for existing documents.
- Customer-facing severe incident is detected during or immediately after release.

## Backend Rollback

Preferred path on IBM Code Engine:

1. Identify the last known-good app revision.
2. Stop traffic promotion to the new revision.
3. Re-point the app to the last known-good image/source revision using the approved Code Engine procedure.
4. Keep the same runtime secret only if the secret is compatible with the previous revision.
5. If the runtime secret changed, restore the previous known-good secret version or recreate it from the approved release record.
6. Verify `/health/live`.
7. Verify `/health/ready`.
8. Run the backend smoke script.
9. Watch logs for payment, TMS, auth, document, and database errors.

Required verification:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\smoke_test_backend.ps1" -BaseUrl "https://YOUR-CODE-ENGINE-URL"
```

## Frontend Rollback

1. Identify the last known-good frontend revision/build.
2. Restore the previous frontend deployment or container image.
3. Confirm `BACKEND_API_BASE_URL`, `BACKEND_UPSTREAM`, and `FRONTEND_PUBLIC_URL` still match the target backend.
4. Verify login, role-based navigation, load board, dispatch flow, document links, payments screens, and admin screens.
5. Confirm browser console has no release-blocking errors.

Frontend rollback must not point customers at a backend schema/API version that is incompatible with the restored UI.

## Database Rollback Or Forward Fix

The Rust migration strategy is forward migration plus operational recovery.

Use this decision model:

- If the migration never mutated schema/data, fix and rerun through staging first.
- If the migration added safe additive structures, prefer a forward-fix migration when approved.
- If the migration performed destructive or incompatible changes, restore from the latest verified database snapshot.
- If production traffic is already using partially migrated data, stop promotion and involve the database owner before changing anything else.

Never deploy application code that assumes a failed migration succeeded.

Use `docs/ENTERPRISE_MIGRATION_RUNBOOK.md` for migration-specific failed-run procedure.

## Object Storage Rollback

Object storage rollback is not the same as application rollback.

1. Confirm whether new release wrote, moved, deleted, or re-keyed objects.
2. Preserve object keys, metadata, and audit evidence before remediation.
3. If reads fail but objects still exist, prefer restoring configuration or IAM credentials over moving files.
4. If objects were incorrectly written, quarantine the bad prefix and preserve evidence.
5. If objects were incorrectly deleted, restore through provider versioning/backup if enabled.
6. Re-run upload, protected read, unauthorized read denial, and document durability checks.

Do not roll back database references without confirming object availability.

## Payments, TMS, And Notifications

During rollback:

- Pause or disable risky release paths if feature flags/kill switches are available.
- Avoid replaying payment webhooks blindly.
- Avoid duplicate Stripe transfers.
- Avoid duplicate TMS publishes.
- Confirm email/SMS notification state before retrying customer-facing sends.

If financial or integration side effects occurred, prefer forward repair with audit notes over pretending rollback erased external effects.

## Staging Rollback Drill

Before `ENT-0105` can be complete, run this in staging:

1. Deploy a known-good backend revision.
2. Deploy a harmless test revision.
3. Confirm `/health/live`, `/health/ready`, and smoke tests.
4. Roll back to the known-good revision.
5. Confirm `/health/live`, `/health/ready`, and smoke tests again.
6. Record revision IDs, operator, timestamp, and result in `docs/ENTERPRISE_WORK_BOARD.md`.

## Release Record Requirements

Every release must record:

- Backend revision/image.
- Frontend revision/image.
- Runtime secret version or source file checksum.
- Migration command result.
- Database snapshot identifier.
- Object-storage bucket and prefix.
- Smoke-test result.
- Rollback owner.
- Customer communication owner.

## Task Mapping

- `ENT-0105` owns this rollback runbook and staging rollback drill.
- `ENT-0103` owns migration command and failed-migration procedure.
- `ENT-0106` owns feature flags and kill switches.
- `ENT-1506` owns backup/restore/RPO/RTO.
