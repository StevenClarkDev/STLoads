# Status And Reference Data Governance

This document is the operational companion to `domain::governance::STATUS_GOVERNANCE_CONTRACT`.
The code contract is the machine-readable source for owners, visibility, and required verification;
this document explains how teams use it when changing freight workflow states or reference data.

## Scope

The governed families are:

- `load_leg_status`: load posting, booking, execution, funding, closeout, and payout milestones.
- `offer_status`: carrier offer review and acceptance lifecycle.
- `escrow_status`: Stripe-backed funding, release, refund, hold, and failure states.
- `tms_handoff_status`: internal STLOADS handoff publishing and reconciliation states.
- `tms_external_status`: upstream TMS operational statuses received by webhook or reconciliation.
- `master_data_reference`: countries, cities, locations, load types, equipment, commodity types, and legacy load status lookup rows.

## Canonical Owners

| Family | Owner | Source Of Truth |
| --- | --- | --- |
| Load/leg statuses | Product Operations / Backend | `domain::dispatch::LegacyLoadLegStatusCode` and `LEGACY_LOAD_LEG_STATUS_DESCRIPTORS` |
| Offer statuses | Marketplace Product / Backend | `domain::marketplace::OfferStatus` and `OFFER_STATUS_DESCRIPTORS` |
| Escrow statuses | Finance Operations / Backend | `domain::payments::EscrowStatus` and `ESCROW_STATUS_DESCRIPTORS` |
| TMS handoff statuses | Integrations Operations / Backend | `domain::tms::HandoffStatus` and `HANDOFF_STATUS_DESCRIPTORS` |
| TMS external statuses | Integrations Product / Backend | `domain::tms::TmsStatus` and `TMS_STATUS_DESCRIPTORS` |
| Master/reference data | Data Stewardship / Product Operations | `domain::master_data::MASTER_DATA_SECTIONS` and admin master-data workflows |

## Visibility Rules

Customer-visible statuses can appear in shipper, broker, carrier, freight forwarder, load board,
execution, document, and notification copy. Internal-only statuses are limited to admin, support,
audit, reconciliation, and export surfaces unless Product explicitly approves exposure.

| Family | Customer-Visible | Internal-Only |
| --- | --- | --- |
| Load/leg statuses | New, Reviewed, Offer Ready, Booked, Pickup Started, At Pickup, In Transit, Escrow Funded, At Delivery, Delivered, Paid Out | Draft |
| Offer statuses | Pending, Accepted, Declined | None |
| Escrow statuses | unfunded, funded, released, refunded | on_hold, failed |
| TMS handoff statuses | None | queued, push_in_progress, published, push_failed, requeue_required, withdrawn, closed |
| TMS external statuses | dispatched, in_transit, at_pickup, at_delivery, delivered, cancelled | invoiced, settled |
| Master/reference data | countries, cities, locations, load_types, equipments, commodity_types | load_status_master |

## Change-Control Checklist

Every status or reference-data workflow change must include:

1. A product/backend change request naming the status family, affected customers, data owner, and migration risk.
2. A Rust domain enum or descriptor update before runtime behavior changes.
3. A database migration when persisted values, lookup rows, constraints, indexes, or backfills change.
4. API DTO, UI copy, admin/support, audit/export, and documentation updates when visibility or meaning changes.
5. Tests for allowed transitions, forbidden transitions, visibility, audit/history evidence, and historical compatibility.
6. Rollout, rollback, and customer communication notes before deployment.

## Required Verification Gates

At minimum, run the gates named in `STATUS_GOVERNANCE_FAMILIES.required_verification` for the affected
family. For broad workflow changes, also run:

- `cargo fmt --check`
- `cargo check -p backend`
- `cargo check -p frontend-leptos`
- `cargo test -p backend`
- `cargo test -p db`
- `trunk build --release`

## Non-Negotiable Rules

- Persisted status values are contract data; never rename, reuse, or delete them without a migration and compatibility plan.
- Status transitions that affect money, carrier assignment, document requirements, tracking, customer notifications, or TMS sync must write audit or history evidence.
- Internal-only statuses may be exported for compliance, but customer-facing workflow copy must use approved customer-visible language.
- New master data that affects routing, pricing, search, compliance, or documents needs an owner, import/export behavior, rollback plan, and tests before release.
