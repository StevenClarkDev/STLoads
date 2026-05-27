# Cross-Border Tax, FX, Duties, And Incoterms Rules

## First-Release Decision

STLoads first enterprise release supports US domestic freight workflows using USD for rating, escrow funding, release, reporting, and operational summaries.

Cross-border financial obligations are explicitly deferred until Legal and Finance approve a broader operating model. Deferred items include non-USD rating, non-USD invoicing, non-USD settlement, VAT/GST, sales tax, withholding, duties, customs fees, broker fees, Incoterms, customs responsibility, and currency-risk ownership.

## Executable Controls

- Cross-border, freight-forwarding, and mixed-mode load workflows remain blocked by freight-mode validation until the later legal, tax, FX, customs, and localization controls are approved.
- Escrow funding accepts only USD. Non-USD funding attempts return an explicit unsupported-state message before Stripe or local escrow persistence.
- Organization-level `cross_border_finance_policies` records document supported currencies, FX support, tax policy status, duty policy status, customs-fee policy status, Incoterms support, approval owner, and approval timestamp.
- `load_cross_border_finance_checks` records future per-load policy decisions and blocked attempts when the workflow is expanded.

## Future Approval Checklist

- Define supported operating countries, tax registrations, and tax calculation owner.
- Choose FX provider, rate-lock timing, rounding, revaluation, failed-rate fallback, audit retention, and reconciliation cadence.
- Decide whether rating, invoice, settlement, and reports can each use non-USD currencies.
- Define Incoterms support, customs broker responsibility, importer/exporter of record responsibility, duties/taxes payer, and customs fee billing owner.
- Add finance/legal approval evidence before enabling non-USD or cross-border money movement.
