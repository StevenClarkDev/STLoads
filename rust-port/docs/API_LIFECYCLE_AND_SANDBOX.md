# STLoads API Lifecycle And Sandbox Governance

## Current Version

- Current version: `2026-05-26`
- Status: `active`
- Minimum customer notice before standard sunset: `180 days`
- Version header: `stloads-api-version`
- Request tracing header: `x-request-id`
- External write safety: `idempotency-key`

## Supported Artifacts

- OpenAPI: `/openapi.json`
- Postman collection: `docs/STLOADS_POSTMAN_COLLECTION.json`
- Integration portal: `/admin/integrations`
- Sandbox base URL: `https://sandbox-api.stloads.com`

## SDK Strategy

STLoads ships OpenAPI-first generated clients, official Postman examples, and runnable sample payloads before hand-written SDK packages. Typed SDK packages should be added after pilot integrations stabilize the contract and reveal the real language/runtime demand.

## Change Policy

- Additive changes can ship in the active version when fields are optional and examples remain compatible.
- Breaking changes require a new API version unless there is an active security or data-integrity incident.
- Standard sunset requires at least 180 days of customer notice.
- Emergency breaking changes require incident approval, documented customer impact, mitigation guidance, and post-incident follow-up.

## Sandbox Governance

Sandbox tenants must use synthetic or safely masked data only. Real PII, payment credentials, freight documents, and customer freight are not allowed.

Sandbox safety controls are enforced as explicit policy:

- Production payments are blocked.
- Production TMS pushes are blocked.
- Live notifications are blocked.
- Sandbox reset jobs record safety evidence before they are queued.

## Upgrade Paths

- REST API customers pin `stloads-api-version`, run compatibility examples in sandbox, then rotate production keys.
- Webhook customers add handlers for new fields in sandbox before production rollout.
- EDI partners validate mappings and 997 acknowledgements in sandbox before production enablement.
- TMS partners validate lifecycle status payloads and reconciliation behavior in sandbox before live sync.
