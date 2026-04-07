# Canonical Status Model

This document is the first Rust-side normalization pass for STLoads lifecycle state.

The important rule is simple: the Rust port should not copy the Laravel integer codes as-is when they currently mix different business concerns into the same field.

## Roles

Observed role ids in the Laravel app:

- `1` = admin
- `2` = shipper
- `3` = carrier
- `4` = broker
- `5` = freight forwarder

Rust mapping lives in `crates/domain/src/auth.rs` as `UserRole`.

## Account Status

Observed user account codes:

- `0` = email verified, onboarding not yet submitted
- `1` = approved
- `2` = rejected
- `3` = pending admin review
- `4` = pending OTP verification
- `5` = revision requested

Rust mapping lives in `crates/domain/src/auth.rs` as `AccountStatus`.

Recommended transition shape:

1. `PendingOtp`
2. `EmailVerifiedPendingOnboarding`
3. `PendingReview`
4. `Approved` or `Rejected` or `RevisionRequested`
5. `RevisionRequested` returns the user to the onboarding flow, then back to `PendingReview`

## Offers

Observed offer status codes:

- `0` = declined
- `1` = pending
- `3` = accepted

Rust mapping lives in `crates/domain/src/marketplace.rs` as `OfferStatus`.

Recommended transition shape:

1. `Pending`
2. `Accepted` or `Declined`

## Escrow

Observed escrow states are string-based instead of integer-based:

- `unfunded`
- `funded`
- `released`
- `refunded`
- `on_hold`
- `failed`

Rust mapping lives in `crates/domain/src/payments.rs` as `EscrowStatus`.

Recommended transition shape:

1. `Unfunded`
2. `Funded`
3. `Released` or `Refunded` or `Failed`

`OnHold` should be treated as an operational exception state, not a normal happy-path milestone.

## STLOADS Handoff

Observed handoff statuses:

- `queued`
- `push_in_progress`
- `published`
- `push_failed`
- `requeue_required`
- `withdrawn`
- `closed`

Observed TMS dispatch statuses:

- `dispatched`
- `in_transit`
- `at_pickup`
- `at_delivery`
- `delivered`
- `cancelled`
- `invoiced`
- `settled`

Rust mapping lives in `crates/domain/src/tms.rs` as `HandoffStatus` and `TmsStatus`.

## Load And Leg Lifecycle

This is the area with the most drift.

Observed legacy `load_legs.status_id` codes in Laravel:

- `0` = draft or hidden pre-release state
- `1` = new or created
- `2` = approved or actively reviewed
- `3` = still in pre-booking flow
- `4` = booked
- `5` = pickup started
- `6` = arrived at pickup
- `7` = departed pickup or in transit
- `8` = escrow funded and also treated as the gate before pickup starts
- `9` = arrived at delivery
- `10` = delivered or completed
- `11` = paid out / financially complete

Why this cannot be copied directly:

- booking state and execution state are in the same integer field
- finance readiness is also encoded in that same field
- controllers disagree on whether `4` or `8` is the true pre-pickup booked state

Rust direction:

- keep the raw integer as an import concern only
- split runtime meaning into:
  - `LegPostingStatus`
  - `LegExecutionStatus`
  - `EscrowStatus`

Current Rust placeholders live in `crates/domain/src/dispatch.rs`:

- `LegPostingStatus`
- `LegExecutionStatus`
- `LegacyLoadLegStatusCode`

The intended normalized flow is:

1. posting flow: `Draft` -> `OpenForReview` -> `OpenForOffers` -> `Booked`
2. funding gate: `EscrowStatus::Unfunded` -> `EscrowStatus::Funded`
3. execution flow: `ReadyForPickup` -> `PickupStarted` -> `AtPickup` -> `InTransit` -> `AtDelivery` -> `Delivered` -> `PaidOut`

## Recovery Work Still Needed

- confirm the live contents of `load_status_master`
- confirm the live contents of `offer_status_master`
- recover the actual schema for `loads`, `load_legs`, `offers`, `user_details`, and reference-data tables
- reconcile the `conversations` migration with the runtime `load_leg_id` model
- confirm whether finance completion belongs on `load_legs`, `escrows`, or both
