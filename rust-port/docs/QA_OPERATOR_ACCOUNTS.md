# QA Operator Accounts

Last updated: 2026-04-18

Disposable operator accounts prepared for Rust staging parity QA.
These are for IBM staging only and are safe to recreate with `scripts/seed_operator_qa_accounts.ps1`.

## URLs

- PHP app: `https://portal.stloads.com`
- Rust backend: `https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud`
- Rust frontend: `https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud`

## Rust Staging Accounts

| Purpose | Role | Email | Password | Final status |
| --- | --- | --- | --- | --- |
| Admin smoke | admin | `admin.smoke@stloads.test` | `AdminPass123!` | Approved |
| Shipper smoke | shipper | `shipper.smoke@stloads.test` | `ShipperPass123!` | Approved |
| Carrier smoke | carrier | `carrier.smoke@stloads.test` | `CarrierPass123!` | Approved |
| Broker happy path | broker | `broker.qa@stloads.test` | `BrokerQaPass123!` | Approved |
| Freight-forwarder happy path | freight_forwarder | `forwarder.qa@stloads.test` | `ForwarderQaPass123!` | Approved |
| Pending OTP flow | shipper | `pending.otp.qa@stloads.test` | `PendingOtpQa123!` | Pending OTP |
| Pending review flow | carrier | `pending.review.qa@stloads.test` | `PendingReviewQa123!` | Pending Review |
| Revision-requested flow | shipper | `revision.requested.qa@stloads.test` | `RevisionQa123!` | Revision Requested |
| Rejected flow | broker | `rejected.qa@stloads.test` | `RejectedQa123!` | Rejected |

## Notes

- The six QA-specific accounts were created and re-verified through the hosted Rust backend using `scripts/seed_operator_qa_accounts.ps1`.
- The pending-OTP account also passed the Rust OTP resend path.
- The Rust frontend is now hosted on IBM Code Engine at the URL listed above, so browser QA can start from a real Leptos deployment instead of a local build.
- PHP-side equivalents for pending OTP, pending review, revision requested, and rejected are still not confirmed from the hosted PHP environment, so QA-002 remains open until matching operator sessions are available on `https://portal.stloads.com`.
