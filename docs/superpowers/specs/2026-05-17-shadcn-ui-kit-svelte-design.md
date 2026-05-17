# Shadcn UI-kit on Svelte — Design Spec

- Date: 2026-05-17
- Status: draft, pending OpenSpec breakdown into 3 changes
- Inspiration: [Zackriya-Solutions/meetily](https://github.com/Zackriya-Solutions/meetily) — same Tauri base, modern shadcn/ui-style frontend
- Scope of this document: the visual-feel (UI-kit) axis only. Two parallel
  directions from the original brainstorm — perceived speed (UX polish) and
  repo layout (`backend/`/`frontend/` rename) — are out of scope here and will
  get their own brainstorms when prioritised.

## Purpose

We want `ozi-rs` to feel as polished as meetily — same design-language
("shadcn-style"), consistent primitives, modern tooling — without giving up
Svelte. This document captures the decisions taken during brainstorming and
the breakdown into OpenSpec changes that follow.

The motivating user request was loose: "make it look/work/be organised like
meetily." During the brainstorm we narrowed it to **visual feel (UI-kit)**
as the first axis to address; we then established that Svelte 5 + shadcn-svelte
+ bits-ui + Tailwind delivers the same outcome as meetily's Next.js + shadcn/ui
+ Radix stack, without a framework migration.

## Decisions

### D1 — Stay on Svelte; adopt shadcn-svelte

We do **not** migrate to React. Svelte 5 + shadcn-svelte (the direct
community port of shadcn/ui) + bits-ui (Radix-equivalent headless primitives)
delivers the same visual outcome. Bundle is smaller, runes-based reactivity
is more predictable, and our existing Svelte components keep working.

### D2 — Migrate to SvelteKit + adapter-static

The pure-Vite + Svelte 5 setup we have today is non-standard. The industry
norm for Svelte + Tauri in 2026 is **SvelteKit with `@sveltejs/adapter-static`**:
file-based routing, `$lib` alias, preconfigured tooling, every Svelte
developer recognises the structure. Tauri loads the prebuilt static output,
so SSR is disabled (`ssr = false`) and prerender is on for static HTML
entries.

### D3 — Dedicated routes for the two top-level surfaces

Instead of conditional rendering inside `App.svelte`, we use real routes:

| Path | View | Entry condition |
|---|---|---|
| `/` | `BundleLoaderView` | No bundle loaded; or user navigated back |
| `/project` | `MapView` + `Sidebar` + panels | Bundle loaded |

Redirects between them happen client-side via `onMount` + `goto`, since
`prerender=true` means route-level `load` functions have no runtime store
access. `BundleLoaderView` moves into `routes/+page.svelte`; the main
working view into `routes/project/+page.svelte`.

This unlocks future `routes/settings/+page.svelte`,
`routes/bundles/+page.svelte`, `routes/history/+page.svelte` without any
additional routing library.

### D4 — Catppuccin remains the source of truth for colours

We keep our four flavours (Latte / Frappé / Macchiato / Mocha) plus Auto.
`@catppuccin/palette` is the canonical palette source. The current
`theme.ts` mechanism — writing `--ctp-*` CSS variables to `<html>` — is
extended, not replaced.

### D5 — Two coexisting CSS variable layers

shadcn-svelte components expect semantic tokens (`--background`,
`--foreground`, `--primary`, `--border`, …) in HSL-triplet form. We map
Catppuccin colours into these tokens at the same time as the existing
`--ctp-*` hex variables, on the same `<html>` element.

| Layer | Variables | Consumer |
|---|---|---|
| Palette | `--ctp-base`, `--ctp-red`, `--ctp-mauve`, … (hex) | Existing component styles, MapLibre placeholder background |
| Semantic | `--background`, `--foreground`, `--primary`, … (HSL triplet) | shadcn-svelte primitives, Tailwind utility classes |

A `class="dark"` toggle on `<html>` is set whenever the active flavour is
not Latte; this enables shadcn-svelte's dark-variant utilities. The
mapping table has Latte- and dark-specific variants for the few cases
where `surface0`/`base` semantics differ.

### D6 — Track and waypoint colours stay in their own coordinate system

Track and waypoint colours live in the domain (`TrackStyle.color: [u8; 4]`,
RGBA bytes) and are interoperable with OziExplorer through PLT
`COLORREF` import and Garmin named-colour GPX export. **The theme system
must not touch these.** `TracksPanel` keeps its native `<input type="color">`;
`tracks-layer.ts` keeps reading the colour off the feature property.

### D7 — Forms via felte + zod

Svelte equivalent of meetily's `react-hook-form + zod`. `superforms`
needs SvelteKit's form-action runtime which we don't use; `felte` is
framework-light and works with our existing IPC-driven mutations.

### D8 — Industry-standard tooling baseline

ESLint flat config + `eslint-plugin-svelte`; Prettier with
`prettier-plugin-svelte` and `prettier-plugin-tailwindcss`; `svelte-check`
in CI; `@testing-library/svelte` for primitive smoke tests.

## Architecture

### Target directory layout (post all 3 changes)

```
src/
  app.html
  app.css                  # @tailwind + base layer
  app.d.ts
  routes/
    +layout.svelte         # global chrome: theme bootstrap, sonner host
    +layout.ts             # ssr=false; prerender=true
    +page.svelte           # BundleLoaderView (welcome / load surface)
    +page.ts
    project/
      +page.svelte         # MapView + Sidebar + panels
      +page.ts
  lib/
    components/
      ui/                  # shadcn-svelte primitives — our code
        button/ dialog/ popover/ select/ tabs/ tooltip/
        switch/ separator/ scroll-area/ input/ label/
        sonner/ slider/ card/ table/
      Console.svelte
      MapView.svelte
      Sidebar.svelte
      SimplifyPanel.svelte
      SymbolPicker.svelte
      ThemePicker.svelte
      TrackPointsPanel.svelte
      TracksPanel.svelte
      WaypointsPanel.svelte
    forms/
      schemas/             # zod schemas per feature
      create-form.ts       # felte + zod resolver
    utils.ts               # cn() = twMerge(clsx(...))
    api.ts                 # unchanged
    stores.ts              # unchanged
    types.ts               # unchanged
    theme.ts               # extended with semantic mapping + class="dark"
    track-names.ts
    track-stats.ts
    windows.ts
    maplibre/              # unchanged
  test/
```

`src/components/` (current root-level folder) and `src/views/` go away;
everything frontend now lives under `src/lib/` and `src/routes/`.

### Catppuccin → semantic token mapping

Semantic tokens use HSL triplets (no `hsl()` wrapper) so Tailwind can
apply alpha. `@catppuccin/palette` exposes `.hsl = {h, s, l}` per colour,
which we render to `"${h} ${s}% ${l}%"`.

Light (Latte) and dark (Frappé / Macchiato / Mocha) need slightly
different mappings — for example, `popover` resolves to `base` in light
but `surface0` in dark to avoid inverted surfaces. We keep
`SEMANTIC_MAP_LIGHT` and `SEMANTIC_MAP_DARK` tables in `src/lib/theme.ts`.

Tailwind config consumes the tokens through
`hsl(var(--background) / <alpha-value>)` patterns:

```ts
extend: {
  colors: {
    background: 'hsl(var(--background) / <alpha-value>)',
    foreground: 'hsl(var(--foreground) / <alpha-value>)',
    card: { DEFAULT: 'hsl(var(--card))', foreground: 'hsl(var(--card-foreground))' },
    // ... popover, primary, secondary, muted, accent, destructive, border, input, ring
  },
}
```

### Migration pattern for existing components

Per-component pattern (worked example: `ThemePicker.svelte`):

1. Replace native control with shadcn primitive (`<select>` → `Select.Root`).
2. Delete the `<style>` block; replace local CSS with Tailwind utility classes that read semantic tokens (`bg-popover`, `text-foreground`, `border-border`).
3. Use `$state` runes for local state.
4. Keep dynamic values from the domain (e.g. track colour `{color}`) on the inline `style=` attribute — these are not theme-driven.

`MapView.svelte` is migrated last and only at the wrapper level; MapLibre
initialization, drag handlers, click handlers, and the tile-protocol code
are untouched.

## OpenSpec change breakdown

The work splits into three OpenSpec changes, each self-contained and each
mergeable independently. They all target the `ui-shell` capability and
no other capability is affected.

### Change 1 — `migrate-to-sveltekit`

Foundation move from pure-Vite to SvelteKit + adapter-static.

Deliverables:
- `svelte.config.js` with `adapter-static`, `src/app.html`, `src/app.d.ts`
- `routes/+layout.svelte`, `routes/+layout.ts`, `routes/+page.svelte` (loader), `routes/project/+page.svelte` (main)
- `tauri.conf.json` updated (`frontendDist: "build"`)
- ESLint flat config with `eslint-plugin-svelte`, Prettier with `prettier-plugin-svelte`, `svelte-check` in CI (the `prettier-plugin-tailwindcss` add-on lands in Change 2, when Tailwind itself is installed)
- `@testing-library/svelte` installed
- `justfile` updated with `lint`, `fmt`, `check` recipes
- `AGENTS.md`, `CLAUDE.md` updated for new structure

Spec-delta (`ui-shell`): MODIFIED Requirement — "Frontend is bootstrapped
via SvelteKit + adapter-static; the two top-level surfaces (bundle loader,
project workspace) live at distinct routes `/` and `/project`."

### Change 2 — `add-design-tokens-and-shadcn`

Tailwind, Catppuccin→semantic mapping, shadcn-svelte primitives library.

Deliverables:
- Tailwind v4 + autoprefixer + postcss + `tailwindcss-animate` + `@tailwindcss/typography`; add `prettier-plugin-tailwindcss` to the Prettier config from Change 1
- `tailwind.config.ts` with token-aware colour scale
- `src/lib/theme.ts` extended: `SEMANTIC_MAP_LIGHT/DARK`, `applySemanticTokens`, `class="dark"` toggle
- `src/lib/utils.ts` with `cn()`
- `shadcn-svelte init`; primitives installed: button, dialog, popover, select, tabs, tooltip, switch, separator, scroll-area, input, label, sonner, slider, card, table
- `lucide-svelte`, `svelte-sonner`, `tailwind-variants`
- `felte` + `zod`, `src/lib/forms/create-form.ts`
- `THIRD_PARTY_LICENSES.md`, `tauri.conf.json` `licenseFile` pointer, README Credits section
- Vitest tests for theme mapping per flavour
- `@testing-library/svelte` smoke tests for each primitive

Spec-delta (`ui-shell`): MODIFIED Requirement — "UI surfaces use the
shadcn-svelte primitive library; theme tokens are derived from the active
Catppuccin flavour and applied as HSL-triplet semantic CSS variables
alongside the existing palette variables."

### Change 3 — `migrate-panels-to-shadcn`

Apply primitives across existing components.

Deliverables (9 component migration steps, one commit each):
- 3.1 Console
- 3.2 ThemePicker
- 3.3 SymbolPicker
- 3.4 SimplifyPanel
- 3.5 TracksPanel (native `<input type="color">` retained)
- 3.6 WaypointsPanel
- 3.7 TrackPointsPanel
- 3.8 Sidebar (shell)
- 3.9 MapView (wrapper only)

Plus:
- Replace text/emoji icons with `lucide-svelte` where appropriate
- Route any toast notifications through `svelte-sonner`
- `docs/frontend-architecture.md`, `docs/project-map.md` updated
- Full 4-theme screenshot pass via `ozi-rs-mcp`

Spec-delta (`ui-shell`): MODIFIED Requirement — "All in-app panels render
through shadcn-svelte primitives and Tailwind utility classes that consume
semantic tokens; component-local `<style>` blocks are removed except for
dynamic per-feature values from the domain."

## Acceptance criteria

Verified at the close of each change:

- `just lint && just check && just clippy && just test` all green
- `tauri build` produces a working application
- Manual smoke through the affected surfaces (create track, import PLT, export GPX, toggle theme)
- For Change 2 and 3: visual screenshot pass across all four Catppuccin flavours via `ozi-rs-mcp`

## Out of scope (future changes)

The full backlog of everything left out — including the two remaining
axes from the original meetily-inspired brainstorm (perceived speed / UX
feel, and repository layout) and the smaller follow-ups — lives in
[2026-05-17-meetily-inspired-future-work.md](./2026-05-17-meetily-inspired-future-work.md).

Highlights, for quick reference:

- **Axis A (UX feel):** command palette, keyboard-first navigation, optimistic updates, skeletons/progress, motion, empty/error states.
- **Axis B (repo layout):** rename `src-tauri/` → `backend/` and `src/` → `frontend/src/`; potential feature-grouped frontend.
- **Smaller follow-ups:** Settings/Bundle Manager/History as full-screen routes, bundle id in the URL, non-Catppuccin themes, an external motion library, backend feature-grouped modules.

## Risks and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| SvelteKit migration breaks `tauri build` | Medium | Run `tauri build` in Change 1 immediately after `adapter-static` install, before any other changes |
| shadcn popovers conflict with MapLibre DOM | Low | Popovers portal into `body` by default; covered by a smoke test "open Select over the map" |
| ESLint trips on legacy components mid-Change-1 | Medium | Temporarily ignore `src/components/*` during Change 1; rules tighten as Change 3 migrates each file |
| Tailwind bundle bloat | Low | Tailwind v4 content-purge is on by default; verified via `vite build --report` |
| Sidebar (Change 3.8) reveals integration regressions only at the end | Medium | Each migrated panel is rendered in the current `Sidebar` immediately, so integration is checked incrementally |
| `tauri.conf.json` `frontendDist` swap breaks CI | Low | Caught immediately by the `tauri build` step in Change 1 |
| Track colours accidentally bound to theme | Low | Explicit rule in Change 3 design: `TracksPanel`/`WaypointsPanel` colour inputs operate on domain RGBA only, never on `--ctp-*` variables |

## Open questions

None at design time. If new questions arise during implementation, they
get raised in the corresponding OpenSpec change's `design.md` rather than
edited back into this document.
