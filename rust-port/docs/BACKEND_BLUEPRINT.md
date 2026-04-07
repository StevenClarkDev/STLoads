# Backend Blueprint

This document translates the current Laravel backend into a Rust migration shape.

## Current Backend Centers

- Identity and onboarding:
  - `app/Http/Controllers/AuthController.php`
  - `app/Http/Controllers/UserController.php`
  - `app/Http/Controllers/AdminController.php`
  - `app/Models/User.php`
  - `app/Models/KycDocuments.php`
- Dispatch core:
  - `app/Http/Controllers/LoadController.php`
  - `app/Http/Controllers/DashboardController.php`
  - `app/Http/Controllers/DispatchDeskController.php`
  - `app/Models/Load.php`
  - `app/Models/LoadLeg.php`
- Marketplace:
  - `app/Http/Controllers/OfferController.php`
  - `app/Http/Controllers/ConversationController.php`
  - `app/Http/Controllers/BidChatController.php`
- Execution and tracking:
  - `app/Http/Controllers/LoadLegController.php`
  - `app/Models/LegEvent.php`
  - `app/Models/LoadLegLocation.php`
  - `app/Models/LegDocuments.php`
- Payments:
  - `app/Http/Controllers/CarrierPayoutController.php`
  - `app/Http/Controllers/EscrowController.php`
  - `app/Http/Controllers/StripeWebhookController.php`
- TMS and STLOADS:
  - `app/Http/Controllers/Api/TmsInboundController.php`
  - `app/Http/Controllers/Api/TmsWebhookController.php`
  - `app/Http/Controllers/StloadsOperationsController.php`
  - `app/Services/StloadsReleaseGate.php`
  - `app/Services/StloadsSyncMonitor.php`
  - `app/Services/StloadsReconciler.php`

## Rust Domain Split

- `identity`
  - login, logout, OTP, password reset, onboarding, KYC, admin approvals
- `rbac`
  - roles, permissions, policy checks
- `dispatch`
  - loads, legs, locations, reference data, histories, carrier preference matching
- `marketplace`
  - offers, booking, conversations, messages
- `execution`
  - pickup and delivery state transitions, tracking points, leg documents, leg events
- `payments`
  - Stripe Connect, escrow funding, escrow release, webhook handling
- `tms`
  - inbound handoff APIs, webhook reconciliation, sync monitoring, operations support
- `audit`
  - application activity log and second database log handling

## Route Surfaces To Preserve

- Browser routes from `routes/web.php`
  - auth and onboarding
  - admin and CRUD pages
  - load and dispatch pages
  - chat, offers, tracking
  - STLOADS operations pages
- Service and token routes from `routes/api.php`
  - Stripe and payout endpoints
  - TMS push, queue, requeue, withdraw, close
  - TMS webhook status, cancel, close
- Scheduled reconciliation from `routes/console.php`

## Main Blocker: Schema Recovery And Dialect Pivot

The repo does not contain create-table migrations for a large part of the running domain, and the first Rust DB pass was written in MySQL dialect because it mirrored the current Laravel environment.

Committed migrations create:

- users, sessions, password resets
- cache and jobs
- roles and permissions
- conversations and messages
- sequences
- escrows
- personal access tokens
- STLOADS handoff tables

But application code also depends on tables such as:

- `loads`
- `load_legs`
- `locations`
- `countries`
- `cities`
- `offers`
- `load_documents`
- `leg_documents`
- `leg_events`
- `leg_locations`
- `load_history`
- `user_history`
- `user_details`
- `shipper_detail`
- `carrier_preferences`
- status master tables

## Known Drift To Resolve Before Porting

- Chat schema mismatch:
  - migration uses `load_id` in `database/migrations/2025_07_17_075048_create_message_and_convo_tables.php`
  - runtime code uses `load_leg_id` in `app/Models/Conversation.php`
- User relation mismatch:
  - onboarding uses `$user->details` in `app/Http/Controllers/AuthController.php`
  - `app/Models/User.php` has no `details()` relation
- Status lifecycle ambiguity:
  - booking and escrow status transitions are hard-coded across:
    - `app/Http/Controllers/OfferController.php`
    - `app/Http/Controllers/LoadLegController.php`
    - `app/Http/Controllers/EscrowController.php`
- Logging depends on a second database via `app/Models/Logs.php` and `config/database.php`
- Rust DB layer still assumes MySQL SQLx types, placeholders, and functions while the target environment is PostgreSQL on IBM.

## Backend Target Shape

Recommended medium-term crate layout:

- `crates/domain`
- `crates/db`
- `crates/http-web`
- `crates/http-api`
- `crates/realtime`
- `crates/integrations/stripe`
- `crates/integrations/tms`
- `crates/jobs`
- `apps/server`

The current scaffold starts smaller and can grow into this:

- `crates/backend`
- `crates/domain`
- `crates/frontend-leptos`
- `crates/shared`

## Immediate Backend Sequence

1. Extract the live current MySQL schema from the primary and secondary databases.
2. Define the target PostgreSQL schema for IBM deployment.
3. Define canonical Rust enums for user status, load leg status, offer status, escrow status, handoff status, and TMS status.
4. Port SQLx from MySQL-specific types, migrations, and queries to PostgreSQL.
5. Build the Rust persistence layer around the PostgreSQL target schema.
6. Implement auth, session, token, and RBAC foundations.
7. Port dispatch core.
8. Port marketplace and tracking.
9. Port payments and TMS integrations.
