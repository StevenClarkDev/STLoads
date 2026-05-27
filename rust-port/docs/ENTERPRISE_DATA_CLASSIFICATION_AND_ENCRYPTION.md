# Enterprise Data Classification And Encryption

Last updated: 2026-05-27

This document defines the Phase 17 data handling baseline for `ENT-1706`.

## Classification

| Class | Examples | Handling Requirement |
|---|---|---|
| Public | Marketing copy, public role counts, public health status | No secrets or tenant data. Cacheable only when explicitly safe. |
| Internal | Operational runbooks, non-sensitive configuration, feature flags | Available to STLoads staff with business need. |
| Confidential | Loads, rates, offers, customer configs, lane guides, invoices, settlements | Tenant-scoped access, audit on sensitive exports, encrypted storage. |
| Regulated/Sensitive | Identity, KYC/KYB, authority, insurance, W-9/tax, legal agreements, support notes | Least privilege, masking in support surfaces, audit on access/export. |
| Location | Driver tracking, geofence events, route pings, tracking consent records | Consent-required, retention-limited, tenant/load scoped. |
| Document | POD, BOL, rate confirmation, carrier packet, certificates, uploaded files | Access-controlled, versioned, validated, scanner-ready, encrypted object storage. |
| Payment-related | Payment intents, transfers, payout references, webhook event IDs, ledger rows | Tokenized provider IDs only; no raw card data. Finance access controls and audit required. |
| Secret | API keys, webhook secrets, database URLs, SMTP credentials, TLS/private keys | Never stored in git, logs, support screens, or analytics. Provider secret manager or environment only. |

## Encryption Baseline

- TLS is required for public frontend, backend API, partner APIs, webhooks, SMTP, Stripe, object storage, and database connections.
- Database, object storage, backups, and managed logs must use provider-managed encryption at rest.
- Local development files are not a production secret store.
- Application-level encryption is reserved for fields where provider encryption and access control are not enough, such as future bank-account metadata or customer-specific regulated identifiers.

## Redaction Rules

Never log or expose:

- Authorization headers, bearer tokens, session IDs, OTPs, MFA recovery codes, password reset tokens.
- Stripe secret keys, webhook secrets, client secrets, raw webhook signatures, or payment method secrets.
- Database URLs, SMTP passwords, object-storage credentials, IBM API keys, TLS/private keys.
- Full document contents in logs, traces, analytics, or support search.

Support/admin screens should mask:

- Email where not needed for the workflow.
- Phone numbers, tax IDs, policy numbers, document hashes, and payout metadata.
- Location history outside the active load/permission context.

## Enforcement Evidence

- CI secret scan blocks likely committed credentials.
- `scripts/run_sensitive_output_scan.ps1` blocks obvious password, token, secret, authorization, client-secret, and webhook-secret output in Rust logs and frontend console statements.
- Hosted smoke scripts require env-provided passwords instead of committed defaults.
- Payment boundaries are documented in `docs/ENTERPRISE_KEY_MANAGEMENT_AND_PCI_SCOPE.md`.
- Browser security headers are documented in `docs/ENTERPRISE_SECURITY_HEADERS_AND_CSP.md`.
