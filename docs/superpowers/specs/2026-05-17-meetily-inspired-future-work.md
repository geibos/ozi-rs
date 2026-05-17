# Meetily-Inspired Future Work — Backlog

- Date: 2026-05-17
- Status: backlog, no commitments
- Parent design: [2026-05-17-shadcn-ui-kit-svelte-design.md](./2026-05-17-shadcn-ui-kit-svelte-design.md)
- Relation: captures everything **not** in the parent UI-kit initiative, so the
  "make ozi-rs feel like meetily" idea is documented end-to-end without
  bloating the active scope.

## Purpose

The original ask — "I like meetily's look, speed, and code layout, can we
get close?" — decomposes into three axes. Only one (visual UI-kit) is
addressed by the parent design. This document holds the other two as
proper backlog entries, plus the smaller follow-ups that fell out of the
UI-kit brainstorm. Each entry is sized as a future OpenSpec change.

Entries are descriptions, not commitments. Priorities and ordering are
decided when (and if) we revisit them.

---

## Axis A — Perceived speed & UX feel

What "fast" actually means in meetily is not raw frame-rate; it is a set
of perceived-speed techniques layered onto the UI. Svelte 5 + shadcn-svelte
makes all of them straightforward.

### A1 — Command palette (Cmd+K / Ctrl+K)

Global searchable launcher: jump to any track or waypoint, fire common
actions (import GPX, export, toggle theme, switch bundle, open recent
project), navigate between routes once Settings/History exist. Powered by
the shadcn `Command` primitive (it gets installed in Change 2 of the
parent design, so the runtime is already there).

Impact: huge. Single biggest perceived-speed win for power users.

### A2 — Keyboard-first navigation

Hotkeys for the most-used flows (`N` new waypoint, `T` new track, `1..4`
panel switch, `Esc` deselect/close, `?` show shortcuts overlay). Focus
traps in dialogs, focus restore on close, visible focus rings everywhere.
Tab order audit across all panels.

Impact: medium-high. Reduces mouse-trip overhead during a SAR operation.

### A3 — Optimistic updates

Mutations (rename track, recolour, add waypoint, drag-edit) update the UI
immediately and reconcile when the Tauri IPC roundtrip resolves. Today
we wait for `state-changed` events; this introduces a one-frame
roll-forward / roll-back pattern in `api.ts` wrappers.

Impact: medium. Most visible during fast drag/edit sequences.

### A4 — Skeleton states & progress

While bundles load, tiles fetch, projects open, exports run — show
skeletons (shadcn `Skeleton`) and explicit progress bars (`Progress`)
instead of empty containers or blocked UI. Long-running native commands
emit progress events back through Tauri.

Impact: medium. Removes the "is it frozen?" feeling.

### A5 — Motion and transitions

Tasteful Svelte transitions between routes (`/` ↔ `/project`), `slide-in`
for panels, `fade` for toast hosts, gentle micro-animations on
select/delete. Built on Svelte's native transitions plus
`tailwindcss-animate`; no external motion library needed (decision D in
parent design).

Impact: low-medium. Pure polish, but signals quality.

### A6 — Empty and error states

