# Enterprise Customer Success And Training

Last updated: 2026-05-24

This document supports `ENT-0006` in `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`. It defines the training, help-center, adoption, renewal, and support-handoff structure for enterprise customers.

## Goals

- Train enterprise users without engineering involvement.
- Give support a clear knowledge base before customer launch.
- Create a repeatable customer-success playbook from implementation through renewal.
- Turn customer friction into support cases, product feedback, and roadmap decisions.

## Training Tracks

| Track | Audience | Required before launch | Core topics |
| --- | --- | --- | --- |
| Admin training | Customer admins, implementation leads | Yes | Tenant setup, users, roles, permissions, SSO/SCIM decisions, audit, support contacts |
| Shipper training | Shipper users, customer operations | Yes if shipper is in scope | Load posting, private freight, lane guides, facility rules, documents, tracking, invoices |
| Carrier training | Carrier admins, dispatchers | Yes if carrier network is in scope | Carrier profile, packet requirements, offers, booking, tracking, POD, settlements |
| Dispatch/operator training | Internal or customer dispatch teams | Yes | Desk queues, assignment, exceptions, notes, closeout, escalation |
| Finance training | Customer finance, internal finance | Yes if billing/payment is in scope | Credit holds, invoices, settlements, payouts, disputes, exports, reconciliation |
| Support training | STLoads support and customer support contacts | Yes | Case intake, severity, SLA, escalation, customer updates, troubleshooting |
| Integration training | Integration admins, customer IT, partners | If integrations are in scope | API keys, webhooks, EDI, sandbox, replay, logs, versioning, cutover |

## Help-Center Structure

Create or maintain articles in these categories:

- Getting started
- Account access, MFA, SSO, and user roles
- Organization and tenant administration
- Load posting and bulk import
- Private networks, lane guides, and customer rules
- Carrier onboarding, packets, and compliance
- Offers, tenders, booking, and counteroffers
- Dispatch desks and exception management
- Driver/mobile execution, tracking, POD, and closeout
- Documents, templates, retention, legal hold, and exports
- Payments, credit holds, invoices, settlements, payouts, and disputes
- TMS, API, webhook, EDI, sandbox, and integration troubleshooting
- Notifications, preferences, delivery issues, SMS/push, and escalation
- Support cases, severity levels, SLA, and customer communication
- Release notes, known issues, maintenance windows, and roadmap feedback

## Customer Success Playbook

### Implementation

- Confirm onboarding checklist from `docs/ENTERPRISE_CUSTOMER_ONBOARDING.md`.
- Identify executive sponsor, operations owner, finance owner, admin owner, support contact, and integration owner.
- Record success criteria for the first 30, 60, and 90 days.
- Define pilot users and pilot lanes.
- Confirm unsupported workflows before launch.

### Launch

- Run training sessions by role.
- Confirm help-center links and support contacts are distributed.
- Monitor first production loads and first finance cycle.
- Track launch issues as support cases with severity and owner.
- Hold daily or weekly launch check-ins until the customer stabilizes.

### Adoption

- Review adoption metrics: active users, posted loads, booked loads, tracking compliance, document completion, payment cycle time, support cases, and failed integrations.
- Identify training gaps and update help-center articles.
- Confirm that customer admins can self-serve normal user and workflow questions.

### Renewal

- Review success criteria, SLA performance, incident history, support trends, product gaps, and upcoming roadmap needs.
- Confirm contract, billing, DPA, security questionnaire, and vendor/subprocessor updates.
- Convert renewal risks into tracked tasks.

### Escalation

- Define severity level and owner.
- Link issue to customer, tenant, load, invoice, integration, or incident.
- Provide customer-visible update cadence.
- Record root cause and follow-up task after resolution.

## Support Handoff

Sales/implementation must hand off these items to ongoing support:

- Customer summary and operating model.
- Tenant ID and production launch date.
- Named contacts and escalation path.
- Support tier and SLA terms.
- Training completion notes.
- Open risks, unsupported workflows, and accepted limitations.
- Active integrations and vendor contacts.
- Billing/credit contacts and finance rules.
- First 30-day monitoring plan.

## Minimum Launch Package

Before launch, the customer should receive:

- Role-specific training agenda.
- Help-center index.
- Support contact and escalation path.
- Known limitations and unsupported workflows.
- Release-note subscription or customer changelog path.
- Integration support path if applicable.
- Go-live checklist and post-launch review date.

## Evidence To Store

- Training attendance or recording links.
- Help-center article list.
- Support handoff record.
- Customer success plan.
- Pilot results.
- Launch issue log.
- 30/60/90-day adoption notes.
- Renewal risk notes.
