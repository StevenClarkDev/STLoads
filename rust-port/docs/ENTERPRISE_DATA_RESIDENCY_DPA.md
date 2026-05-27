# Enterprise Data Residency, DPA, And Regional Requirements

This document closes `ENT-1710` by defining the first-release residency position, DPA posture, and engineering boundaries.

## First-Release Position

The first enterprise release should support a single approved production region per customer environment. Until another region is formally approved, customer production data is treated as US-region data and must stay in the selected US production region for primary database, object storage, backups, logs, and operational exports.

Any EU, Canada, Mexico, UK, or other regional commitment requires a contract amendment, provider review, support-process review, and engineering approval before launch.

## Data Residency Commitments

For each enterprise customer, Sales/Legal must record:

- Contracted region.
- Data categories in scope.
- Whether backups, object storage, logs, analytics, support tickets, and monitoring are included in the residency commitment.
- Approved subprocessors and their regions.
- Retention and deletion requirements.
- Cross-border transfer mechanism if any.

Engineering must not enable customer production data in an unapproved region or provider.

## DPA And Privacy Posture

The DPA or privacy addendum should cover:

- STLoads role as processor/service provider for customer-controlled logistics data, except where STLoads acts as independent controller for platform operations, billing, fraud/security, legal compliance, and product administration.
- Processing purposes: logistics workflows, load board operations, TMS/EDI integrations, payments, support, security, compliance, analytics where approved, and customer success.
- Subprocessor approval and notice workflow.
- Security controls, encryption, access control, audit logging, incident notification, retention, export, deletion, and return of data.
- Data-subject request routing through the enterprise customer when the customer is controller.
- Deletion and backup behavior after termination or approved privacy requests.

## Region Rules By Data Store

| Store/process | Residency rule |
| --- | --- |
| Primary database | Must run in the contracted production region. |
| Object storage/documents | Must run in the contracted production region unless customer contract approves otherwise. |
| Backups/PITR | Must stay in approved backup region set and follow retention. |
| Logs and audit events | Must stay in approved logging region; secrets and unnecessary PII must be redacted. |
| Analytics | Disabled or limited unless vendor, region, and DPA terms are approved. |
| Support tickets | Must avoid unnecessary sensitive data; provider region must be disclosed if customer data is included. |
| Payment provider | Governed by payment-provider terms; STLoads stores tokens and ledger references, not raw payment credentials. |
| TMS/EDI/ELD vendors | Customer-specific region and transfer approval required. |

## Regional Privacy Applicability

The first target customer base appears US-focused. Still, STLoads must be prepared to evaluate:

- CCPA/CPRA for California personal information.
- GDPR/UK GDPR when EU/UK personal data or customers are in scope.
- Canadian privacy requirements when Canadian customers, drivers, or logistics records are in scope.
- Mexican or cross-border privacy requirements when Mexico-related operations are in scope.

Legal must make the applicability decision per customer segment and contract.

## Engineering Guardrails

- Environment names, region, database URL, object bucket, log drain, and backup target must be recorded for each production environment.
- Customer-specific regional commitments must be visible in onboarding and deployment checklists.
- New vendors require vendor/subprocessor review before receiving customer data.
- Exports, deletion jobs, support access, and analytics must honor the approved region and DPA scope.

## Completion Criteria

This document is complete for first-release planning. Launch remains blocked if the actual production provider, buckets, databases, backup targets, or log drains do not match the contracted region for a customer.
