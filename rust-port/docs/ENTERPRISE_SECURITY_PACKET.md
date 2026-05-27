# Enterprise Security Packet

This document closes `ENT-1705` by giving Sales, Support, Customer Success, Security, and Legal a stable security-questionnaire source for enterprise customers. Customer-specific answers must still be reviewed before sharing externally.

## Customer-Facing Summary

STLoads is an enterprise logistics load board and TMS integration platform. The security posture is based on tenant isolation, least-privilege access, strict state machines, auditability, encryption, secret hygiene, controlled integrations, and documented operational procedures.

## Evidence Index

| Area | Source |
| --- | --- |
| Threat model | `docs/ENTERPRISE_THREAT_MODEL.md` |
| Security headers and CSP | `docs/ENTERPRISE_SECURITY_HEADERS_AND_CSP.md` |
| Data classification and encryption | `docs/ENTERPRISE_DATA_CLASSIFICATION_AND_ENCRYPTION.md` |
| Key management and PCI scope | `docs/ENTERPRISE_KEY_MANAGEMENT_AND_PCI_SCOPE.md` |
| Secret file hygiene | `docs/ENTERPRISE_SECRET_FILE_HYGIENE.md` |
| Privacy/data requests | `docs/ENTERPRISE_PRIVACY_DATA_REQUEST_WORKFLOW.md` |
| Customer onboarding | `docs/ENTERPRISE_CUSTOMER_ONBOARDING.md` |
| Customer offboarding | `docs/ENTERPRISE_CUSTOMER_OFFBOARDING.md` |
| Release environments | `docs/ENTERPRISE_RELEASE_ENVIRONMENTS.md` |
| Reliability runbooks | `docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md` |
| Vendor/subprocessor inventory | `docs/ENTERPRISE_VENDOR_SUBPROCESSOR_RISK.md` |
| Data residency and DPA | `docs/ENTERPRISE_DATA_RESIDENCY_DPA.md` |
| Pentest and vulnerability disclosure | `docs/ENTERPRISE_PENTEST_AND_VULNERABILITY_DISCLOSURE.md` |
| SOC 2/ISO readiness | `docs/ENTERPRISE_SOC2_ISO_READINESS.md` |

## Standard Questionnaire Answers

### Hosting

Production hosting is designed around managed cloud infrastructure, containerized application services, managed database services, object storage, TLS, environment-specific runtime configuration, CI validation, and release gates. The current target environment is documented in `docs/ENTERPRISE_RELEASE_ENVIRONMENTS.md`.

### Tenant Isolation

Tenant and organization boundaries are product-critical controls. Tenant-scoped workflows must never expose cross-tenant loads, documents, offers, execution events, invoices, support records, audit entries, integrations, or master data. Tenant isolation remains a launch blocker until every customer-facing and support-facing workflow has tests or explicit acceptance evidence.

### Authentication And Access Control

The platform uses authenticated sessions, hashed session tokens, MFA/step-up requirements for privileged workflows, role-based authorization, and privileged access-review processes. Access reviews and stale-access revocation remain tracked in the enterprise task list until fully complete.

### Encryption

All production traffic must use TLS. Database, object storage, backups, and provider-managed secrets must use managed encryption at rest. Application-level encryption is required only for fields approved as needing additional protection. Logs, analytics, support views, and exports must redact secrets, payment credentials, and unnecessary private data.

### Payment And PCI Boundary

STLoads minimizes PCI scope by relying on Stripe-hosted/tokenized payment flows and by prohibiting raw card, bank, or payment credential storage in STLoads systems. Stripe IDs, webhook event IDs, ledger references, and invoice/settlement records may be stored. Raw payment credentials are prohibited.

### Backups And Retention

Backup, restore, retention, PITR, and failover expectations are tracked as enterprise readiness gates. Retention and deletion behavior must follow the privacy workflow, customer offboarding workflow, finance/tax obligations, legal hold, safety, insurance, fraud, and dispute requirements.

### Incident Response

Security incidents must be triaged by severity, logged with evidence, routed to Security/Engineering/Legal as needed, and communicated to customers according to contract, law, and materiality. Incident response testing is still tracked as a separate enterprise readiness item.

### Vulnerability Management

CI security checks include dependency and secret scanning. Third-party penetration testing and vulnerability-disclosure workflow are defined in `docs/ENTERPRISE_PENTEST_AND_VULNERABILITY_DISCLOSURE.md`; actual third-party test execution remains a pre-launch blocker until scheduled, completed, and remediated or risk-accepted.

### Logging And Audit

The platform is expected to audit auth, admin, loads, documents, offers, execution, payments, TMS/integrations, support actions, and master-data changes. Logs and audit exports must not include secrets, raw payment credentials, or unnecessary private data.

### Subprocessors

Vendors and subprocessors are inventoried in `docs/ENTERPRISE_VENDOR_SUBPROCESSOR_RISK.md`. New vendors require Security, Legal, Product, and owner review before production use.

## Sharing Rules

- Share this packet only after customer name, environment, hosting region, and contract scope are confirmed.
- Do not promise certifications, regions, uptime, controls, pentest dates, or subprocessors that are not in the current evidence set.
- Mark WAF/DDoS/bot protection and penetration testing as pre-launch controls until provider configuration and external testing evidence exist.
- Escalate custom questionnaire answers involving insurance, legal terms, breach notification, data residency, retention, or regulated freight to Legal and Security.

## Known Pre-Launch Gaps

- Edge WAF/DDoS/bot protection must be configured with the selected production edge provider.
- Third-party penetration testing must be scheduled, completed, and remediated or risk-accepted.
- Incident response tabletop testing must be completed.
- Final tenant-isolation, privileged-access, backup/restore, and observability readiness items remain governed by `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`.
