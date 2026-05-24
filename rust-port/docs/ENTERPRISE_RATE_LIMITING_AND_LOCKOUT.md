# Enterprise Rate Limiting And Account Lockout

This document tracks `ENT-0202`.

## Implemented Rust Baseline

The backend now has a shared `RateLimiter` in `AppState` with:

- fixed-window request limits by named policy and identity
- repeated-failure lockout buckets
- forwarded-IP plus user-agent fingerprinting for unauthenticated flows
- account/email-based lockout identity for login and OTP verification
- success cleanup so valid login or OTP verification clears the failure bucket

Protected surfaces:

- login
- registration
- OTP verification
- OTP resend
- forgot password
- reset password
- profile KYC uploads
- onboarding KYC uploads
- dispatch load document uploads
- dispatch load document reads
- execution leg document uploads
- execution leg document reads
- Stripe Connect onboarding
- admin Stripe Connect onboarding
- escrow fund, hold, and release actions
- Stripe webhooks
- TMS status, bulk-status, and close webhooks

## Current Policy Defaults

- Auth flows: 10 attempts per 15 minutes per client fingerprint plus subject.
- Login and OTP lockout: 5 failed attempts in 15 minutes locks the subject for 15 minutes.
- Document uploads and reads: 60 attempts per hour per user and target.
- Payment actions: 30 attempts per hour per actor and target.
- Webhooks: 120 attempts per minute per client fingerprint.

## User Recovery Behavior

- Login and OTP failures return clear wait guidance when rate-limited or locked.
- Password recovery endpoints remain rate-limited to prevent enumeration and mail abuse.
- Successful login or OTP verification clears the corresponding lockout bucket.

## Distributed Store

The limiter uses the `security_rate_limits` Postgres table when a database pool is available. That gives all backend replicas the same counters and lockout state. If the database is unavailable, the backend falls back to process-local memory so local/dev flows can still run and tests stay deterministic.

Distributed properties:

- Atomic insert/update counter changes through Postgres `ON CONFLICT`.
- TTL-style expiry through `expires_at`.
- Shared lockout state through `locked_until`.
- Key namespace includes `stloads`, policy scope, and normalized identity.
- Future tenant namespacing should be added when tenant isolation lands.

## Support Visibility

Operators can inspect and clear active throttles without ad hoc database spelunking:

```powershell
./scripts/security_rate_limit_status.ps1 -DatabaseUrl $env:DATABASE_URL
./scripts/security_rate_limit_status.ps1 -DatabaseUrl $env:DATABASE_URL -Key "stloads:lockout:user@example.com"
./scripts/security_rate_limit_status.ps1 -DatabaseUrl $env:DATABASE_URL -Key "stloads:lockout:user@example.com" -Clear
```

The script requires `psql` and intentionally prints the counter key, count, window, lockout expiry, and update timestamp.

## Verification Completed

- `cargo fmt --check`
- `cargo test -p db`: 7 passed
- `cargo test -p backend`: 36 passed
- Unit tests cover fixed-window blocking, lockout behavior, success cleanup, and forwarded-IP fingerprinting.

## Required Next Step

Add dashboard-level security metrics and tenant namespacing after tenant isolation and observability foundations land.
