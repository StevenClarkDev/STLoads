# Governed Master Data And Configuration Admin

## Governance Scope

Enterprise master data is split into simple writable catalogs and governed configuration. Simple catalogs can be edited by users with `manage_master_data`. Governed configuration is visible in the admin catalog and must retain effective dates, approval status, and rollback evidence before it can safely drive billing, compliance, dispatch, or customer-specific workflows.

## First Governed Catalogs

- Service levels: standard, expedited, guaranteed, and team.
- Rejection reasons: capacity unavailable, compliance block, rate rejected, and customer cancelled.
- Exception reasons: late pickup, late delivery, missing POD, payment hold, and TMS drift.

## Change Ledger

`governed_configuration_changes` records the configuration area, target table, target record, change type, requester, approver, approval status, rollback payload, summary, and effective dates. High-impact configuration changes should write to this ledger when write workflows are expanded beyond the initial seeded catalogs.

## Next Expansion Rules

- Add write routes only after each governed catalog has validation, permission checks, audit entries, effective-date handling, and rollback payload capture.
- Customer-specific configuration must link to organization, customer contract, lane, facility, carrier group, or billing rule before becoming active.
- Deprecated or inactive values must remain readable for historical loads and reports, while new load creation should prefer active/effective values.
