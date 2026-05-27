# Frontend Accessibility Baseline

Target level: WCAG 2.2 AA for core enterprise workflows.

## Baseline Controls

- Every app shell exposes a `Skip to main content` link.
- Main content regions are focusable with `tabindex="-1"` so skip links land predictably.
- Global `:focus-visible` styling is required for links, buttons, inputs, navigation, and custom controls.
- `prefers-reduced-motion: reduce` disables nonessential animation and transitions.
- Dialog primitives use `role="dialog"` or `role="alertdialog"`, `aria-modal="true"`, accessible labels, and focusable containers.
- Toast/status messages use `role="status"` and `aria-live="polite"`.
- Form errors use `role="alert"`.
- Tables wider than the viewport must use a scroll-safe wrapper.

## Manual Spot Checks

Before marking a screen production ready, check:

- The page can be traversed by keyboard without losing visible focus.
- The first tab stop reaches the skip link.
- Modal or drawer content has a clear label and can be reached by keyboard.
- Buttons and links have visible text or an accessible name.
- Validation errors appear near the failing input and are announced.
- Status colors are paired with text, not used as the only signal.
- Text and controls remain legible at mobile widths and browser zoom.

## Current Exceptions

- Legacy large pages still contain repeated inline styles. They remain scheduled for ENT-1302 refactor.
- Automated browser accessibility and screenshot regression coverage is scheduled for ENT-1305.
