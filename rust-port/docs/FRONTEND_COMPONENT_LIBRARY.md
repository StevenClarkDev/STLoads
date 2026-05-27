# Frontend Component Library

Phase 13 source of truth for new Leptos UI work.

## Required Defaults

New enterprise screens should start with the primitives in `crates/frontend-leptos/src/components` before adding page-specific inline styles.

- `UiPageHeader`: page title, eyebrow, and short operational context.
- `UiPanel`: plain 8 px panel for grouped controls or repeated operational sections.
- `UiToolbar`: action rows with consistent spacing.
- `UiFilterBar`: filters/search controls with an accessible region label.
- `UiTableShell`: horizontally safe table wrapper.
- `UiStatusPill` and `UiBadge`: status and compact metadata indicators.
- `UiToast`: non-blocking status feedback with `role="status"` and polite live-region behavior.
- `UiModal`, `UiDrawer`, and `UiConfirmDialog`: dialog shells with modal roles and focusable containers.
- `UiTimeline`: event history scaffolding.
- `UiFileUploadFrame`: document and image upload shell.
- `UiMapPanel`: map/route placeholder shell.
- `UiMoneyInput`: currency input shell.
- `UiFieldError`: accessible form validation message.

## Design Rules

- Keep cards and panels at `8px` radius unless a legacy PHP shell class is already controlling the surface.
- Use status pills or badges for state, not ad hoc colored spans.
- Use `UiTableShell` for tables wider than mobile viewports.
- Put keyboard-accessible buttons, links, and form controls in `UiToolbar` or `UiFilterBar`.
- Avoid nested panels. If a section needs hierarchy, use headings, table sections, or timeline rows.

## Migration Path

Existing large pages may keep legacy markup until they are touched for Phase 13 refactors. Any new screen or major edit should move repeated inline status/table/panel patterns to the shared primitives.
