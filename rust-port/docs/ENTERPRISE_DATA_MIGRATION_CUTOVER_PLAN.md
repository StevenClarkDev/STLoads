# Enterprise Data Migration And Cutover Plan

This document defines the production data migration and cutover plan for `ENT-0107`.

## Goal

Move STLoads from the legacy production source of truth to the Rust/PostgreSQL platform without data loss, duplicated financial actions, broken freight workflows, or unclear ownership.

## Source Of Truth

| Phase | Source Of Truth | Notes |
| --- | --- | --- |
| Pre-cutover | Legacy production application/database | Rust may read mirrored/staged data only. |
| Rehearsal | Staging PostgreSQL with production-like copied/sanitized data | Business users validate migrated data before authority changes. |
| Freeze window | Legacy remains authoritative until final approval | Writes may be paused or limited by workflow. |
| Cutover | Rust/PostgreSQL becomes authoritative after signoff | Legacy becomes read-only/archive unless rollback is triggered. |
| Post-cutover | Rust/PostgreSQL | Reconciliation remains active until signoff window closes. |

## Data Domains

Migration planning must cover:

- Users, roles, permissions, sessions, lifecycle states, OTP/password reset state where allowed.
- Organizations, tenants, offices, contacts, carrier/broker/shipper profiles.
- Loads, load legs, locations, commodities, equipment, pricing, status histories.
- Offers, bids, tenders, booking state, carrier assignment, conversations, messages.
- Tracking sessions, GPS pings, execution notes, milestones, pickup/delivery events.
- Documents, document metadata, protected file paths, object-storage providers, document hashes.
- Payments, escrow records, payment intents, transfers, holds, releases, refunds, disputes.
- TMS handoffs, retry queue, reconciliation log, webhook events, external IDs.
- Master data: countries, states, cities, ports, commodities, equipment, units, business rules.
- Audit/history records required for support, compliance, legal, and finance.

## Reconciliation Reports

Before production authority changes, create reports comparing legacy source data to Rust/PostgreSQL target data:

- User count by role, status, organization, and tenant.
- Load count by status, owner, equipment, lane, and date.
- Leg count by status, booked carrier, price, and execution stage.
- Document count by owner, type, storage provider, and protected-read availability.
- Payment/escrow count and total by status, currency, payer, payee, and load leg.
- TMS handoff count by status, tenant, external TMS load ID, retry state, and reconciliation outcome.
- Conversation/message counts by load and participant.
- Master-data count by table and active/archive state.

Each report must show:

- Legacy count/value.
- Rust target count/value.
- Difference.
- Sample mismatches.
- Owner.
- Resolution status.

The Rust target report is generated with:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\run_cutover_reconciliation.ps1" -RustDatabaseUrl "postgres://USER:PASSWORD@HOST:PORT/DB?sslmode=require" -OutputPath "rust-port\runtime\cutover-reconciliation.json"
```

After a legacy/source summary exists in the same JSON shape, compare it with:

```powershell
powershell -ExecutionPolicy Bypass -File "rust-port\scripts\run_cutover_reconciliation.ps1" -RustDatabaseUrl "postgres://USER:PASSWORD@HOST:PORT/DB?sslmode=require" -ExpectedJsonPath "rust-port\runtime\legacy-cutover-summary.json" -OutputPath "rust-port\runtime\cutover-reconciliation.json"
```

The script exits non-zero when the expected and actual summaries mismatch.

## Freeze Window

Before cutover:

1. Announce maintenance/freeze window to affected internal teams and customers.
2. Pause or restrict high-risk writes: booking, payment release, TMS pushes, document uploads, and lifecycle changes.
3. Record current legacy backup/snapshot and Rust PostgreSQL snapshot.
4. Run final migration command/process.
5. Run reconciliation reports.
6. Run smoke checks.
7. Get business signoff from operations, finance, support, and product.

## Rollback Point

Rollback is possible only before Rust becomes the long-running source of truth for new writes.

Rollback trigger examples:

- Reconciliation mismatch in critical data.
- Login/role/access failure for core users.
- Missing active loads or active legs.
- Missing protected documents.
- Payment or escrow mismatch.
- TMS handoff mismatch for active freight.
- Business owner rejects validation.

After Rust has accepted live writes, prefer controlled forward repair unless data integrity requires restoring a snapshot.

## Document/Object Migration

If legacy files exist outside IBM Cloud Object Storage:

- Inventory source storage locations and file counts.
- Copy files to the target bucket/prefix.
- Preserve document IDs, filenames, MIME types, hashes where available, uploader, timestamps, and access scope.
- Verify protected read for admin, owner/uploader, and unauthorized denial.
- Keep source files immutable until post-cutover retention approval.

## Business Validation

Business users must validate:

- Users can log in with expected roles.
- Admin queues show expected pending/approved/rejected/revision states.
- Active loads and legs match operational expectations.
- Carrier booking state is correct.
- Execution/tracking state is correct.
- Payments/escrow state is correct.
- TMS handoffs and drift queues are correct.
- Documents open only for authorized users.

## Required Evidence Before Completion

`ENT-0107` is not complete until:

- Staging rehearsal uses production-like migrated data.
- Reconciliation reports exist and are reviewed.
- Business users validate migrated data.
- Freeze window and rollback point are approved.
- Document/object migration is verified if legacy files are outside IBM COS.
- Cutover result is recorded in the enterprise work board.

## Task Mapping

- `ENT-0107` owns this plan and the actual cutover rehearsal evidence.
- `ENT-0103` owns migration command mechanics.
- `ENT-0105` owns rollback procedure.
- `ENT-0106` owns kill switches during freeze/cutover.
- `ENT-1506` owns backup/restore/RPO/RTO.
