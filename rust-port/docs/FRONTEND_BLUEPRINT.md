# Frontend Blueprint

This document translates the current Blade and jQuery frontend into a Leptos migration shape.

## Current UI Shells

- Auth shell:
  - `resources/views/auth/app.blade.php`
- User shell:
  - `resources/views/layout/app.blade.php`
- Admin shell:
  - `resources/views/admin-layout/app.blade.php`

## First Pages To Port

### Phase 1

- login and admin login
- OTP and password reset
- register flows by role

### Phase 2

- `resources/views/dashboard.blade.php`
- `resources/views/load/index.blade.php`
- `resources/views/load/load_profile.blade.php`

### Phase 3

- `resources/views/load/add.blade.php`
- user profile and onboarding pages under `resources/views/users`

### Phase 4

- dispatch desk pages under `resources/views/desk`
- STLOADS operations pages under `resources/views/stloads`

### Phase 5

- admin dashboard and approvals
- CRUD pages for users, roles, and reference data

### Phase 6

- `resources/views/chat/index.blade.php`
- `resources/views/load/track.blade.php`

## Shared Components To Rebuild In Leptos

- `AuthFrame`
- `UserFrame`
- `AdminFrame`
- `SidebarNav`
- `TopBar`
- `Breadcrumbs`
- `ToastHost`
- `StatusBadge`
- `DataTable`
- `Pagination`
- `FormField`
- `FileUploadField`
- `Modal`
- `ConfirmDialog`

The current Blade equivalents live in:

- `resources/views/layout/sidebar.blade.php`
- `resources/views/admin-layout/sidebar.blade.php`
- `resources/views/layout/header.blade.php`
- `resources/views/components`

## JS-Heavy Hotspots

These are the frontend areas that should not be treated as simple template conversions:

- `resources/views/load/index.blade.php`
  - booking modals
  - Stripe payment modal
  - carrier preference modal
  - export and filtering behavior
- `resources/views/load/add.blade.php`
  - dynamic multi-leg builder
  - Google Maps autocomplete
- `resources/views/chat/index.blade.php`
  - realtime messages
  - realtime offers
  - modal-driven offer flows
- `resources/views/load/track.blade.php`
  - live tracking map
  - browser geolocation

## SSR First vs Interactive

Good SSR-first pages:

- dashboards
- desk views
- STLOADS operations, reconciliation, and sync errors
- admin CRUD lists and forms
- load detail pages

Client-interactive islands:

- chat and offer state
- tracking map
- Stripe funding flow
- Google Maps autocomplete
- OTP resend and countdown
- dynamic multi-leg form builder

## Target Frontend Structure

- `frontend/app.rs`
- `frontend/layouts/`
- `frontend/components/`
- `frontend/pages/auth/`
- `frontend/pages/dashboard/`
- `frontend/pages/loads/`
- `frontend/pages/desks/`
- `frontend/pages/stloads/`
- `frontend/pages/chat/`
- `frontend/pages/users/`
- `frontend/pages/admin/`
- `frontend/integrations/`
- `frontend/state/`

## Frontend Risks

- The shells load a lot of global vendor JS and theme logic today.
- `chat/index.blade.php` is already a mini SPA embedded in Blade.
- `load/index.blade.php` mixes too many workflows into one page and should be decomposed during the port.
- Several browser integrations will need JS interop wrappers instead of full Rust-native replacements:
  - Stripe
  - Google Maps
  - Leaflet
  - emoji picker

## Immediate Frontend Sequence

1. Build the auth, user, and admin shells in Leptos.
2. Port dashboard and load list/detail pages with SSR.
3. Port onboarding and load-create flows.
4. Port dispatch and STLOADS operational dashboards.
5. Port chat, offers, and tracking as interactive feature modules.
