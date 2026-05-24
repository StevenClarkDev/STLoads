# Enterprise Organization And Tenant Model

This document closes `ENT-0204` and defines the tenant foundation that `ENT-0205` will enforce across route-level access checks.

## Data Model

The Rust port now has a first-class organization boundary:

- `organizations`
- `organization_roles`
- `organization_memberships`

The default migration creates:

- organization id `1`
- slug `stloads-default`
- name `STLoads Default Organization`
- account type `platform_default`

This default organization is the migration bridge for all legacy data that previously belonged only to users.

## Organization-Scoped Tables

The following tables now carry `organization_id` with a foreign key and supporting index:

- `users`
- `loads`
- `load_documents`
- `kyc_documents`
- `conversations`
- `offers`
- `escrows`
- `stloads_handoffs`
- `stloads_sync_errors`
- `stloads_reconciliation_log`

Backfill rules:

- existing users are assigned to the default organization
- loads inherit the owner's organization when available
- load documents inherit the load organization
- KYC documents inherit the user's organization
- conversations inherit the shipper's organization
- offers and escrows inherit the related load organization
- TMS handoffs inherit the related load organization or default organization
- TMS sync errors and reconciliation rows inherit the handoff organization

## Memberships

Each user receives an active `organization_memberships` row.

Default mapping:

- legacy admin role id `1` maps to organization role `admin`
- all other legacy roles map to organization role `member`

The migration includes a database trigger so new user inserts automatically create or refresh the default membership. The Rust registration and admin-create user flows also explicitly sync default membership to make the application intent clear.

## Current Scope

`ENT-0204` establishes the tenant boundary and migration path. It does not yet enforce tenant isolation in every query. That is intentionally reserved for `ENT-0205`, where the route guards and repository filters will be tightened with cross-tenant tests.

## Verification

- `cargo test -p db organization_foundation_assigns_default_membership`
- `cargo test -p backend routes::auth::tests::registration_and_password_reset_routes_work_end_to_end`

## Next Required Step

`ENT-0205` must apply organization filters and cross-tenant denial tests across load, document, chat, payment, TMS, and admin workflows.
