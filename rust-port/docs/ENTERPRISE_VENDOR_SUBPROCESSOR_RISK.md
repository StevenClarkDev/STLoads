# Enterprise Vendor, Subprocessor, And Third-Party Risk Management

This document closes `ENT-1709` by defining the vendor inventory, approval workflow, review cadence, and customer-answering rules.

## Inventory

| Vendor/subprocessor | Purpose | Data processed | Region/residency notes | Owner | Status |
| --- | --- | --- | --- | --- | --- |
| IBM Cloud or selected production cloud | Hosting, compute, database, networking, object storage | Customer data, logs, backups, documents, runtime metadata | Must match contracted production region | DevOps/Security | Target provider |
| Stripe | Payments, invoices, subscriptions, payouts where used | Payment tokens/IDs, invoices, customer billing metadata, webhook events | Region governed by Stripe contract and customer DPA | Finance/Backend | In scope |
| SMTP/email provider | Transactional email | Email address, message metadata, email contents | Provider and region must be approved before production | Product/Ops | Provider TBD |
| Maps/geocoding provider | Geocoding, route display, facility lookup | Addresses, coordinates, route metadata | Provider terms and region must be approved | Product/Backend | Provider TBD |
| ELD/telematics providers | Tracking and vehicle/driver integrations | Location, equipment, driver/device metadata, event timestamps | Customer-specific approval required | Product/Integrations | Customer-specific |
| EDI/TMS providers | Load, tender, status, document, invoice integrations | Shipment, partner, invoice, document, and event data | Customer-specific approval required | Integrations | Customer-specific |
| GitHub or source-control provider | Source code and CI metadata | Code, issues, CI logs, limited operational metadata | No production secrets allowed in source | Engineering | In scope |
| Monitoring/logging/SIEM provider | Logs, metrics, alerts, traces, audit/security event export | Operational logs and telemetry, redacted where required | Must support contracted residency and retention | DevOps/Security | Provider TBD |
| Support/help-center provider | Customer tickets, help material, support history | Contact data, support details, limited operational context | Customer PII exposure must be controlled | Customer Success | Provider TBD |
| Analytics provider | Product analytics and adoption metrics | Usage events and metadata, no secrets/payment credentials | Must be approved before production | Product | Optional/TBD |

## Approval Workflow

New vendors cannot be used in production until the owner records:

- Business purpose and affected product area.
- Data categories processed and whether any sensitive, payment, location, identity, document, or regulated data is involved.
- Customer/tenant impact and whether the vendor is a subprocessor.
- Region, retention, encryption, deletion, incident notification, and support access behavior.
- Contract, DPA, security documentation, insurance or certification evidence if required.
- Data export/deletion support and offboarding plan.
- Security and Legal approval.

## Review Cadence

- Review critical subprocessors before launch and at least annually.
- Review customer-specific EDI, ELD, telematics, and TMS vendors before enabling each customer integration.
- Review vendor access quarterly when vendor users can access customer data.
- Re-review a vendor after breach notice, material service change, region change, DPA change, or new sensitive data use.

## Customer Disclosure

Enterprise customers receive a current vendor/subprocessor list during onboarding and when procurement requests it. Material additions or changes must follow the customer contract and DPA notice process.

## Enforcement

Production configuration, integration setup, and procurement must reject unapproved vendors. If an urgent operational need requires temporary use, Legal and Security must sign a time-boxed risk acceptance with customer-impact notes.
