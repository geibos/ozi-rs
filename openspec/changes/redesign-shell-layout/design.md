## Context

ozi-rs ships a SAR-oriented OziExplorer alternative on a modern stack (Tauri 2 + Svelte 5 + Tailwind v4 + shadcn-svelte). The current workspace shell at `src/routes/project/+page.svelte` is a thin wrapper that mounts `src/components/Sidebar.svelte` (a ~224px nervous-system column with five hand-stacked sections: Project / Map / Tracks / Waypoints / Footer) alongside three floating panel components (`TracksPanel`, `TrackPointsPanel`, `WaypointsPanel`) that float above the `MapView`. The default theme is Catppuccin in four flavours + Auto-follows-OS, wired through two coexisting CSS-variable layers (`--ctp-*` palette + semantic-token HSL triplets) on the root element. Surfaces use shadcn-svelte primitives styled against those semantic tokens.

That structure ports the OziExplorer mental model rather than reimagining it. The session brainstorming locked a new direction:

- Inspiration: Gaia GPS sidebar, CalTopo layer power, macOS Maps chrome restraint, Ozi keyboard density.
- Drops: the 65-button Win95 toolbar, MDI floating dialogs, beveled gray system chrome, the 9-menu menubar, Catppuccin-as-default.
- Layout: 3-pane shell (left rail ~280px library / center MapView / right rail ~360px on-demand inspector), thin top context-bar with mode chips + a Cmd-K trigger, bottom status bar. No menubar.

The full direction lands in three sequential changes:

1. **redesign-shell-layout** (this change) — workspace shell, design tokens, typography, primitives.
2. **library-sidebar** — content for the left rail (project picker, layer tree, tracks, waypoints, "Maps…" affordance against the new Sheet primitive).
3. **inspector-pane** — content for the right rail (selection-driven detail panels) and the Cmd-K command palette behavior.

This change is documentation-only for the shell foundation and the design-token layer. It does not redesign the cold-start `/` route structurally; that route stays a 2-column projects/maps grid for v1 and is restyled only through the new tokens.

There is a parallel proposed change, `consolidate-state-event-flow`, that rewrites the Tauri event-listener topology in `src/routes/+layout.svelte` and the bundle-loader page. The two changes touch overlapping files. To avoid merge conflicts and behavior regressions, `redesign-shell-layout` MUST land AFTER `consolidate-state-event-flow` — the listener consolidation is mechanical and unblocks the shell rewrite from worrying about event topology.

This change also supersedes the active `bundle-loader-as-overlay` proposal. The Sheet primitive that proposal adds, and the "Maps…" slide-in behavior it describes, are absorbed here (the primitive) and into `library-sidebar` (the affordance wiring).

## Goals / Non-Goals

**Goals:**
- Replace the workspace layout primitive with a 3-pane shell that has named slots ready for content from later changes.
- Replace the default theme with native-feeling Zinc + Emerald tokens; preserve Catppuccin as opt-in.
- Switch typography to Geist + Geist Mono; ban Inter.
- Add the shadcn-svelte primitives (`sheet`, `command`) that the follow-up changes consume.
- Preserve the MapView "mounted once across route changes" invariant and the status-bar stable-layout invariant from previous changes.

**Non-Goals:**
- Filling the rail slots with content. Library content belongs to `library-sidebar`; inspector content belongs to `inspector-pane`.
- Wiring the Cmd-K palette behavior. The trigger button is visual scaffolding only; the actual command palette behavior belongs to `inspector-pane`.
- Wiring the "Maps…" Sheet affordance. The primitive is added here; the call site is `library-sidebar`.
- Redesigning the cold-start `/` route structurally. Restyling only through the new tokens.
- Removing `Sidebar.svelte`. It stays in the tree (hidden behind the empty library rail slot) until `library-sidebar` replaces it.
- Backend changes. No new IPC, no tile-renderer work, no `.ozf2`/SQLite changes.
- Resizable rails. Fixed widths for v1.
- Removing the existing Catppuccin code. It is moved, not deleted, so the theme selector continues to function.
- Adding new motion libraries beyond what Svelte 5 already ships. `svelte/motion` + `svelte/transition` cover spring physics; `motion-primitives-svelte` is on the table only if a real choreography need shows up in change 2 or 3.

