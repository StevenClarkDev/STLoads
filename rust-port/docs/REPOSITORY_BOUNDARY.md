# STLoads And ATMP Repository Boundary

## Purpose

This document prevents STLoads and ATMP-OS from drifting into one mixed codebase.

STLoads and ATMP-OS work hand in hand, but they must remain separate products with separate repositories, commits, deployments, and ownership boundaries.

## STLoads Repository

Use the STLoads repository for:

- STLoads backend routes
- STLoads domain models
- STLoads database migrations
- STLoads frontend/Leptos screens
- STLoads middleware
- STLoads auth/RBAC implementation
- STLoads marketplace workflows
- STLoads carrier eligibility
- STLoads offers, tenders, and booking locks
- STLoads documents, payments, tracking, and reconciliation
- STLoads IBM frontend/backend deployment files
- STLoads production-readiness documentation

Local path:

```text
C:\New folder\STLoads-api-review
```

Remote:

```text
https://github.com/StevenClarkDev/STLoads.git
```

## ATMP Repository

Use the ATMP repository only for:

- ATMP Dispatch system-of-record behavior
- ATMP Dispatch STLoads push/requeue/queue routes
- ATMP-side outbound handoff adapter
- ATMP-side STLoads launcher or redirect
- ATMP-side contract tests that prove Dispatch emits the agreed STLoads payload
- ATMP-side documentation that belongs to Dispatch

Local path:

```text
C:\New folder\atmp-os
```

Remote:

```text
https://github.com/sabertech-development/atmp-os.git
```

## Split Commit Rule

If a task requires both products:

1. Commit STLoads files in the STLoads repo.
2. Commit ATMP files in the ATMP repo.
3. Reference the matching commit in each commit message or release note.
4. Do not commit STLoads files into ATMP.
5. Do not commit ATMP files into STLoads.

## Deployment Rule

STLoads deploys independently from ATMP.

ATMP should only need the STLoads public frontend URL, backend API URL, and integration credentials.

STLoads should only need the ATMP outbound callback URL, integration credentials, and contract version.

## Source-Of-Truth Rule

ATMP-OS is the TMS and operational source of truth.

STLoads is the marketplace/load-board layer:

- board exposure
- carrier visibility
- search
- offers
- tenders
- booking
- execution visibility
- documents
- payment board workflows
- reconciliation events back to ATMP

STLoads must not mutate canonical shipment state silently. Any marketplace outcome that matters to the shipment lifecycle must emit a contract-compliant event back to ATMP.
