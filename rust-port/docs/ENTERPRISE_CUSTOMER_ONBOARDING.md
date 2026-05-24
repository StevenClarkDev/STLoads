# Enterprise Customer Onboarding

Last updated: 2026-05-24

This document supports `ENT-0005` in `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`. It defines the repeatable onboarding path for enterprise shippers, brokers, carriers, and mixed organizations.

## Onboarding Principles

- No enterprise tenant goes live without an owner, checklist, and launch decision.
- Setup data must be collected before configuration begins.
- Compliance, billing, integrations, and support contacts must be confirmed before production use.
- Any unsupported workflow must be documented before customer launch.
- Onboarding evidence must be linked back to the work board or customer implementation record.

## Roles And Owners

| Area | Primary owner | Backup owner | Required before launch |
| --- | --- | --- | --- |
| Customer implementation | Customer Success/Ops | Product | Yes |
| Tenant and user setup | Support/Ops | Engineering | Yes |
| Roles and permissions | Product/Ops | Security | Yes |
| Billing and credit setup | Finance | Ops | Yes |
| Compliance setup | Compliance/Ops | Legal | Yes |
| Carrier packet setup | Ops | Support | If carriers are onboarded |
| Integrations | Engineering/Ops | Product | If API, webhook, EDI, TMS, ELD, or SSO is in scope |
| Training | Customer Success/Support | Product | Yes |
| Launch approval | Product/Ops | Engineering | Yes |

## Required Setup Data

Collect these before configuring the tenant:

- Legal customer name, DBA, tax ID if needed, billing address, and operating address.
- Primary business model: shipper, broker, carrier, freight forwarder, marketplace participant, or mixed.
- Target freight modes and geography.
- Required users, roles, departments, and approval chains.
- Admin contacts, finance contacts, operations contacts, support contacts, legal/security contacts, and integration contacts.
- Billing terms, payment method, credit limit, invoice contacts, PO/reference requirements, and tax reporting needs.
- Compliance requirements, insurance requirements, contracts, terms, DPAs, carrier packet rules, and document retention requirements.
- Private network rules: preferred carriers, blocked carriers, carrier groups, lanes, service levels, and customer-specific booking rules.
- Load data: lanes, facilities, commodities, equipment, accessorials, service levels, references, appointment rules, and document requirements.
- Notification preferences: in-app, email, SMS/push decisions, escalation rules, quiet hours, and operational templates.
- Integration needs: SSO/SCIM, API keys, webhooks, EDI transactions, TMS sync, ELD/telematics, accounting export, sandbox data, and cutover date.
- Support tier, SLA expectation, training plan, launch date, and pilot group.

## Repeatable Onboarding Checklist

### 1. Intake

- Assign onboarding owner.
- Confirm customer type and operating model fit.
- Confirm target launch scope and explicitly deferred scope.
- Create customer implementation record or issue.
- Link implementation record to `docs/ENTERPRISE_WORK_BOARD.md` where needed.

### 2. Commercial And Legal

- Confirm signed customer agreement, payment terms, privacy/DPA needs, tracking consent language, and support tier.
- Confirm any procurement evidence needed: insurance, security packet, SOC/ISO roadmap, penetration-test summary, vendor/subprocessor list.
- Confirm whether branded portal, custom domains, SSO/SCIM, EDI, or cross-border workflows are in scope.

### 3. Tenant Setup

- Create organization/tenant.
- Configure tenant name, status, domains, support tier, billing account, and operating geography.
- Configure roles, permissions, initial admins, finance users, operators, shippers, carriers, integration admins, and read-only auditors.
- Confirm MFA/SSO/SCIM requirements.

### 4. Master Data And Customer Rules

- Load or configure facilities, lanes, equipment, commodities, service levels, accessorials, references, and document requirements.
- Configure preferred carriers, blocked carriers, carrier groups, private network rules, and visibility rules.
- Configure credit rules, payment holds, approval thresholds, and finance contacts.
- Configure notification preferences, escalation paths, and operational templates.

### 5. Compliance And Documents

- Configure required agreements and acceptance workflow.
- Configure carrier packet requirements, COI rules, authority checks, W-9/tax workflow, and safety/compliance checks.
- Configure document retention, legal hold, export/delete behavior, and closeout package requirements.
- Validate upload, view, replace, scan/quarantine, and access-control behavior.

### 6. Integrations

- Configure sandbox tenant or demo tenant first.
- Configure API keys, webhook endpoints, EDI partners, TMS sync, ELD/telematics, SSO/SCIM, and accounting export as required.
- Validate idempotency, replay, rate limits, delivery logs, and alerting.
- Confirm production cutover window and rollback plan.

### 7. Training And Pilot

- Train admins, operators/dispatchers, finance, support contacts, shippers, and carriers as applicable.
- Run pilot load lifecycle: post, tender/offer, book, pickup, tracking, POD, closeout, invoice/settlement, and support case.
- Record gaps, owners, severity, and go/no-go decision.

### 8. Launch

- Confirm launch checklist with product, ops, support, finance, legal/compliance, and engineering.
- Enable production tenant access.
- Monitor first production loads, notifications, integrations, payments, documents, and support cases.
- Schedule post-launch review.

## Go/No-Go Criteria

Go only when:

- Required contracts and legal/privacy terms are complete.
- Tenant, roles, billing, compliance, master data, carrier network, and notifications are configured.
- Required integrations pass sandbox or pilot validation.
- Support owner and escalation path are known.
- Training is complete for launch users.
- Pilot workflow issues are resolved or explicitly accepted.

No-go if:

- Required compliance or legal evidence is missing.
- Payment, credit, payout, or bank-account controls are not ready for the launch scope.
- Required integrations cannot be observed, replayed, or rolled back.
- Tenant isolation or access controls are uncertain.
- Support cannot operate the customer without engineering intervention.

## Handoff

After launch:

- Customer Success owns adoption and renewal health.
- Support owns day-to-day customer issues.
- Ops owns freight workflow escalations.
- Finance owns billing, credit, AR, settlements, payouts, and disputes.
- Engineering owns product defects, integration defects, and incident remediation.
- Product owns scope decisions and roadmap feedback.

## Evidence To Store

- Completed onboarding checklist.
- Signed commercial/legal documents or links.
- Tenant configuration summary.
- User/role export.
- Billing and credit setup summary.
- Compliance setup summary.
- Integration validation results.
- Training attendance or completion notes.
- Pilot load evidence.
- Launch approval and post-launch review notes.
