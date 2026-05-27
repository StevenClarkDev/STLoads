# Enterprise Reliability Runbooks

Phase 15 makes STLoads operable under real customer volume. The system now has database-backed reliability catalogs for observability, alerts, jobs, query controls, backup/restore, archive policy, incident runbooks, continuity exercises, and cost guardrails.

## Runtime Modes

`STLOADS_RUNTIME_MODE` controls process responsibility:

- `web`: serve HTTP only; do not start background workers.
- `worker` or `workers`: start background workers only; do not bind an HTTP listener.
- `all`: serve HTTP and start workers, useful only for local development or explicitly approved small deployments.

Production should run web and worker services separately so scaling web traffic does not duplicate email, TMS, reporting, or integration side effects.

## Structured Logs And Traces

Production uses JSON logs by default. Set `LOG_FORMAT=json` explicitly in production and configure `RUST_LOG` with route/module filters. The OpenTelemetry export endpoint is represented by `OTEL_EXPORTER_OTLP_ENDPOINT`; traces should cover HTTP, SQL, Stripe, object storage, SMTP, TMS, and worker surfaces as providers are attached.

## Metrics And Alerts

The `observability_signal_catalog` and `alert_rules` tables define required signals and P0/P1 alert ownership for:

- HTTP latency and error rate
- Database pool usage
- Queue lag and worker outcomes
- Email and webhook failures
- Object storage errors
- Payment failures
- TMS drift

Alert routing is governed by `on_call_escalation_policies`.

## Worker Or Queue Incident

Use `background_jobs` as the durable queue contract. Jobs move through `queued`, `running`, `retry_scheduled`, `succeeded`, `dead_letter`, and `cancelled`. Dead-letter rows are operational evidence and must be reviewed before replay.

First 15 minutes:

- Identify queue, worker, tenant, and error class.
- Stop unsafe replay if side effects may duplicate.
- Review dead letters and external idempotency records.
- Communicate customer impact if freight movement, payments, or TMS updates are delayed.

## API Or Auth Outage

First 15 minutes:

- Declare incident and assign commander.
- Check `/health/live`, `/health/ready`, request error rate, auth errors, and recent deploys.
- Freeze deploys and risky feature flags.
- Roll back or disable affected auth/SSO/MFA path if safe.

## Database Outage

Recovery target is governed by `backup_restore_policies`. First release is restore-oriented, not automatic multi-region failover unless a customer contract explicitly changes that posture.

## Object Storage Outage

Use document-upload kill switch if writes are unsafe. Queue or manually collect POD/BOL evidence, then reconcile document metadata and hashes after recovery.

## Payment Incident

Freeze payout release if money movement risk exists. Reconcile Stripe events, ledger entries, invoices, settlements, and approval records before replaying any failed action.

## Duplicate Booking

Stop the affected booking surface, identify the source-of-truth booking, notify impacted carriers/customer, and record the final decision in audit evidence.

## TMS Outage

Queue handoffs, pause unsafe pushes if needed, maintain customer-visible status updates, and replay only after source-of-truth conflicts are resolved.

## Email Outage

Use in-app notification and support fallback. Preserve outbox state and replay only when provider rejection or suppression state is understood.

## Data Exposure

Start the security incident path, preserve logs, stop exposure, rotate/revoke access as needed, export audit evidence, and coordinate legal/customer notification decisions.

## Bad Deploy

Roll back the affected revision, validate readiness, run smoke tests, and capture missing regression coverage in the post-incident review.

## Backup, Restore, RPO, And RTO

Policies are stored in `backup_restore_policies`:

- PostgreSQL: 15 minute RPO, 240 minute RTO target.
- Object storage: 60 minute RPO, 360 minute RTO target.
- Derived search/reporting: rebuildable after operational restore.
- Queues: durable rows replayed after duplicate-safety checks.

## Archiving

Policies are stored in `archive_policies` for location pings, messages, audit events, TMS handoffs, and document metadata.

## Business Continuity

`continuity_exercises` tracks planned tabletop exercises for regional provider outage, payment provider outage, TMS outage, and email/SMS outage. Evidence should include gaps found, follow-up owners, and remediation decisions.

## Cost And Usage Guardrails

`usage_quota_policies` and `provider_spend_controls` define fair-use limits, alert routes, budget thresholds, and approval requirements for heavy usage or provider spend spikes.