## Decisions

### Decision 1: Default theme switches from Catppuccin to native auto/light/dark

The application's default theme SHALL be a native-feeling Zinc + Emerald system that auto-tracks OS light/dark. Catppuccin SHALL move to an opt-in theme pack at `src/lib/themes/catppuccin.css`, selectable from settings.

**Rationale**: Catppuccin is a strong, opinionated aesthetic that biases the app toward "developer-friendly tinted dark UI" — fine for an editor, off-key for a SAR tool used on a sunlit laptop screen by users who expect their app to look like Maps or Gaia GPS, not like a code editor. Zinc (slate-tinted-cool) reads as native on both macOS and Windows; Emerald as a single accent stays calm at high luminance and survives projection in field conditions. Catppuccin remains valuable for users who explicitly want it — keeping it as an opt-in pack costs nothing structurally.

**Open question to confirm with parent**: should the Catppuccin pack ship enabled-by-default-but-overridden, or fully gated behind a settings toggle? Current proposal assumes fully gated.

**Alternatives considered**:
- _Keep Catppuccin as default, add Zinc as opt-in_: rejected — Catppuccin's saturation makes the new minimal surfaces (cards, status bar, mode chips) read as decorative rather than functional. The new design language was authored around Zinc tones.
- _Drop Catppuccin entirely_: rejected — the four flavours are useful, the code is already written, the semantic-token mapping tables are tested. Moving the layer behind a settings toggle is cheaper than deleting and re-adding it later.

### Decision 2: Geist + Geist Mono, Inter explicitly banned

The UI font SHALL be Geist (variable), and Geist Mono SHALL be used for numbers (with `font-mono tabular-nums`). Both SHALL be loaded via the `geist` npm package and consumed through `src/app.css`. Inter SHALL be explicitly listed as banned in the typography spec — components SHALL NOT add it back via a font stack override or a font import.

**Rationale**: Geist is the cleanest neutral sans for native-feeling Tauri apps in 2026 — it shares enough metrics with SF Pro and Segoe UI Variable that the chrome reads as native on both platforms, while staying brand-consistent across OS boundaries. Inter is the obvious default but is overused in 2025-era SaaS UI; using it here makes the app look templated rather than native.

**Alternatives considered**:
- _SF Pro / Segoe UI Variable native stacks_: rejected — readable but inconsistent across the two platforms (numerics differ, weights differ), and Tauri's font fallback is not reliable enough to guarantee identical layout.
- _Inter as the workhorse_: rejected per ban above.
- _Custom font (e.g. Sohne, Söhne)_: rejected — licensing complexity.

### Decision 3: Single accent (Emerald) with strict saturation ceiling

The accent palette SHALL be a single hue (Emerald, `oklch(0.7 0.13 152)` or equivalent), with saturation capped at 70%. Multi-accent UIs (purple secondary, blue tertiary, etc.) are out — the app uses Emerald for primary actions and active states, and the neutral Zinc scale for everything else.

**Rationale**: SAR is high-stakes. A multi-accent palette communicates "casual SaaS dashboard"; a single-accent palette communicates "tool". Emerald specifically: stays readable on both dark and light surfaces, doesn't conflict with map vector colours (which trend toward red/orange/yellow for warnings and tracks), and survives projection.

**Anti-pattern guard**: the design-token layer SHALL NOT declare a "secondary accent" CSS variable. Any future need for a contrasting colour SHALL be served by a Zinc shade, not a competing hue.

### Decision 4: 3-pane shell with named slots, fixed widths, on-demand right rail

The workspace shell SHALL be a fixed 3-pane CSS grid (or flex, see implementation note below) with named slots `library-rail` (left, 280px), `canvas` (center, MapView, flexible), `inspector-rail` (right, 360px on-demand). The right rail SHALL be present in the DOM only when its content store is non-empty — its absence SHALL NOT shift the canvas; the canvas grows into the freed space.

**Why named slots, not just children**: the two follow-up changes mount their content via `<svelte:fragment slot="library-rail">` (or named children with explicit prop wiring — implementation detail TBD by `library-sidebar`). Named slots make the contract between the shell and its fillers explicit and testable.

**Why fixed widths for v1**: SAR users typically run a single laptop screen, so the layout is sized for 1280-1440px viewports. A resize affordance adds complexity (drag handle, persistence, content reflow at narrow widths) for a feature that has not been requested. Defer.

