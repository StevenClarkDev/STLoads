# Enterprise Key Management And PCI Scope

Last updated: 2026-05-27

This document defines the Phase 17 key-management and payment-data boundary for `ENT-1706A`.

## Key Ownership

| Secret / Key | Owner | Storage | Rotation |
|---|---|---|---|
| Database credentials | DevOps/Security | IBM/runtime secret store | At least quarterly and after staff/vendor change or incident |
| Object storage credentials | DevOps/Security | IBM/runtime secret store | At least quarterly and after object-storage access changes |
| SMTP credentials | DevOps/Security | IBM/runtime secret store | At least quarterly or provider incident |
| Stripe secret key | Finance/Security | IBM/runtime secret store | Per Stripe key rotation policy and after suspected exposure |
| Stripe webhook secrets | Finance/Security | IBM/runtime secret store | Rotate on endpoint changes, incident, or at least annually |
| TMS/API shared secrets | Integrations/Security | IBM/runtime secret store | Per partner contract, incident, or at least annually |
| TLS/private keys | DevOps/Security | Certificate manager or runtime secret store | Before expiry and after suspected exposure |

## Rotation Rules

- Emergency rotation starts immediately after suspected exposure in logs, git, screenshots, chat, tickets, or vendor incident notices.
- Rotation must include deploy verification, old-secret revocation, and evidence capture.
- Dual-secret windows are allowed only when a provider requires them and must have an expiration date.

## PCI Scope Boundary

STLoads must not collect, process, transmit, store, log, or display raw card numbers, CVV, magnetic stripe data, or full payment credentials.

Allowed payment data:

- Stripe payment intent IDs.
- Stripe charge/transfer/account IDs.
- Webhook event IDs and dedupe keys.
- Amounts, currency, transfer groups, ledger entries, approval records, and settlement status.
- Stripe-hosted onboarding links and tokenized references.

Disallowed payment data:

- Raw PAN/card number.
- CVV/CVC.
- Full bank account credentials.
- Stripe secret keys or webhook secrets in logs, support screens, analytics, or audit exports.

## Required Checks

- CI secret scanning must run before merge.
- Payment logs and support screens must avoid provider secrets and client secrets.
- Hosted Stripe verification scripts must read credentials from environment variables only.
- Finance release workflows must retain two-person approval and audit evidence.
