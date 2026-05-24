# Enterprise Tenant Isolation

This document records the first production enforcement slice for `ENT-0205`.

## Current Boundary

Authenticated Rust sessions now carry organization context from `organization_memberships`:

- `organization_id`
- `organization_role_key`

The backend treats missing organization context as unsafe for tenant-scoped operations.

## Enforced Surfaces

The following Rust paths now apply organization checks before returning or mutating scoped data:

- load profile access
- load document create, update, and blockchain verification
- escrow fund, hold, and release
- marketplace offer review
- marketplace chat send and read receipt updates
- admin chat workspace results for organization-scoped admin sessions
- user-authenticated TMS requeue, withdraw, and close operations
- admin user directory and onboarding review lists
- admin user profile, profile update, account update, and delete workflows

System-to-system TMS and Stripe webhook flows remain credential-scoped by shared secrets or Stripe signatures. Those flows are not treated as user break-glass access.

## Repository Guards

The database layer now exposes scoped helpers for tenant-sensitive lookups:

- `find_load_leg_scope`
- `find_load_document_scope`
- `find_escrow_for_leg_in_organization`
- `handoff_belongs_to_organization`
- organization-filtered chat workspace lookups for admin sessions

These helpers make cross-tenant checks explicit and testable.

## Verification

The integration test `tenant_scoped_queries_reject_cross_organization_records` creates two organizations with separate users, loads, documents, conversations, escrows, and TMS handoffs. It verifies that tenant A cannot see tenant B records through the scoped DB helpers.

Verified commands:

- `cargo test -p db tenant_scoped_queries_reject_cross_organization_records`
- `cargo test -p db`
- `cargo test -p backend`
- `cargo check -p frontend-leptos`

## Break-Glass Rule

Cross-tenant user access is default-deny unless an MFA-verified admin starts a time-boxed break-glass session.

The current break-glass workflow requires:

- explicit justification capture
- ticket or incident reference
- MFA-confirmed session
- target organization scope
- 5 to 60 minute expiry
- immutable audit events when the session starts and when cross-tenant admin access is used

Future support tooling can add approval queues and review dashboards on top of the same audit ledger, but cross-tenant access is no longer implicit.
