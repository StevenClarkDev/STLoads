# STLoads API Contract V1

Version: `2026-05-26`

The Rust backend now exposes the enterprise API contract at:

- `GET /openapi.json`

This OpenAPI 3.1 contract covers the first enterprise integration surface for:

- Auth, SSO, SCIM, legal agreement, profile, and carrier onboarding APIs.
- Load board search, load creation, bulk import, API posting, booking, lifecycle, carrier match, and freight document APIs.
- Marketplace offers, counteroffers, tenders, and chat workspace APIs.
- Execution tracking, route plans, telematics, consented location pings, POD, closeout, and customer tracking APIs.
- Payments, escrow, Stripe Connect, Stripe webhooks, accounting export, platform billing, credit, and payout controls.
- STLoads/TMS handoff, retry, requeue, withdraw, close, reconciliation, and inbound webhook APIs.
- Webhook compatibility routes under `/api/stloads`.

## Contract Rules

- All responses follow the existing STLoads `ApiResponse` envelope unless an endpoint explicitly documents a protocol-specific acknowledgement.
- All endpoints accept `x-request-id`; the backend generates one when omitted.
- All external writes must support `idempotency-key`; enforcement is completed in `ENT-1103`.
- Partner APIs use `x-stloads-api-key`; issuing, rotating, signing, and rate limiting these credentials is completed in `ENT-1102`.
- Webhook routes use provider signatures or STLoads webhook signatures depending on direction and provider.
- Partners may send `stloads-api-version` to pin behavior to a supported contract version.

## Lifecycle Policy

- Current version: `2026-05-26`.
- Compatibility window: at least 365 days for a published enterprise version.
- Sunset notice window: at least 180 days for non-emergency removals.
- Breaking changes require a new API version unless a security incident requires emergency action.
- SDKs, Postman collections, runnable partner examples, and formal deprecation notices are handled in `ENT-1109`.