Illustrated empty states ("No tracks yet — import a PLT, draw one, or
load an example"), inline error states with retry, recoverable toast
errors via `svelte-sonner`. Currently empty panels show nothing.

Impact: medium. First-run experience.

### A7 — Hover and focus affordances

Subtle hover lift on cards, focus rings everywhere, cursor changes over
the map for the active interaction mode (drawing / editing / waypoint /
measurement). Some of this is already wired; the audit completes it.

Impact: low. Cumulative polish.

### Suggested OpenSpec breakdown for Axis A

If/when this axis is picked up, a reasonable split is:

- `add-command-palette` (A1 alone — high value, isolated)
- `add-keyboard-shortcuts` (A2 + A7 focus piece)
- `add-optimistic-updates` (A3 — touches `api.ts` and stores; do alone)
- `add-loading-and-progress-states` (A4 + A6 — share infrastructure)
- `add-motion-and-transitions` (A5 — pure polish, smallest)

---

## Axis B — Repository layout

Meetily uses `backend/` (Rust) + `frontend/` (Next.js) at the repo root.
We currently use `src-tauri/` (Rust) + `src/` (Svelte) — the default
Tauri scaffold layout.

### B1 — Rename top-level directories

`src-tauri/` → `backend/`, the SvelteKit `src/` → `frontend/src/` (i.e.
the frontend gets its own subtree). The Cargo workspace continues to
point at `backend/Cargo.toml`; `tauri.conf.json` and `vite.config.ts`
are updated to the new paths; `justfile` recipes follow.

Pros:
- Symmetric with meetily and the broader polyglot-monorepo industry pattern
- Reads better when grep'ing for "all backend changes" / "all frontend changes"
- Aligns with adding a third top-level surface later (e.g. `mobile/`, `docs-site/`)

Cons:
- Tauri's documentation, examples, scaffolds, and most community help
  assume `src-tauri/` — every external recipe needs translation
- Breaks every external link to source files
- Disrupts contributors' muscle memory

Impact: low functional, medium ergonomic. Worthwhile only if/when we add
a second top-level frontend (mobile?) or a docs subproject.

### B2 — Feature-grouped frontend layout

Inside `frontend/src/lib/`, group by feature instead of by file type:

```
lib/
  features/
    tracks/        components/  api/  schemas/  stores/
    waypoints/     ...
    bundles/       ...
    map/           ...
  components/ui/   # shared primitives stay here
  utils.ts
```

Pros: each feature is self-contained, easier to extract or rewrite.
Cons: only pays off above ~30 components; we are at 9. Premature today.

Suggested OpenSpec breakdown for Axis B: one change per item, B1 first
(it's a precondition for B2 looking natural).

---

## Smaller follow-ups (from UI-kit brainstorm)

Pulled out of the parent design's "Out of scope" section so they aren't
lost.

### F1 — Settings, Bundle Manager, History as full-screen routes

Now-trivial after Change 1 of the parent design. Each becomes a
`routes/<name>/+page.svelte`, lifted out of the Sidebar where they
currently squat. Likely individual OpenSpec changes — one per surface —
because each has its own behaviour to spec out (`ui-shell` capability
gains routes; the underlying capabilities — `project-persistence`,
`map-bundles` — gain "list view" requirements).

### F2 — Bundle id in URL (`/project/<bundle-id>`)

Enables deep-linking and external launch with a specific project. Touches
`ProjectCommand`, persistence layer, and `currentBundle` store. Sizeable
architectural change; not unlocked by UI-kit work alone.

### F3 — Custom themes beyond Catppuccin

Today the four Catppuccin flavours cover the request space. A user-defined
theme (or a SAR-branded high-contrast theme) would require the semantic
map to accept arbitrary input and a token-validator UI. Out of scope
indefinitely unless real demand appears.

### F4 — Motion library beyond Svelte transitions

`motion-svelte` or similar. Only if A5 starts hitting cases that Svelte
transitions + `tailwindcss-animate` cannot express (orchestrated
multi-element sequences, gesture-driven animations).

### F5 — Rust backend feature-grouped modules

Mirror of B2 on the backend: instead of strict layer split
(`domain/`/`application/`/`infrastructure/`/`commands/`), group by
feature (`tracks/`, `waypoints/`, `bundles/`). The current four-layer
split is intentional per ADR — flagging this is more for completeness
than as a recommendation.

---

## How this document is used

- Whenever the parent UI-kit work ships and the team asks "what's next?",
  this is the reference list.
- Each axis or follow-up that gets picked up enters the normal flow:
  brainstorm → design spec (`docs/superpowers/specs/`) → OpenSpec
  change(s) → implementation.
- Items can be removed if they become irrelevant — no need to keep dead
  entries. New items get appended.
