# Enterprise Threat Model

Last updated: 2026-05-27

This document is the Phase 17 threat model for STLoads. It tracks enterprise launch risks, required mitigations, and follow-up owners for auth, payments, documents, TMS/API integrations, admin/support tooling, and tenant isolation.

## Scope

In scope:

- Browser frontend, backend API, WebSocket/realtime routes, hosted workers, database, object storage, email, Stripe, TMS/API integrations, and admin/support tooling.
- Enterprise tenants, STLoads operators, shippers, carriers, brokers, freight forwarders, drivers, external TMS/API partners, and support staff.
- Data classes documented in `docs/ENTERPRISE_DATA_CLASSIFICATION_AND_ENCRYPTION.md`.

Out of scope for this pass:

- Third-party penetration testing execution. That is tracked separately in `ENT-1711`.
- Formal SOC 2 or ISO 27001 audit. That is tracked separately in `ENT-1712`.

## Critical Assets

| Asset | Primary Risk | Current Controls | Required Evidence |
|---|---|---|---|
| Session tokens and MFA artifacts | Account takeover, session replay | Hashed bearer tokens, rotation on sensitive changes, step-up/MFA docs | Auth tests and access-review tests |
| Tenant freight data | Cross-tenant disclosure or mutation | Tenant-scoped queries, organization memberships, admin break-glass | Tenant isolation tests and audit events |
| Uploaded documents and closeout packages | Unauthorized disclosure, malicious files | Storage abstraction, path traversal guards, validation/scanner hook, role checks | Document access tests and upload validation tests |
| Payment and payout workflows | Duplicate release, fraud, PCI scope expansion | Stripe tokenization, webhook signature validation, idempotency, ledger, two-person finance release | Payment tests, webhook tests, PCI boundary doc |
| TMS/API integrations | Forged webhooks, replay, duplicate side effects | Shared secrets, idempotency keys, event dedupe, replay logs, sandbox lifecycle | Integration tests and API lifecycle docs |
| Admin/support tooling | Privilege abuse, unsafe support access | RBAC, MFA/step-up, access reviews, support audit notes | Access review tests and support search audit tests |
| Realtime and tracking | Privacy breach, location over-collection | Tracking consent, retention labels, scoped execution access | Execution consent tests and privacy workflow doc |

## Threats And Mitigations

| ID | Area | Threat | Severity | Mitigation Status | Evidence / Follow-up |
|---|---|---|---|---|---|
| TM-001 | Auth | Password login remains active for verified SSO domains | High | Mitigated for active OIDC domains | `enterprise_sso_discovery_blocks_password_login_when_routing_is_active` |
| TM-002 | Auth | Stolen bearer token remains usable after privilege change | High | Mitigated by token revocation paths | SCIM/access-review/elevation tests |
| TM-003 | Tenant isolation | User reads another organization's loads/documents | Critical | Mitigated by tenant-scoped query tests, still needs broader full-route audit | Track residual in Phase 17 checklist until route audit is complete |
| TM-004 | Documents | Path traversal reads local files | Critical | Mitigated | Document storage tests from Phase 4 |
| TM-005 | Documents | Dangerous upload reaches operators/customers | High | Partially mitigated by validation and scanner hook | Real malware scanner integration remains later provider work |
| TM-006 | Payments | Raw card data enters STLoads systems | Critical | Mitigated by Stripe-hosted/tokenized boundary | `docs/ENTERPRISE_KEY_MANAGEMENT_AND_PCI_SCOPE.md` |
| TM-007 | Payments | Duplicate webhook or repeated release creates duplicate payout | Critical | Mitigated by idempotency, webhook dedupe, ledger and approval tests | Payment tests from Phases 9 and 16 |
| TM-008 | TMS/API | Forged partner webhook mutates dispatch/payment state | Critical | Mitigated by shared secret/API signature paths | TMS/API lifecycle tests |
| TM-009 | Admin/support | Support search exposes sensitive data without audit | High | Mitigated | Support search scoped/audited backend test |
| TM-010 | Browser | Missing headers allow framing, sniffing, or broad referrer leakage | High | Mitigated in Phase 17 | Backend header test and frontend nginx config |
| TM-011 | Secrets | Local IBM/COS/TLS secrets enter git | Critical | Mitigated by `.gitignore` and CI scan; local hook optional | `scripts/run_ci_security.ps1` and `docs/ENTERPRISE_SECRET_FILE_HYGIENE.md` |
| TM-012 | Location | Driver tracking collected without consent or retained too long | High | Partially mitigated by consent workflow | Privacy workflow and retention implementation still need deeper operational validation |

## Review Cadence

- Review after every auth, payment, document, TMS/API, support, or tenant-isolation change.
- Review before any enterprise pilot.
- Add new threats to this table before marking related enterprise tasks complete.
