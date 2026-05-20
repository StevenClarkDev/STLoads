# STLoads Security Overview

## Purpose

This overview explains the security posture partners should expect from STLoads. It is written for product, operations, and partner review rather than as a replacement for source-level security documentation.

## Tenant Isolation

STLoads is designed around tenant-scoped operations. Production records should always be created, read, updated, and audited with tenant context.

Tenant isolation goals:

- users only see data authorized for their tenant and role
- marketplace records do not cross tenant boundaries without an explicit integration contract
- documents remain bound to owner, tenant, and approved access path
- operational recovery actions include tenant ID and source record ID

Any suspected tenant-boundary defect is a high-severity security incident.

## Role-Based Access Control

STLoads separates user capabilities by role and lifecycle state.

Primary access groups:

- admin
- operator
- shipper
- broker
- freight forwarder
- carrier
- pending or restricted account states

Role checks should control admin surfaces, onboarding review, payment actions, document visibility, booking actions, and operational write paths.

## Signed Integrations

External system integrations must use signed or otherwise authenticated requests. ATMP handoff, Stripe webhooks, and any future partner integrations must be validated before state changes are accepted.

Integration requirements:

- validate source
- validate signature or credential
- validate tenant
- validate idempotency
- reject malformed payloads
- audit accepted writes

## Audit Controls

Production write paths should leave audit evidence for support, compliance, and incident review.

Important audited events:

- account approval, rejection, and revision requests
- role or status changes
- load creation and update
- booking decisions
- document upload and protected access
- payment funding, release, and recovery
- queue replay and DLQ recovery
- integration handoff acceptance or rejection

Audit entries should include actor, tenant, target record, action, timestamp, and result.

## Document Controls

Documents are protected operational evidence. They should not be exposed through raw public links.

Document security controls:

- authenticated upload
- owner and tenant binding
- role-aware protected read routes
- object storage metadata validation
- restricted review access for admins and authorized users
- recovery process for missing metadata or missing objects

## Payment Controls

STLoads uses Stripe-oriented payment controls for marketplace payment flow.

Payment security expectations:

- signed webhook verification
- Stripe event ID tracking
- PaymentIntent state reconciliation
- Stripe Connect account readiness checks
- transfer release only when booking and closeout state allow it
- no manual payment state change without Stripe evidence

Payment incidents should be treated conservatively because duplicate charges, duplicate transfers, or unsupported manual state changes can create financial exposure.

## Runtime And Deployment Controls

STLoads staging deploys through IBM Code Engine with separate backend and frontend applications.

Runtime expectations:

- secrets are configured through Code Engine secrets
- source builds are repeatable through the deployment wrapper
- health and readiness endpoints are available
- revisions can be inspected and rolled back
- production configuration is not hardcoded into source

## Security Review Checklist

- tenant-scoped data access verified
- role gates verified for admin and operator actions
- signed webhook verification enabled
- document access is protected
- payment state follows Stripe evidence
- readiness and health checks pass
- deployment secrets are not committed
- support and recovery actions preserve audit history