**Why on-demand right rail**: an empty inspector rail wastes 360px of horizontal space. Hiding it when there is no selection is the standard pattern in Gaia GPS, Procreate, and macOS Maps.

**Implementation note (non-binding)**: CSS grid with `grid-template-columns: 280px 1fr 360px` and a conditional `display: none` on the inspector rail when empty. Alternative: flex with explicit widths. Either is acceptable; choose during implementation.

### Decision 5: Sheet primitive added here, wired in change 2

The shadcn-svelte `sheet` primitive SHALL be generated by this change (`npx shadcn-svelte@latest add sheet`) and SHALL live unmounted at `src/lib/components/ui/sheet/`. The actual "Maps…" call site that opens the Sheet over the workspace lives in `library-sidebar` — this change does not introduce a Sheet mount point in the shell.

**Rationale**: the primitive is a shell-layer concern (it is the second-most fundamental UI primitive after a button); its wiring is a content-layer concern. Splitting the work across the two changes keeps each change small and reviewable. The `bundle-loader-as-overlay` proposal collapsed both into one change; absorbing the primitive here and the wiring into `library-sidebar` is a cleaner split.

**Consequence**: the spec delta in this change adds a `sheet` primitive availability requirement but does NOT add a "Maps… opens a Sheet" scenario — that scenario belongs to `library-sidebar`'s spec delta.

### Decision 6: Status bar and mode chips are visual scaffolding, content deferred

The top context-bar SHALL render four mode-chip placeholders labelled View / Draw / Edit / Measure and a single Cmd-K trigger button. The bottom status bar SHALL render an empty stable-height region. Neither has functional content in this change. The mode chips are styled but not wired to any state; the Cmd-K button does not open the palette yet (the palette behavior lands in `inspector-pane`).

**Rationale**: the visual scaffolding is part of the layout, but the content driving it depends on data structures (selected mode, active status messages, command registry) that the follow-up changes design. Building the scaffolding now lets us screenshot the shell in its full visual form without committing to the content model.

**Stable-layout requirement**: the status bar's height SHALL be a single CSS value reused both as the bar's own height and as the bottom inset of the workspace grid, mirroring the bundle-loader stable-layout contract from the `bundle-loader-non-blocking` and `stream-bundle-file-availability` requirements. Status content (string, spinner, progress chip) can change; the bar's outer footprint does not.

### Decision 7: Cmd-K wiring is deferred to inspector-pane

The Cmd-K trigger button in the top context-bar SHALL render in this change but SHALL NOT bind to any keyboard shortcut and SHALL NOT open the `command` primitive in this change. The actual palette behavior (registry of commands, keyboard shortcut binding, palette open/close, fuzzy search) lands in `inspector-pane`.

**Rationale**: a command palette is a content concern, not a chrome concern — its content is the registry of inspector and library actions that the follow-up changes own. Wiring it now would force this change to specify a registry shape that the follow-ups would then have to retrofit.

**This change DOES**: generate the `command` primitive at `src/lib/components/ui/command/` so `inspector-pane` can mount it without re-running the shadcn generator.

### Decision 8: Tokens live in a dedicated tokens.css, not inlined in app.css

The design-token layer (Zinc, Emerald, radii, shadows, inner-border) SHALL be declared in `src/lib/tokens.css` via Tailwind v4 `@theme`, and imported from `src/app.css`. Catppuccin's existing variables move to `src/lib/themes/catppuccin.css` as an opt-in layer imported only when the user selects a Catppuccin flavour from settings.

**Rationale**: splitting tokens from utilities keeps the design-system surface auditable. Future theme packs (high-contrast, color-blind safe, etc.) follow the same `src/lib/themes/<name>.css` pattern.

## Risks / Trade-offs

