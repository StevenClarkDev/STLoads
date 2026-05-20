# STLoads Support Playbook

## Purpose

This playbook gives support staff a practical first-response guide for common STLoads issues. Every support case should capture tenant, user, account type, load ID, booking ID, payment ID, document ID, browser, timestamp, and the user-visible error.

## Common Carrier Issues

Carrier cannot see loads:

1. Confirm the carrier is approved.
2. Confirm email verification and onboarding are complete.
3. Confirm role is carrier or an approved carrier-equivalent role.
4. Confirm the load is published, active, and visible to that carrier class.
5. Confirm there are no tenant or lane filters excluding the carrier.

Carrier cannot book a load:

1. Confirm carrier approval state.
2. Confirm load status allows booking.
3. Confirm no existing booking or duplicate offer conflict exists.
4. Confirm required documents or payment prerequisites are satisfied.
5. Escalate if booking state and UI state disagree.

Carrier execution update fails:

1. Confirm the booking belongs to the carrier.
2. Confirm the load leg is active.
3. Confirm required closeout proof is present for delivery completion.
4. Confirm backend health and readiness.

## Login Problems

User cannot sign in:

1. Confirm email spelling.
2. Confirm account exists.
3. Confirm account is not rejected, disabled, or pending a required step.
4. Use password reset if credentials are unknown.
5. For OTP-pending users, resend OTP through the approved admin path.

User lands on onboarding instead of dashboard:

1. Confirm account lifecycle state.
2. If pending onboarding or revision requested, this behavior is expected.
3. If approved user is still blocked, refresh session state and check role permissions.

Admin cannot access admin surfaces:

1. Confirm user role.
2. Confirm role-permission matrix.
3. Confirm session refresh is current.
4. Escalate as an access-control incident if permissions appear broader than expected.

## Document Problems

Upload fails:

1. Confirm file size and type are allowed.
2. Confirm user is authenticated.
3. Confirm user owns the document context or has admin review rights.
4. Confirm object storage health.
5. Check backend logs for storage or multipart errors.

Document cannot be viewed:

1. Confirm metadata row exists.
2. Confirm object exists in storage.
3. Confirm tenant and owner match.
4. Confirm protected read route allows the user role.
5. Never send raw object links outside the protected STLoads flow.

Wrong document uploaded:

1. Preserve audit history.
2. Ask the user to upload the corrected document.
3. Mark the prior document superseded or rejected if that control is available.
4. Do not delete evidence required for payment or dispute review.

## Payment Problems

Payment intent failed:

1. Confirm Stripe status.
2. Confirm STLoads payment record.
3. Confirm webhook event receipt.
4. Ask the payer to retry only if Stripe state shows no successful charge.

Escrow not funded after payment:

1. Find the Stripe payment intent.
2. Confirm `payment_intent.succeeded` event exists.
3. Confirm webhook signature accepted.
4. Resend the Stripe event if STLoads has not recorded it.

Carrier transfer failed:

1. Confirm carrier has completed Stripe Connect onboarding.
2. Confirm account capabilities allow transfers.
3. Confirm booking and closeout state allow release.
4. Confirm no duplicate transfer exists.
5. Escalate before any manual correction.

## Booking Disputes

1. Capture load, carrier, shipper or broker, booking, payment, and document IDs.
2. Freeze payment release if dispute is payment-impacting.
3. Review status history, documents, execution notes, and timestamps.
4. Ask both parties for missing proof through controlled channels.
5. Record final decision with audit detail.

## Sync Failures

ATMP-to-STLoads load handoff fails:

1. Confirm ATMP sent the request to the correct STLoads endpoint.
2. Confirm signature or integration credential is valid.
3. Confirm tenant and idempotency key are present.
4. Confirm the payload validates against the STLoads contract.
5. Replay only if the target record was not already created.

STLoads status does not appear in ATMP:

1. Confirm STLoads has the updated status.
2. Confirm outbound dispatch integration is configured.
3. Confirm retry queue or failed event state.
4. Replay only idempotent events after confirming no duplicate downstream write.

## Escalation Rules

Escalate immediately for:

- suspected tenant data leak
- admin access broader than expected
- payment double-charge or double-transfer risk
- missing object storage evidence for a payment dispute
- recurring 5xx responses
- failed readiness check
- webhook signature failure in production

## Support Tone

Support should be clear, calm, and operational. Do not promise payment changes, account approval, or dispute outcomes until the record has been reviewed.

