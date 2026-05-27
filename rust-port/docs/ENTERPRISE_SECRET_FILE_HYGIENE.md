# Enterprise Secret File Hygiene

Last updated: 2026-05-27

This document defines the local credential policy for `ENT-1707`.

## Never Commit

The following must never be committed:

- `.env`, `.env.*`, except explicitly approved examples.
- `.env.ibm.secret`, `.env.ibm.runtime`, `.cos-*`, temporary IBM exports, and local cloud credential files.
- TLS private keys, certificate bundles with private material, SSH private keys, service-account exports, database dumps, and provider token exports.
- Screenshots or logs that contain credentials, tokens, API keys, reset links, OTPs, or webhook signatures.

## Allowed Examples

Example files may contain variable names and placeholder values only:

- `.env.ibm.example`
- deployment docs that list environment variable names without live values

## Local Developer Workflow

1. Store real credentials outside the repo or in the platform secret manager.
2. Use `.env.ibm.example` as the shape reference.
3. Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_ci_security.ps1 -SkipCargoAudit` before sharing changes.
4. If a credential is committed, pasted into chat, written to logs, or shared in a ticket, rotate it immediately and record the incident.

## CI And Hook Options

- CI runs `scripts/run_ci_security.ps1` and installs `cargo-audit`.
- Developers can run `scripts/run_pre_commit.ps1` locally before commit for the same repo hygiene checks.
- The security lane intentionally excludes build outputs and local target directories.
