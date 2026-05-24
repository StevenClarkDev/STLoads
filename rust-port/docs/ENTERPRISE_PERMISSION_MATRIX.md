# Enterprise Permission Matrix

This document closes `ENT-0206` by defining the Rust permission contract for platform roles and organization roles.

## Permission Keys

The Rust backend currently enforces these permission keys:

- `access_admin_portal`
- `manage_users`
- `manage_roles`
- `manage_master_data`
- `manage_loads`
- `manage_dispatch_desk`
- `manage_marketplace`
- `manage_tracking`
- `manage_payments`
- `manage_tms_operations`

Route guards must use these keys instead of hard-coded role labels whenever possible. Owner checks and participant checks still apply on top of permission checks for tenant-sensitive records.

## Platform Role Matrix

| Platform role | Permissions |
| --- | --- |
| Admin | all permission keys |
| Shipper | `manage_loads`, `manage_marketplace`, `manage_payments` |
| Carrier | `manage_marketplace`, `manage_tracking`, `manage_payments` |
| Broker | `manage_loads`, `manage_dispatch_desk`, `manage_marketplace`, `manage_payments` |
| Freight Forwarder | `manage_loads`, `manage_dispatch_desk`, `manage_marketplace`, `manage_payments`, `manage_tms_operations` |

Platform role permissions are defined in `crates/domain/src/auth.rs` and can be overridden by database-backed `role_has_permissions` records.

## Organization Role Matrix

| Organization role | Permissions |
| --- | --- |
| owner | all permission keys |
| admin | all permission keys |
| finance | `access_admin_portal`, `manage_payments` |
| operator | `manage_loads`, `manage_dispatch_desk`, `manage_marketplace`, `manage_tracking`, `manage_tms_operations` |
| support | `access_admin_portal`, `manage_users` |
| integration_admin | `access_admin_portal`, `manage_tms_operations` |
| member | no additional organization-level permissions |
| auditor | no write permissions in the current Rust slice |

Organization role permissions are stored in `organization_role_permissions` from migration `0013_organization_role_permissions.sql`.

## Session Resolution

When a session is resolved:

1. The platform role contributes permissions from `role_has_permissions` when present.
2. The user active organization membership contributes permissions from `organization_role_permissions`.
3. The final permission list is sorted and deduplicated.
4. Role or permission changes delete affected personal access tokens so the user must reconnect with a fresh session.

## Route Guard Map

| Surface | Required permission or guard |
| --- | --- |
| Admin overview | one of `access_admin_portal`, `manage_tms_operations`, `manage_payments`, `manage_master_data`, `manage_users` |
| Admin users, onboarding review, profile, break-glass | `access_admin_portal` or `manage_users`; sensitive mutations require MFA |
| Admin role permissions | `access_admin_portal` or `manage_roles`; mutations require MFA |
| Admin loads and load review | `access_admin_portal` or `manage_loads` |
| Admin STLoads operations and reconciliation | `access_admin_portal` or `manage_tms_operations` |
| Master data mutation routes | `access_admin_portal` or `manage_master_data` |
| Load builder create/update | `manage_loads` |
| Load profile documents | same-organization load owner with `manage_loads`, or same-organization admin/dispatch permission |
| Dispatch desk actions | one of `manage_dispatch_desk`, `manage_loads`, `access_admin_portal` |
| Marketplace offer review | same-organization load owner or same-organization admin |
| Marketplace chat send/read | same-organization conversation participant |
| Execution actions and documents | booked carrier, owner, `manage_tracking`, `manage_loads`, or `access_admin_portal` depending on action |
| Payments Connect and escrow lifecycle | `manage_payments`/`access_admin_portal` plus same-organization owner checks for escrow actions |
| TMS lifecycle | shared secret or one of `manage_tms_operations`, `access_admin_portal`, `manage_dispatch_desk`, `manage_loads` with tenant checks |
| TMS webhooks | shared secret or one of `manage_tms_operations`, `access_admin_portal` |
| Realtime websocket | valid bearer session token |

## Known Permission Gaps For Later Tasks

The current Rust permission keys are intentionally coarse. Later tasks should split read-only and support-only access into narrower keys before broad support tooling launches:

- `view_admin_portal`
- `view_audit_events`
- `manage_support_cases`
- `view_reports`
- `manage_billing`
- `manage_integrations`
- `impersonate_with_approval`

Until those keys exist, auditor remains read-only by policy but receives no privileged backend route permissions.
