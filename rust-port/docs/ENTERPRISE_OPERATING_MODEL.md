# Enterprise Operating Model

Last updated: 2026-05-24

This document supports `ENT-0004` in `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`. It defines the recommended first enterprise-release scope and the decisions that still need product, operations, finance, and legal approval.

## Recommended First Enterprise Release

STLoads should target a US-first logistics loadboard and broker-operations platform with marketplace features, rather than a fully global freight-forwarding platform on day one.

Recommended scope:

- Primary geography: United States domestic freight.
- Initial modes: FTL dry van, reefer, flatbed, and limited LTL support where the data model is ready.
- Initial customers: shippers, brokers, carriers, dispatchers/operators, finance users, support users, and admins.
- Product model: broker operating system plus private/public loadboard marketplace.
- Integration posture: API/webhook-first, with EDI support for enterprise partners that require it.
- Payment posture: Stripe-backed escrow, invoices, carrier settlements, credit controls, payout controls, and finance review queues.
- Mobile posture: mobile-first web/PWA execution before committing to native apps.

## Explicitly Deferred Unless Approved

These areas should not be promised for the first enterprise release without signed product/legal/ops approval:

- Full international freight forwarding.
- Full customs brokerage.
- Ocean, air, and rail-native workflows beyond explicit intermodal/drayage fields.
- Multi-country tax automation.
- Multi-currency money movement beyond documented FX/tax decisions.
- Native iOS/Android apps.
- White-label/custom-domain support unless `ENT-1206` is accepted for the target release.
- Factoring, fuel advances, fuel cards, or carrier advances unless `ENT-0907` is accepted for the target release.

## Supported Roles For First Enterprise Release

- Admin
- Support
- Operator/dispatcher
- Finance
- Shipper/customer
- Carrier
- Broker
- Integration admin
- Read-only auditor

Freight-forwarder and customs-broker roles should remain configurable but not promised as complete workflows until `ENT-0508`, `ENT-0509A`, and `ENT-0306` are approved for that scope.

## Operating Authority Decision

Open decision:

- Whether STLoads is software-only, broker of record, marketplace operator, freight forwarder, payment facilitator, or a mixed model.

Recommended path:

- Treat the first enterprise release as a broker-operations/loadboard platform.
- Do not market STLoads as customs broker, freight forwarder, or payment facilitator until `ENT-0306`, `ENT-0509A`, `ENT-0909`, `ENT-0910`, and `ENT-1006` are reviewed by legal/finance.

## Required Approval

This document is a recommendation, not final approval.

Required approvers:

- Product owner
- Operations owner
- Finance owner
- Legal/compliance owner
- Engineering owner

Approval evidence should be recorded here or linked from the work board before `ENT-0004` is marked complete.

## Current Status

- Recommendation written.
- Approved by project owner on 2026-05-24 as the working first-release operating model.
- Future legal, finance, or operations changes must update this document and reopen `ENT-0004` or create a follow-up task.
