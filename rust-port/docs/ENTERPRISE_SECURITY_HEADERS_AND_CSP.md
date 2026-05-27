# Enterprise Security Headers And CSP

Last updated: 2026-05-27

This document defines the browser security baseline for `ENT-1702`.

## Implemented Headers

The backend API adds these headers to responses:

- `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Permissions-Policy: camera=(), microphone=(), payment=(), usb=(), geolocation=(self)`

The frontend nginx container adds the same baseline plus a frontend CSP that allows the current required browser integrations:

- STLoads static assets from `self`
- Runtime config and Trunk/Leptos scripts from `self`
- Google Fonts and Font Awesome CDN styles/fonts
- Google Maps/Places
- Stripe hosted frames/API
- OpenStreetMap embedded route preview
- WebSocket connections for realtime

## Current CSP Exceptions

The frontend still allows `'unsafe-inline'` for script and style because the current Leptos/Trunk output and page shell include inline bootstrap/style content. This is accepted for this phase with the following follow-up:

- Move the large inline style block to a compiled stylesheet.
- Remove inline event attributes from the static shell.
- Add nonce or hash-based CSP for any remaining inline bootstrap script.
- Re-run Playwright and browser console checks after tightening CSP.

## Verification

- Backend unit test `app::tests::enterprise_security_headers_are_defined` verifies the API header baseline.
- Frontend nginx template includes the browser CSP and security headers.
- Playwright remains the browser smoke gate after header changes.
