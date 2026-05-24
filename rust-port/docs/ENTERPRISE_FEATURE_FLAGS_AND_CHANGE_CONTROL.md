# Enterprise Feature Flags, Kill Switches, And Change Control

This document defines the first enterprise feature-flag and emergency kill-switch model for `ENT-0106`.

## Goals

- Risky changes can be deployed dark and enabled deliberately.
- Critical workflows can be paused during an incident without redeploying code.
- Production changes to money, booking, auth, tenant isolation, integrations, and migrations have explicit approval.

## Runtime Kill Switches

The backend supports these environment-driven kill switches:

| Env var | Default | Effect |
| --- | --- | --- |
| `KILL_SWITCH_PAYMENTS` | `false` | Blocks escrow release from the Rust payments route. |
| `KILL_SWITCH_BOOKING` | `false` | Blocks carrier self-booking from the Rust load board route. |
| `KILL_SWITCH_TMS_PUSHES` | `false` | Blocks new TMS handoff pushes into STLoads. |
| `KILL_SWITCH_NOTIFICATIONS` | `false` | Skips outbound email notification delivery/enqueue. |
| `KILL_SWITCH_DOCUMENT_UPLOADS` | `false` | Blocks load and execution document upload routes. |

Kill switches are emergency controls. They are not a substitute for fixing the underlying incident.

## Ownership

| Environment | Who can enable | Required approval |
| --- | --- | --- |
| Local | Any developer | None |
| CI | CI owner | Engineering lead if persistent |
| Staging | Engineering lead or release owner | Release owner |
| Pilot | Release owner | Product, support, and engineering |
| Production | Incident commander or release owner | Engineering lead plus business owner for affected workflow |

Payment-related switches require finance awareness. TMS-related switches require operations awareness. Customer-facing notification switches require support/customer-success awareness.

## Change Approval Checklist

Before production changes in these areas, record approval evidence:

- Migrations: migration owner, database snapshot, staging result, rollback/forward-fix path.
- Payments: finance owner, Stripe mode, duplicate payout risk review, webhook replay plan.
- Auth/session/RBAC: security owner, tenant-isolation review, account-lockout rollback path.
- Tenant isolation: data-access review, audit impact, support visibility.
- Integrations/API/TMS: operations owner, retry/replay policy, customer communication if applicable.
- Documents/object storage: object-storage owner, protected-read validation, backup/versioning status.

## Dark Launch Strategy

- Deploy code with kill switches off by default unless an incident is active.
- For risky new workflows, add a separate feature flag before broad rollout.
- Enable first in staging.
- Enable for pilot tenants or internal users before full production.
- Watch `/health/ready`, smoke checks, support cases, and logs.
- Roll back or disable immediately if money, data integrity, tenant isolation, or critical freight workflow risk appears.

## Incident Use

When a kill switch is enabled:

1. Record who enabled it, when, why, and expected customer impact.
2. Notify support/customer success if customer workflows are affected.
3. Add a status/incident note if enterprise customers may notice impact.
4. Verify the workflow is actually paused.
5. Fix the underlying incident.
6. Re-enable only after validation and owner approval.
7. Record post-incident follow-up.

## Implemented Coverage

- `KILL_SWITCH_PAYMENTS` blocks escrow release.
- `KILL_SWITCH_BOOKING` blocks carrier self-booking.
- `KILL_SWITCH_TMS_PUSHES` blocks TMS handoff pushes.
- `KILL_SWITCH_NOTIFICATIONS` skips outbound email notification delivery/enqueue.
- `KILL_SWITCH_DOCUMENT_UPLOADS` blocks load and execution document uploads.

Future tasks should extend kill switches to every production-critical workflow as those workflows mature.