- **Risk**: switching the default theme is visually disruptive — existing screenshots in docs, the QA evidence directory, and any user expectations from prior screenshots are now stale. **Mitigation**: docs/qa/* screenshots are regenerated on each QA pass anyway; the theme switch is a one-time delta, not an ongoing concern. The Catppuccin theme pack remains available so users who prefer the old look can opt in.
- **Risk**: merge conflict with `consolidate-state-event-flow`. **Mitigation**: stated landing order (consolidate first, redesign-shell-layout after). Rebase before implementation.
- **Risk**: superseding `bundle-loader-as-overlay` while it is still in the active changes list creates ambiguity. **Mitigation**: this change's proposal.md explicitly notes the supersession; the parent should archive (without applying) `bundle-loader-as-overlay` once this change lands, or fold its remaining task content into the `library-sidebar` proposal.
- **Risk**: the empty rail slots in this change look "broken" in a screenshot (lots of empty space). **Mitigation**: smoke evidence in the tasks list explicitly notes "empty rail slots are expected"; the next two changes fill them.
- **Risk**: Geist's variable axes or weight subset could conflict with shadcn-svelte primitives that assume a fixed font stack. **Mitigation**: shadcn-svelte primitives consume `font-sans` from the Tailwind theme; setting `font-sans` to Geist in the token layer is enough.
- **Trade-off**: the status bar and mode chips render as inert placeholders for an entire change cycle. Acceptable: they communicate the final layout intent without committing to content shape too early. Worst case: a follow-up screenshot review surfaces a layout problem and we adjust the scaffolding in change 2 or 3.
- **Trade-off**: keeping `Sidebar.svelte` in the tree while hiding it behind an empty slot duplicates the "where does the library live" decision across two changes. Acceptable: deleting `Sidebar.svelte` here would force this change to also reimplement project-pick, layer-tree, tracks-list, waypoints-list — too much work for a shell change. The dead-code window is one change long.

## Open Questions (parent confirmation requested)

- **Q1**: Should the Catppuccin pack be fully gated behind a settings toggle (clean default, advanced opt-in), or shipped as a "Theme: Native (default) / Catppuccin (legacy)" top-level selector visible at first launch? Current proposal assumes fully gated.
- **Q2**: Emerald specifically vs a slightly desaturated Teal — both meet the "single accent ≤ 70% saturation" criterion. Current proposal locks Emerald per session direction. Parent may want to confirm Emerald reads correctly against the SAR map-tile colour palette (typically OSM-derived oranges and reds) before lock-in.
- **Q3**: Is fixed-width-rail acceptable for v1, or does the parent want a resize affordance in scope? Current proposal defers resize.
- **Q4**: Should `bundle-loader-as-overlay` be archived as "obsolete by supersession" or remain in the active list as a reference for what `library-sidebar` will absorb? Current proposal leaves the housekeeping to the parent.

## Migration Plan

Implementation order, to keep the working tree shippable at each step:

1. Wait for `consolidate-state-event-flow` to land. Do not start this change until the listener consolidation is merged. Rebase.
2. Add the `geist` npm package; commit `package.json` + lockfile only, no code changes yet.
3. Create `src/lib/tokens.css` with the Zinc + Emerald + radii + shadow tokens. Update `src/app.css` to `@import` it and to wire Geist as `font-sans`.
4. Move the Catppuccin variables out of their current home into `src/lib/themes/catppuccin.css` as an `@layer theme` block. Update the theme selector to import that file only when a Catppuccin flavour is active.
5. Run `npx shadcn-svelte@latest add sheet` and `npx shadcn-svelte@latest add command`. Commit the generated files.
6. Create `src/components/WorkspaceShell.svelte` with the 3-pane grid, named slots for `library-rail` and `inspector-rail`, the canvas slot for MapView, the top context-bar scaffolding, and the bottom status-bar scaffolding.
7. Rewrite `src/routes/project/+page.svelte` to mount `WorkspaceShell` with empty rail slots. Leave `Sidebar.svelte` in the repo tree but unmounted from the workspace route — `library-sidebar` will either reuse it inside the rail slot or delete it.
8. Restyle `src/routes/+page.svelte` (cold-start) surfaces to consume the new tokens. No structural change.
9. Smoke verification per the tasks list. Capture screenshots of the empty shell (rails empty), the cold-start route under the new tokens, and the Catppuccin opt-in theme to confirm it still applies.
10. `just ci` + `openspec validate redesign-shell-layout --strict`.

Two PRs are acceptable if the diff is large: PR 1 = tokens + Geist + Catppuccin move + primitive generation; PR 2 = WorkspaceShell + route rewrite + smoke. A single PR is also fine if reviewer bandwidth allows.
