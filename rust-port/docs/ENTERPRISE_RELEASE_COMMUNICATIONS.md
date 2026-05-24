# Enterprise Release Notes, UAT, And Rollout Communication

This document defines customer-facing release communication for `ENT-0106A`.

## Goal

Enterprise customers should not be surprised by operational changes that affect freight workflows, finance, integrations, compliance, documents, or support.

## Release Communication Types

| Type | Audience | When Used | Owner |
| --- | --- | --- | --- |
| Release notes | All affected customers and internal teams | Every customer-visible release | Product |
| Customer changelog | Customers, sales, support, customer success | Ongoing record of shipped changes | Product/Ops |
| Maintenance notice | Affected customers | Planned downtime, degraded service, risky deploy window | Ops/Support |
| Known issue notice | Affected customers and support | Confirmed issue with workaround or pending fix | Support/Product |
| Breaking workflow notice | Customers with affected workflows | UI/workflow/API/integration behavior changes | Product/Ops |
| UAT plan | Pilot customers and internal operators | Before risky or major workflow release | Product/Customer Success |

## Required Release Note Fields

- Release title.
- Release date and target environment.
- Customer impact summary.
- Affected roles: shipper, broker, carrier, driver, admin, finance, support.
- Affected workflows: auth, load posting, booking, tracking, documents, payments, TMS/API, notifications, reports, admin.
- New behavior.
- Changed behavior.
- Fixed issues.
- Known issues.
- Required customer action.
- Rollback/escalation condition.
- Support contact and case instructions.

## Notice Windows

| Change Type | Minimum Notice |
| --- | --- |
| No-impact bug fix | Same day release notes |
| Minor UI copy/layout change | 2 business days when enterprise users are affected |
| Operational workflow change | 5 business days |
| Finance/payment behavior change | 10 business days plus finance contact notice |
| Integration/API/TMS behavior change | 15 business days unless emergency |
| Planned maintenance/degraded service | 5 business days where practical |
| Emergency security or data-integrity fix | As soon as safely communicable |

## UAT And Pilot Rollout

Risky changes must go through UAT before production rollout when they affect:

- Booking or assignment behavior.
- Escrow, payout, invoices, or settlements.
- TMS/API/webhook behavior.
- Auth, RBAC, tenant isolation, or admin workflows.
- Compliance gates or carrier eligibility.
- Document upload, document access, or retention.
- Customer-visible notifications.

UAT steps:

1. Define pilot tenant/users.
2. Define success criteria and rollback/escalation criteria.
3. Run staging validation with production-like data.
4. Run customer or internal operator UAT.
5. Record issues, support cases, and product feedback.
6. Approve or defer production rollout.
7. Monitor post-release adoption and support tickets.

## Post-Release Feedback Loop

For major releases, product/support must review:

- Support case volume and severity.
- CSAT or customer feedback.
- Workflow completion rate where measurable.
- Known issue count and age.
- Rollback/escalation triggers.
- Adoption by tenant and role.
- Follow-up documentation or training needs.

## Release Hold Criteria

Hold or roll back a release when:

- UAT finds a money, booking, tenant isolation, or data-integrity issue.
- Support cannot explain the customer impact.
- Rollback owner is unavailable.
- Required customer notice was not sent.
- A known issue lacks a workaround for active freight.
- Enterprise pilot customer rejects production promotion.

## Task Mapping

- `ENT-0106A` owns this release communication process.
- `ENT-0105` owns rollback.
- `ENT-0106` owns feature flags and kill switches.
- `ENT-0006` owns training/help-center handoff.
