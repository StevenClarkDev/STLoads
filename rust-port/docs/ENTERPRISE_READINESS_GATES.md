# Enterprise Readiness Gates

Last updated: 2026-05-24

This document defines the gates STLoads must pass on the way from the current Rust port to an enterprise-grade logistics loadboard. It supports `ENT-0003` in `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md`.

## Gate Rules

- A gate is not passed by opinion. It needs evidence.
- Evidence must link back to commits, tests, smoke checks, screenshots, runbooks, monitoring, issue IDs, or signed-off review notes.
- Failed criteria become follow-up tasks in `docs/ENTERPRISE_LOADBOARD_TASK_LIST.md` and, for `P0`/`P1`, `docs/ENTERPRISE_WORK_BOARD.md`.
- A later gate cannot override a failed earlier gate unless the failure has an explicit risk acceptance owner and expiration date.

## Gate 1: Alpha

Purpose: prove the Rust platform can be developed safely by the team.

Required evidence:

- `cargo fmt --all -- --check` passes.
- `cargo check --workspace` passes.
- `cargo test --workspace` passes or failures are linked to tracked blockers.
- `cargo clippy --workspace --all-targets -- -D warnings` passes or remaining lints are tracked in `ENT-1606`.
- The enterprise task list and work board are up to date.
- Local secrets stay untracked.
- P0/P1 tasks have owners, acceptance criteria, and verification notes.

Exit decision:

- Product and engineering agree the backlog is executable.
- No undocumented P0 security, tenant, payment, or data-loss risk is knowingly ignored.

## Gate 2: Beta

Purpose: prove core workflows work end to end in a controlled environment.

Required evidence:

- Auth, onboarding, load posting, matching, offer, booking, execution, documents, payments, TMS, notifications, support, and admin flows have smoke coverage.
- Tenant isolation and permission checks cover critical reads and writes.
- Stripe, SMTP, object storage, TMS, and frontend runtime configuration are validated in staging.
- Basic observability exists for HTTP, SQL, workers, email, storage, payments, and TMS.
- Support and operator workflows can run without direct database access.
- Known P0 defects are closed or formally risk accepted.

Exit decision:

- Team can run a complete freight lifecycle in staging with realistic data.
- Product accepts that the remaining scope is feature depth or hardening, not missing platform spine.

## Gate 3: Enterprise Pilot

Purpose: prove the platform can support selected enterprise users under close supervision.

Required evidence:

- Enterprise customer onboarding checklist exists and has been rehearsed.
- SSO/SCIM decision is documented; if in scope, pilot tenant path is tested.
- Carrier packet, compliance, document retention, legal agreement, and payment workflows are ready for pilot scope.
- Incident response, customer communication, support case, and escalation paths are documented.
- Backup/restore and business continuity exercises have evidence.
- Integration sandbox or pilot integration path is available.
- Accessibility, mobile, and browser support decisions are documented for pilot users.
- Data quality checks exist for tenant isolation, payment state, documents, TMS handoffs, and load lifecycle.

Exit decision:

- Pilot customer risks are understood and documented.
- Customer-facing limitations are visible before pilot launch.

## Gate 4: Production

Purpose: prove the platform can run normal customer traffic safely.

Required evidence:

- All P0 tasks are complete or explicitly deferred with signed risk acceptance.
- Production runtime guardrails prevent unsafe fallback mode.
- Migrations, rollback, feature flags, and kill switches are documented and tested.
- WAF/DDoS/bot protections are configured or formally deferred.
- Monitoring, alerts, on-call, runbooks, and status-page/customer communication paths are active.
- Payment idempotency, ledgering, reconciliation, credit controls, payout controls, and bank-change controls are active.
- Document access, scanning/quarantine, retention, legal hold, and export/delete behavior are implemented or scoped.
- API, webhook, EDI, and TMS flows have idempotency, replay, observability, and versioning rules.
- Security headers, CORS, CSRF/cookie/session controls, key management, PCI scope, and secret scanning are validated.

Exit decision:

- Engineering, operations, product, finance, legal/security, and support sign off.
- No open P0 defect or unowned P1 customer-impacting defect remains.

## Gate 5: Enterprise Ready

Purpose: prove the platform can pass enterprise procurement and operate at enterprise expectations.

Required evidence:

- Every item in the Enterprise Ready Completion Checklist is complete, explicitly deferred, or has signed risk acceptance.
- SOC 2 or ISO 27001 readiness plan exists with evidence owners and timeline.
- Penetration testing and vulnerability disclosure/remediation workflow are complete.
- Vendor/subprocessor inventory, DPA, data residency, privacy, retention, legal hold, and data-return processes are ready.
- SLAs, support tiers, support cases, CSAT/feedback, customer training, release notes, UAT, and offboarding are ready.
- Operating authority, surety/bond, corporate insurance, AML/payment obligations, tax/FX/cross-border rules, and jurisdiction decisions are complete or explicitly out of scope.
- Business continuity, disaster recovery, PITR/failover/restore strategy, tabletop exercises, and customer communication evidence are ready.
- Cost, quota, provider spend, tenant limits, and abuse controls are ready.

Exit decision:

- The team can answer enterprise procurement, security, operational, legal, financial, and integration questions from maintained evidence.
- The platform can be called enterprise-ready without relying on hidden tribal knowledge.
