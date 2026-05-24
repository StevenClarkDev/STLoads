# Enterprise SLA And Support Tiers

Last updated: 2026-05-24

This document supports `ENT-0007` in `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`. It defines the first enterprise-ready support tier model, uptime targets, response targets, maintenance-window expectations, and escalation commitments.

## Support Principles

- Do not sell a support commitment that operations cannot staff.
- SLA targets must map to monitoring, incident response, on-call ownership, backup/restore, and customer communication.
- Severity is based on customer impact, not internal team preference.
- SLA credits, if offered, require legal and finance approval.

## Proposed Support Tiers

| Tier | Target customer | Coverage | Response target | Notes |
| --- | --- | --- | --- | --- |
| Standard | Small shippers, carriers, brokers | Business hours | Next business day | Good for non-critical usage and pilots |
| Business | Operational customers | Business hours plus urgent incident coverage | P1 within 4 business hours | Recommended default paid enterprise tier |
| Enterprise | High-volume logistics operations | 24/7 P0 incident intake plus named escalation | P0 within 1 hour, P1 within 4 hours | Requires on-call and incident process before sale |
| Premium Enterprise | Strategic accounts | 24/7 P0/P1 intake, named success owner, scheduled reviews | Contract-specific | Requires staffing and legal approval |

## Severity Levels

| Severity | Definition | Example | Target response |
| --- | --- | --- | --- |
| P0 | Critical production outage or money/security/data-loss risk | Login unavailable for all users, duplicate payout risk, cross-tenant data exposure | 1 hour for Enterprise tier |
| P1 | Major customer workflow blocked with no practical workaround | Booking, tracking, document upload, payment release, or TMS sync blocked for active freight | 4 hours for Business/Enterprise tier |
| P2 | Important workflow degraded with workaround | Report export broken, one integration delayed, partial notification issue | 1 business day |
| P3 | Question, training, cosmetic issue, or low-risk enhancement | Help-center question, minor UI polish, feature request | 2 to 5 business days |

## Uptime Target

Initial target:

- Enterprise production availability target: 99.9% monthly once production observability and on-call are active.
- Exclusions: scheduled maintenance, customer network issues, customer integration outages, third-party carrier/shipper systems, force majeure, and approved beta/pilot limitations.

Dependencies:

- `ENT-1501` structured logs and tracing
- `ENT-1502` metrics and alerts
- `ENT-1502A` on-call, escalation, and security log export
- `ENT-1506` backup/restore/RPO/RTO
- `ENT-1508` incident response/status page/runbooks
- `ENT-1509` business continuity/tabletop exercises

## Maintenance Windows

- Standard maintenance window should be defined before production enterprise launch.
- Customer-impacting maintenance requires advance notice unless emergency security or data-protection action is needed.
- Maintenance notices must include expected impact, affected systems, start/end window, rollback condition, and support contact.

## Escalation Paths

| Escalation area | Owner | Backup |
| --- | --- | --- |
| Customer communication | Support/Ops | Customer Success |
| Freight workflow incident | Ops | Product |
| Payment or finance incident | Finance | Engineering |
| Security or privacy incident | Security/Engineering | Legal |
| Integration incident | Engineering/Ops | Product |
| Infrastructure incident | DevOps/Engineering | Support |

## SLA Credits

Decision:

- Do not offer automatic SLA credits until production telemetry, uptime calculation, incident classification, and legal terms are finalized.
- If credits are offered later, finance/legal must define credit calculation, exclusions, claim window, maximum liability, and evidence requirements.

## Customer Commitments Checklist

Before a tier is sold or activated:

- Support coverage hours are staffed.
- P0/P1 alert routing exists.
- Customer escalation contacts are recorded.
- Status-page or customer-communication process exists.
- Incident severity definitions are published internally.
- Support cases can track severity, SLA clock, status, owner, and resolution.
- Backup/restore and business continuity commitments match the sold tier.
- Legal terms match the actual operational commitment.

## Evidence To Store

- Customer support tier.
- SLA terms and exclusions.
- Named escalation contacts.
- Support case reports.
- Incident reports and post-incident reviews.
- Uptime reports once telemetry exists.
- Maintenance-window notices.
- SLA credit approval or deferral decision.
