# Enterprise WAF, DDoS, And Bot Protection

This document partially addresses `ENT-1708`. The policy and operational workflow are defined, but the task cannot be honestly marked complete until the selected production edge provider is configured and tested.

## Required Public Surfaces

Protect these public surfaces at the edge:

- Frontend application routes.
- Backend API routes.
- Authentication routes, login, MFA, OTP, password reset, registration, and invitation flows.
- Public quote, tracking, document-download, webhook, partner API, EDI/API integration, and sandbox routes.
- Static assets and runtime configuration files.

## Required Edge Controls

Before enterprise launch, the selected provider must enable:

- Managed DDoS protection for all public hostnames.
- WAF managed rules for common OWASP classes: injection, XSS, path traversal, protocol abuse, malicious user agents, and request smuggling patterns.
- Bot protection or challenge rules for login, registration, OTP, password reset, public quote, customer tracking, and high-volume API routes.
- Per-route rate limits that complement the application-level rate limiters.
- IP and ASN allow/block lists that can be changed without code deployment.
- Geo or region rules only when approved by Legal/Product for customer commitments.
- WAF event logs exported to the security log-drain/SIEM path when that lane is complete.

## Baseline Rule Set

| Surface | Edge control | Notes |
| --- | --- | --- |
| `/api/auth/*` | Managed WAF, bot challenge, strict rate limit | Protect login, OTP, MFA, and password reset. |
| `/api/partner/*` | Managed WAF, rate limit, optional allowlist by customer | Do not block valid TMS/EDI partners without escalation. |
| `/api/webhooks/*` | Managed WAF with signature-aware allow rules | Stripe/TMS/webhook signature checks remain application controls. |
| `/api/loads*` and search | Managed WAF, adaptive rate limit | Prevent scraping and abusive search traffic. |
| Customer tracking links | Bot protection and burst limits | Avoid breaking customer-visible tracking pages. |
| Document downloads | WAF, rate limit, hotlink protection where supported | Application auth remains authoritative. |

## Abuse Response Workflow

1. Detect abuse through edge alerts, backend rate-limit metrics, auth failures, support reports, or infrastructure traffic anomalies.
2. Classify severity:
   - Critical: service-impacting DDoS, credential stuffing at scale, active exploit attempts, or data-access attempts.
   - High: targeted tenant abuse, high-volume scraping, or repeated partner/API attacks.
   - Medium: suspicious bot bursts, unusual route pressure, or failed auth spikes.
   - Low: nuisance traffic with no user impact.
3. Apply edge mitigation first when it can be done safely: block IP/ASN, challenge route, tighten rate limit, enable managed attack mode, or create a temporary allowlist.
4. Open an incident/security ticket and capture evidence: timestamps, hostnames, source IPs/ASNs, routes, WAF rule IDs, request IDs, tenant IDs if known, and mitigation actions.
5. Review after mitigation to decide whether application rules, customer communication, credential resets, partner allowlists, or legal escalation are needed.

## Change Control

WAF changes that can block customer traffic require:

- Owner, reason, affected routes, duration, rollback plan, and approval.
- Short emergency approval path for active attacks.
- Post-change evidence and customer impact review.

## Completion Criteria For `ENT-1708`

`ENT-1708` can move from partial to complete only when:

- The production edge provider is selected and configured for all public hostnames.
- Managed WAF, DDoS protection, bot protection, and route-specific rate limits are enabled.
- Security can add allow/block/challenge rules without code deployment.
- WAF events are monitored and alerting is tested.
- A tabletop or controlled test proves a block/challenge/rollback workflow works.
