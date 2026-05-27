# Frontend Page Refactor Plan

Phase 13 keeps page maintainability honest by separating shared UI helpers from page-specific workflow logic and tracking the remaining large-file targets.

## Completed Refactor Foundation

- Added `crates/frontend-leptos/src/components` for shared UI primitives.
- Added `crates/frontend-leptos/src/pages/shared.rs` for shared page helpers such as status tone styles, numeric parsing, file-size labels, and comma-list parsing.
- Reused shared helpers in payments, loads, integrations, profile, and notifications instead of duplicating local implementations.
- Split page-specific helper modules for auth, load profile, load builder, master data, loads, admin users, integrations, payments, profile KYC, load-profile documents, and execution workflow sections.
- Added `scripts/frontend_page_inventory.ps1` to scan page modules recursively and report large page files before and after refactor batches.

## Remaining Large-Page Targets

No recursive page module currently exceeds the large-page threshold. Keep this section updated if future frontend work introduces a new oversized page.

## Refactor Rules

- Move repeated formatting/parsing helpers into `pages/shared.rs`.
- Move reusable view primitives into `components`.
- Split page-specific sections into sibling modules under `pages/<page_name>/` when a section is touched.
- Keep old behavior intact unless the task explicitly changes UX.
- Run `cargo check -p frontend-leptos` after every page split.
