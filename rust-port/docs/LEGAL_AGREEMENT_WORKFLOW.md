# Legal Agreement Acceptance Workflow

This document supports `ENT-0305`.

## Production Decision

For the first enterprise release, STLoads will use in-platform clickwrap acceptance with immutable
audit evidence for platform terms, privacy policy, tracking consent, payment terms, carrier operating
agreements, broker/customer contract terms, shipper customer contract terms, and freight forwarder
terms.

A dedicated e-signature provider is not required for the first release unless Legal later classifies
a specific customer contract as needing signature ceremony features such as signer identity proofing,
multi-party countersignature, certificate of completion, or negotiated redlines. If that happens,
Legal/Product should open a new integration task for DocuSign, Dropbox Sign, Adobe Sign, or the
selected provider instead of overloading the clickwrap flow.

## Evidence Captured

Each acceptance stores:

- Agreement key and version.
- Agreement title, document URI, and content hash snapshot.
- User and organization context.
- Signer user ID, signer name, and signer email.
- Timestamp, IP address, user agent, and request ID when available.
- JSON evidence snapshot.
- Linked `audit_events` row.

## Blocking Rule

Onboarding submission is blocked until the current user has accepted all active required agreements
for their role and organization context. Updated agreement versions become missing automatically
because acceptance is stored against the specific template/version row.

## Rollout Process

1. Legal publishes final content and document location.
2. Backend updates `legal_agreement_templates` through a migration with the final `content_sha256`.
3. Product confirms role targeting and whether organization-level acceptance is required.
4. Frontend shows the missing agreements through `/auth/legal-agreements`.
5. QA verifies onboarding blocks before acceptance and resumes after acceptance.
6. Ops exports proof through audit search when a customer or legal request requires evidence.
