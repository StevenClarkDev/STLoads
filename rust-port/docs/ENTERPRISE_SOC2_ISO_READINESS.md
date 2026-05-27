# Enterprise SOC 2 Or ISO 27001 Readiness Program

This document closes `ENT-1712` by defining the compliance-readiness decision, control map, evidence ownership, and certification timeline posture.

## Readiness Decision

For the first enterprise release, STLoads should operate a SOC 2-style readiness program and defer formal certification unless a signed enterprise customer requires SOC 2 Type I, SOC 2 Type II, or ISO 27001 before launch.

If certification becomes contractually required:

- SOC 2 Type I is the recommended first external target.
- SOC 2 Type II should follow after a stable observation window.
- ISO 27001 should be evaluated if target customers require an ISMS certification rather than SOC 2.

## Control Map

| Control area | Evidence owner | Evidence examples |
| --- | --- | --- |
| Access management | Security/Engineering | MFA policy, role matrix, access reviews, deprovisioning records |
| Change management | Engineering/Ops | PRs, CI results, approvals, release notes, deployment logs |
| Incident response | Security/Ops/Legal | Incident tickets, tabletop evidence, customer notices, postmortems |
| Vendor management | Security/Legal/Product | Vendor inventory, DPAs, reviews, approvals, subprocessor notices |
| Backups and recovery | DevOps/Backend | Backup config, restore tests, RPO/RTO evidence, PITR/failover plan |
| Encryption and secrets | Security/Backend/DevOps | Key-management docs, rotation evidence, secret-scan results |
| Logging and monitoring | DevOps/Security | Audit logs, alert rules, dashboards, SIEM/log-drain evidence |
| Vulnerability management | Security/Engineering | Dependency scans, secret scans, pentest reports, remediation tickets |
| Data retention and privacy | Legal/Product/Backend | Retention matrix, privacy-request records, deletion/export manifests |
| Tenant isolation | Engineering/Security | Tests, threat model, code review evidence, customer boundary checks |
| Availability | DevOps/Ops | Uptime metrics, incident history, runbooks, on-call rotations |
| Customer communication | Customer Success/Legal/Ops | Maintenance notices, SLA reports, release communications |

## Evidence Repository

Evidence should be kept in a restricted compliance workspace with:

- Policy documents and approved versions.
- CI/security reports.
- Access-review exports and approvals.
- Vendor review records.
- Incident and tabletop records.
- Backup/restore test evidence.
- Penetration-test reports and remediation evidence.
- Release approvals and customer communications.

Do not store secrets, raw payment credentials, private keys, or unrestricted customer documents in the evidence repository.

## Timeline

| Milestone | Target |
| --- | --- |
| Internal SOC 2-style readiness map | Before enterprise pilot |
| Evidence owners assigned | Before enterprise pilot |
| Critical control gaps logged in task list | Before enterprise pilot |
| Readiness assessment | After core enterprise controls are implemented |
| SOC 2 Type I decision | Before first contract requiring certification |
| SOC 2 Type II observation window | After Type I or after stable production operations |

## Procurement Answer

Current posture: STLoads is building against SOC 2-style control expectations with documented ownership and evidence collection. Formal SOC 2 or ISO 27001 certification is not represented as complete unless an auditor engagement, readiness assessment, and certification report exist.

## Completion Criteria

This readiness program is complete when the control map, owners, evidence storage rules, and certification/deferral posture are visible to procurement and tracked as operational work. Formal certification remains separate from readiness.
