# Enterprise Reporting And Metrics

Phase 14 establishes the first production reporting contract for STLoads. Operational screens should keep using workflow tables and APIs; analytics should use read models, snapshots, or warehouse exports so reports do not slow freight execution.

## Business Metrics

The accepted first-release metrics are seeded in `business_metric_definitions`:

- `posted_loads`: count of loads posted during the reporting period.
- `booked_loads`: count of load legs with a booked carrier.
- `acceptance_rate`: accepted offers or tenders divided by eligible offers or tenders.
- `quote_to_book_time`: minutes from quote/load posting to booking.
- `tracking_compliance`: active execution legs with consented and timely tracking.
- `on_time_pickup`: pickup appointments completed inside the accepted on-time window.
- `on_time_delivery`: delivery appointments completed inside the accepted on-time window.
- `document_cycle_time`: minutes from delivery completion to required document approval.
- `margin`: customer revenue minus carrier cost and approved accessorials.
- `payout_time`: hours from financial release eligibility to carrier payout completion.
- `dispute_rate`: completed loads with claims, detention, accessorial, or settlement disputes divided by completed loads.

## Reporting Read Models

The first read models are registered in `reporting_read_models`:

- `load_operational_metrics_daily`
- `finance_metrics_daily`
- `customer_scorecards_monthly`
- `carrier_scorecards_monthly`

Each model records source tables, target table, refresh strategy, refresh cadence, owning team, refresh status, and whether warehouse export is enabled.

## Scorecards

Customer scorecards track volume, booking conversion, quote-to-book time, on-time service, document cycle time, gross margin, dispute rate, score, and tone.

Carrier scorecards track offered/accepted loads, acceptance rate, tracking compliance, on-time pickup, on-time delivery, claims rate, document quality, payout cycle, score, and tone.

## Operating Rule

Phase 14 reporting is allowed to be eventually consistent. Read models should refresh on hourly or daily cadence depending on customer impact. Operational pages must not depend on expensive aggregate queries over hot workflow tables.

## Lane Pricing Intelligence

Lane pricing history is stored in `lane_pricing_history` with organization, lane key, origin, destination, equipment type, mode, observed rate, source type, pickup date, margin, and delivery outcome.

Current pricing recommendations live in `lane_pricing_recommendations` with recommended, low, and high rates, confidence score, sample size, anomaly status, and recommendation reason. Rate anomalies are deliberately visible before recommendations are used in customer-facing pricing.

## Global Search

The global search index lives in `global_search_documents`. Each result is organization-scoped and may require a permission key such as `manage_loads` or `manage_payments`. Search documents cover loads, users, organizations, documents, invoices, payments, conversations, TMS handoffs, and support cases.

## Data Quality

Data quality governance is stored in:

- `data_quality_rules`: rule catalog, severity, owner, cadence, alert threshold, and repair playbook.
- `data_quality_runs`: execution evidence for scheduled or manual checks.
- `data_quality_findings`: owner-routed findings with severity, entity, status, repair action, and audit linkage.

The first rule catalog covers orphan records, invalid states, duplicate external references, missing documents, stale TMS handoffs, unmatched payments, tenant ownership drift, lane-rate anomalies, carrier score changes, suspicious tracking, unusual document replacement, and sudden volume changes.
