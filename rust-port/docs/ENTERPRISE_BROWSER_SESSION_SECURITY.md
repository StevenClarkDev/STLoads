# Enterprise Browser Session Security

This document closes `ENT-0201A` by defining the browser authentication contract for the Rust loadboard and the controls required before any cookie-backed session model is introduced.

## Current Auth Transport

- The Rust backend authenticates browser API calls only through an explicit `Authorization: Bearer stl_<prefix>.<secret>` header.
- The backend does not issue session cookies, remember-me cookies, or auth cookies.
- Static evidence check: `rg` found no backend `Set-Cookie` auth path and found bearer-token usage in `crates/frontend-leptos/src/api.rs`, `device_location.rs`, and `document_upload.rs`.
- `bearer_token_from_headers` reads only the `Authorization` header. Cookie-only requests are treated as unauthenticated.

## CSRF Decision

CSRF is formally ruled out for the current auth transport because browsers do not attach the Rust bearer token to cross-site requests automatically. A cross-site form, image, script, or fetch request cannot authenticate unless attacker-controlled code can read and set the token, which is an XSS/token-storage problem rather than a CSRF problem.

Required controls while bearer transport remains active:

- Do not add auth-bearing cookies without completing the cookie migration gate below.
- Keep production CORS explicit. Wildcard CORS is forbidden by runtime validation.
- Keep logout server-side by deleting the hashed token record, not only by clearing client state.
- Rotate sessions on login, password reset, role changes, status changes, and role-permission changes.
- Treat token exposure as a credential incident and revoke all affected user tokens.

## Cookie Migration Gate

If the product later moves browser auth into cookies, the work must be reopened before release. Minimum cookie requirements:

- `HttpOnly`: required for every auth cookie.
- `Secure`: required in every non-local environment.
- `SameSite`: `Lax` by default; `Strict` for admin-only session cookies where UX allows; `None` only for approved cross-site embeds and only with `Secure`.
- `Domain`: host-only by default. Tenant custom domains must not share a parent-domain cookie unless a written tenant-boundary review approves it.
- `Path`: narrow to the application/API path that needs the cookie.
- `Expiry`: short absolute lifetime plus idle timeout; no unbounded persistent auth cookies.
- Rotation: rotate on login, MFA completion, SSO callback, privilege elevation, role changes, password reset, recovery, and tenant switch.
- CSRF: require synchronizer token or signed double-submit token for all state-changing browser requests.
- Logout: expire the browser cookie and invalidate the server-side session record.

## CORS And Custom Domains

Production and pilot environments must set all public URLs and allowed origins explicitly:

- `CORS_ALLOWED_ORIGINS`
- `PUBLIC_BASE_URL`
- `FRONTEND_PUBLIC_URL`
- `LOADBOARD_PUBLIC_URL`
- `ADMIN_PUBLIC_URL`
- `DISPATCH_PUBLIC_URL`

Operational rules:

- No wildcard origins in production.
- No origin reflection.
- No tenant custom domain is enabled until the exact domain is listed in the environment configuration and checked in staging.
- Uploads and document reads use the same bearer-token boundary as normal API calls.
- Webhooks remain server-to-server endpoints and must not rely on browser CORS for trust.

## Session Fixation Controls

The Rust session layer prevents fixation through server-side token rotation and revocation:

- Login deletes existing user tokens before issuing a new hashed bearer token.
- Password reset invalidates existing user tokens.
- Admin user role/status changes invalidate the changed user's sessions.
- Role-permission changes invalidate all users assigned to the changed role.
- Stored token material is a SHA-256 secret hash plus lookup prefix, not a usable bearer token.

## Required Verification

Completed checks:

- `rg` verified that the backend has no `Set-Cookie` auth issuer.
- Unit tests verify bearer parsing accepts explicit `Authorization: Bearer ...`.
- Unit tests verify cookie-only headers do not authenticate.
- Unit tests verify non-bearer authorization schemes do not authenticate.
- `ENT-0104` runtime validation rejects production wildcard CORS and missing public URL settings.
- `ENT-0201` tests verify token hashing, legacy token rejection, login rotation, password reset invalidation, and role-change invalidation paths.

Reopen this task if any future change introduces cookie auth, SSO callback cookies, embedded cross-origin apps, tenant shared-parent domains, or browser credential modes that send ambient credentials.
