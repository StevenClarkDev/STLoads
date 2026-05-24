# Enterprise MFA And Step-Up Controls

This document closes `ENT-0203` for the current Rust authorization model.

## MFA Method

The first production MFA method is email OTP for privileged login. It uses the existing enterprise mail/outbox path and stores only SHA-256 hashes of MFA codes.

Flow:

- Password login succeeds for a privileged account.
- Backend creates an `mfa_challenges` row with a hashed 6-digit code and 10-minute expiry.
- Backend emails the MFA code and returns no bearer token.
- User submits `/auth/mfa/verify`.
- Backend consumes the challenge and issues a normal bearer token with the `mfa_verified` capability in token abilities.
- Recovery codes are generated the first time MFA succeeds and are shown once.

## Protected Accounts

The current Rust role model requires MFA for admin users. Finance, operator lead, and integration-admin responsibilities currently collapse into admin permissions in this slice; when separate roles are introduced, they must be added to `privileged_user_requires_mfa`.

## Recovery Codes

MFA recovery codes are stored as hashes in `mfa_recovery_codes`.

- Recovery codes can be used in the MFA verification form.
- A used recovery code is marked with `used_at`.
- MFA-verified sessions can regenerate recovery codes through `/auth/mfa/recovery-codes/regenerate`.
- Regeneration deletes the old set and returns a new one-time display set.

## Step-Up Requirements

The `mfa_verified` token ability is required before high-risk actions:

- user deletion
- user role/status changes
- role-permission changes
- escrow release
- admin profile KYC document deletion
- recovery-code regeneration

## Frontend Contract

Login now supports an MFA challenge response:

- `mfa_required`
- `mfa_challenge_id`
- `mfa_expires_at`
- `next_step`
- `dev_mfa_code` in non-production only

The Leptos frontend adds `/auth/mfa`, verifies the challenge, stores the returned bearer token, and navigates to the user dashboard.

## Verification

- `cargo fmt --check`
- `cargo check -p frontend-leptos`
- `cargo test -p backend routes::auth::tests::admin_login_requires_mfa_before_session_token`
- `cargo test --workspace`

## Future Hardening

Add authenticator-app TOTP or WebAuthn/passkeys after tenant isolation and device-management foundations land. Email OTP is the initial enterprise-control baseline because the current Rust port already has auditable email outbox behavior and no device enrollment surface yet.
