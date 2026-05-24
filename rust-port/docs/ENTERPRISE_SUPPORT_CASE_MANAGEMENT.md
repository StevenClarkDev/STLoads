# Enterprise Support Case Management And Feedback Loop

This document defines the enterprise support-case model for STLoads. It is the operating contract for `ENT-0009` and must be updated when support tooling, SLA rules, or feedback workflows change.

## Goal

Enterprise support must be managed as accountable cases with owners, severity, SLA clocks, customer-visible status, internal notes, linked operational context, and product feedback loops.

Chat and email can be channels, but they are not the system of record.

## System Of Record Decision

Initial enterprise build target: implement STLoads-native support cases with integration points for external systems.

External systems such as Zendesk, Intercom, Linear, Jira, or Slack may be connected later, but the product must still maintain tenant-linked case records, SLA state, audit references, and feedback classification.

## Case Model

Each support case must track:

- Case number and stable public identifier.
- Tenant, account, office, and customer contacts.
- Reporter user and affected users.
- Related load, bid, shipment, invoice, settlement, document, integration, webhook, API request, or incident.
- Channel: portal, email, phone, chat, integration, internal escalation, or incident follow-up.
- Severity: SEV-1 through SEV-4.
- Status: new, triage, waiting_on_customer, waiting_on_stloads, escalated, engineering_review, product_review, resolved, closed, reopened.
- Owner, team, and escalation owner.
- SLA target, first-response deadline, next-update deadline, resolution target, and breach state.
- Customer-visible updates and private internal notes.
- Resolution reason, root-cause category, and follow-up action.
- CSAT or equivalent customer feedback result.

## Required Workflows

### Intake

- Create a case from portal, support channel, incident follow-up, integration alert, or internal escalation.
- Auto-link tenant, user, and operational context when available.
- Require severity, category, owner, and customer impact before moving out of triage.

### SLA Management

- Start SLA clock at case creation or first qualified customer contact.
- Pause only when contractually allowed, such as waiting_on_customer.
- Show breach risk before breach.
- Notify support lead before breach.
- Notify escalation owner after breach.
- Preserve breach history even if severity later changes.

### Escalation

- Escalate SEV-1 and SEV-2 immediately to support lead and engineering on-call.
- Link incidents when customer impact affects availability, data integrity, payments, integrations, or security.
- Escalate recurring cases to product review.
- Escalate contractual or financial disputes to legal/finance.

### Customer Updates

- Customer-visible updates must show current status, owner/team, next update time, and resolution summary.
- Internal notes must never leak to customer-visible views.
- Case closure must include resolution reason and any remaining customer action.

### Feedback Loop

- Collect CSAT or equivalent feedback after resolution for enterprise cases.
- Tag feedback by product area, workflow, severity, customer segment, and recurring pain.
- Review recurring case patterns in product planning.
- Link high-impact feedback to roadmap items or known gaps in the enterprise work board.

## Reporting Requirements

- Open cases by tenant, severity, owner, and age.
- SLA breach risk and actual breach history.
- First response and resolution-time trends by support tier.
- Recurring issue categories by product area.
- Cases linked to incidents, payments, integrations, and data-quality issues.
- CSAT trend by tenant and support tier.
- Enterprise readiness blockers derived from support volume and severity.

## Tooling Requirements

- Support cases must be tenant-scoped and RBAC-protected.
- Case activity must be auditable.
- Customer-visible comments and private notes must be stored separately.
- Case attachments must use enterprise document-storage controls.
- Cases must link to incidents, audit events, loads, invoices, integrations, and roadmap feedback.
- External helpdesk sync must preserve the STLoads case identifier.

## Verification Checklist

- A support case can be created for a tenant and linked to a load, invoice, integration, or incident.
- SLA deadlines are calculated from support tier and severity.
- Customer-visible updates exclude internal notes.
- Breach-risk reporting works before an SLA breach occurs.
- Resolved cases require resolution reason and feedback classification.
- Recurring case reports can be reviewed by product and operations.

## Task Mapping

- `ENT-0009` defines this operating process.
- Later support-case implementation, SLA dashboards, notification, incident, and roadmap-feedback tasks must reference this document before completion.
