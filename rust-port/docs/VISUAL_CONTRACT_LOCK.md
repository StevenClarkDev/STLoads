# STLoads Visual Contract Lock

## Decision

The STLoads UI/UX source of truth is the developer-owned GitHub repository:

- Repository: `https://github.com/StevenClarkDev/STLoads.git`
- Locked visual commit: `a258e74082ae147f12c17ab793d9fffc236174a7`
- Locked by: ATMP/STLoads integration planning
- Reason: The ATMP-imported copy does not match the current developer UI/UX and must not drive product design.

## Repository Ownership

STLoads-owned work belongs in the STLoads repository:

- Local clean source: `C:\New folder\STLoads-api-review`
- Remote: `https://github.com/StevenClarkDev/STLoads.git`

ATMP-owned work belongs in the ATMP repository only when the change is part of ATMP Dispatch or the ATMP side of the STLoads API contract:

- Local ATMP source: `C:\New folder\atmp-os`
- Remote: `https://github.com/sabertech-development/atmp-os.git`

Do not commit STLoads frontend, backend, middleware, production-readiness, marketplace, or deployment implementation into ATMP unless the file is an ATMP Dispatch integration adapter, launcher, or API contract counterpart.

Do not use `C:\New folder\atmp-os-core-rebuild`.

## Non-Authoritative Copy

The imported copy at `C:\New folder\atmp-os\STLoads` is not authoritative for STLoads visual design. It is an integration artifact and currently differs from the GitHub UI/UX state.

Do not use it to make visual decisions.

## Protected Visual Areas

The following areas are protected by this visual contract:

- Login and authentication surfaces
- Public portal entry
- Dashboard frame
- Admin frame
- User frame
- Navigation structure
- Page spacing and density
- Color language
- Typography rhythm
- Logo and image assets
- Button and panel treatment
- Existing Leptos interaction model

## Allowed Frontend Changes

Frontend changes are allowed when they:

- Connect existing screens to real API data.
- Remove production-visible demo, mock, placeholder, or fake data.
- Replace fake data with honest empty states.
- Fix broken navigation or runtime errors.
- Add required production workflow fields inside the existing UI pattern.
- Improve accessibility or readability without changing the visual identity.
- Add loading, error, permission, and empty states required by real APIs.

## Forbidden Frontend Changes

Do not:

- Redesign STLoads to match ATMP Dispatch.
- Replace the developer layout system.
- Replace the developer navigation model.
- Change global colors, spacing, typography, or brand treatment without explicit approval.
- Convert STLoads into an embedded ATMP module.
- Use ATMP's visual language as a drop-in replacement.
- Treat `C:\New folder\atmp-os\STLoads` as the visual source of truth.

## Review Checklist

Before merging any STLoads frontend change, verify:

- Login screen is preserved.
- Dashboard frame is preserved.
- Admin frame is preserved.
- User frame is preserved.
- Navigation is preserved.
- Core color language is preserved.
- Existing assets still render.
- No production-visible demo data remains.
- Empty states are real and concise.
- API errors display without breaking layout.
- Mobile layout remains usable.

## Commit And Push Rule

Use the STLoads repository for STLoads changes:

```powershell
git -C "C:\New folder\STLoads-api-review" status --short --branch
git -C "C:\New folder\STLoads-api-review" add rust-port/docs/VISUAL_CONTRACT_LOCK.md
git -C "C:\New folder\STLoads-api-review" commit -m "docs: lock STLoads visual contract"
git -C "C:\New folder\STLoads-api-review" push origin HEAD
```

Use the ATMP repository only for ATMP-side integration files:

```powershell
git -C "C:\New folder\atmp-os" status --short --branch
```

If a change touches both products, split it into two commits in two repositories.
