## Context

This change is the third and final step of the UI-kit axis defined in [`docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md`](../../../docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md). It corresponds to **"Change 3 — `migrate-panels-to-shadcn`"** in the parent design's "OpenSpec change breakdown" section and applies the "Migration pattern for existing components" subsection of Architecture across every existing panel in `src/components/`.

The change has hard dependencies on both predecessors:

- **Change 1 — `migrate-to-sveltekit`** is required because primitives and `cn()` are imported through `$lib/...`, and the `routes/project/+page.svelte` host that mounts `Sidebar` + `MapView` is created there.
- **Change 2 — `add-design-tokens-and-shadcn`** is required because the Tailwind utility classes used throughout (`bg-background`, `text-foreground`, `border-border`, …) resolve to nothing without Tailwind installed and the Catppuccin→semantic-token mapping applied; and the primitive components (`Card`, `Dialog`, `Popover`, `Select`, `Tabs`, `Tooltip`, `Switch`, `Separator`, `ScrollArea`, `Slider`, `Label`, `Table`, `Button`) are not on disk until Change 2 runs `shadcn-svelte init` and adds them.

If either dependency is rolled back, this change must be rolled back with it.

What stays explicitly outside scope (per parent design):

- The MapLibre internals of `MapView.svelte` — source/layer setup, drag handlers, click handlers, the tile protocol code — are not touched. Only the outer wrapper container is.
- The OziExplorer-compatible RGBA colour system on `TrackStyle.color` (parent design **D6**). Track and waypoint colour swatches remain native `<input type="color">` and operate on domain RGBA only.
- Future routes (Settings, Bundle Manager, History), bundle-id-in-URL, and the feature-grouped `src/lib/components/` (`sidebar/`, `pickers/`) restructure — all in the future-work backlog.
- New components. This change migrates exactly the nine that exist today.

## Migration order and rationale

| Step | Component             | Primitives needed                                             |
|------|-----------------------|---------------------------------------------------------------|
| 3.1  | `Console.svelte`      | Card, ScrollArea, Button                                      |
| 3.2  | `ThemePicker.svelte`  | Select                                                        |
| 3.3  | `SymbolPicker.svelte` | Popover, Button, Tooltip                                      |
| 3.4  | `SimplifyPanel.svelte`| Slider, Switch, Button, Label                                 |
| 3.5  | `TracksPanel.svelte`  | Button, Separator, Tooltip; native `<input type="color">` RETAINED |
| 3.6  | `WaypointsPanel.svelte` | + Dialog (on top of TracksPanel's set)                      |
| 3.7  | `TrackPointsPanel.svelte` | Table, ScrollArea, Tooltip                                |
| 3.8  | `Sidebar.svelte`      | Tabs, ScrollArea (shell — migrated last)                      |
| 3.9  | `MapView.svelte`      | wrapper-only — MapLibre init/handlers untouched               |

Rationale for the order:

1. **Leaves first, container last.** The leaf panels (Console, ThemePicker, SymbolPicker, SimplifyPanel, TracksPanel, WaypointsPanel, TrackPointsPanel) are migrated before `Sidebar` because each is independently observable inside the current Sidebar shell. Integration regressions are surfaced incrementally rather than batched at the end (mitigation for the parent design's "Sidebar regressions revealed only at the end" risk).
2. **`Sidebar` after all its children.** The Sidebar wraps every panel in this list. Migrating it last means the Tabs/ScrollArea swap happens with primitive-rendered children already in place, so the only delta on that commit is the shell.
3. **`MapView` last.** The MapLibre instance is the most fragile surface to regress; wrapper-only migration goes at the end so that no chrome change can be mistaken for a behavioural regression of the map.

## Per-component primitive checklist

| Component               | shadcn primitives                                        | Native control retained                | Notes |
|-------------------------|----------------------------------------------------------|----------------------------------------|-------|
| `Console.svelte`        | Card (shell), ScrollArea (log body), Button (close)      | none                                   | Backtick toggle and FPS overlay logic stay untouched. |
| `ThemePicker.svelte`    | Select.Root (root/trigger/content/item)                  | none                                   | Worked example from parent design. |
| `SymbolPicker.svelte`   | Popover, Button (trigger), Tooltip                       | none                                   | Emoji set is data, not chrome — kept. |
| `SimplifyPanel.svelte`  | Slider, Switch, Button, Label                            | none                                   | Debounced preview logic untouched. |
| `TracksPanel.svelte`    | Button, Separator, Tooltip                               | `<input type="color">`                 | Per-track colour swatch stays native (D6). |
| `WaypointsPanel.svelte` | Button, Separator, Tooltip, Dialog                       | `<input type="color">` (if present)    | Dialog for the rename/symbol/colour confirm flow. |
| `TrackPointsPanel.svelte` | Table, ScrollArea, Tooltip                             | none                                   | Per-row selection state stays in `$state`. |
| `Sidebar.svelte`        | Tabs, ScrollArea                                         | none                                   | Migrated after every child. |
| `MapView.svelte`        | none for the map itself; outer container uses Tailwind   | none                                   | MapLibre init/sources/layers/handlers not touched. |

## Style handling rules

Three buckets, in priority order:

1. **Tailwind utility classes from semantic tokens.** Every static, theme-driven style moves here. Examples: `bg-background`, `text-foreground`, `bg-popover text-popover-foreground`, `border-border`, `bg-card`, `bg-muted`, `rounded-md`, `p-3`, `gap-2`, `text-sm`. Hover/focus/disabled variants use the corresponding semantic tokens (`hover:bg-accent`, `focus-visible:ring-ring`, `disabled:opacity-50`).
2. **Inline `style=` attribute, computed from domain values.** Anything whose value comes from the Rust domain — a `TrackStyle.color` swatch, a `WaypointSymbol`-driven background, a per-row width number — stays on inline `style=`. These bindings are explicitly RGBA / domain values and are not theme-driven. Example: `<span style="background: {track.color}">…</span>`.
3. **Component-local `<style>` blocks are deleted.** No panel keeps a `<style>` block after migration unless it expresses something that fits neither category 1 nor category 2 (no such case is expected; if one arises during implementation, document it inline in the commit message and in this design).

The MapLibre canvas and its own injected styles (`maplibre-gl/dist/maplibre-gl.css`) are untouched. Theme-aware overrides of MapLibre internals are out of scope.

## Risks

| Risk                                                                                          | Likelihood | Mitigation |
|-----------------------------------------------------------------------------------------------|------------|------------|
| `Sidebar` (step 3.8) reveals integration regressions only at the end                          | Medium     | Each migrated panel is rendered in the current `Sidebar` immediately, so integration is checked incrementally; the Sidebar-shell commit is then a small delta. |
| Track / waypoint colour accidentally tied to a Catppuccin token                               | Low        | Explicit rule in proposal + this design + spec scenario: `TracksPanel`/`WaypointsPanel` colour inputs operate on `TrackStyle.color` (RGBA) only, never on `--ctp-*` or semantic tokens. Verified by a "track colour swatch unchanged across all four flavours" screenshot pass. |
| shadcn Popover / Dialog conflicts with MapLibre DOM (z-index, pointer events)                 | Low        | Primitives portal to `body` by default. Smoke test: open `SymbolPicker` popover and `WaypointsPanel` dialog while the map is mounted, on each of the four flavours. |
| Per-component `<style>` deletion drops something that was load-bearing                        | Low        | Audit-per-component approach: read each `<style>` block before deletion, map each rule into either a Tailwind utility (category 1) or an inline `style=` (category 2) explicitly. |
| `lucide-svelte` icon swap changes hit-areas / alignment subtly                                | Low        | Icon swap is per-audit, not blanket; screenshot pass before/after each step catches alignment regressions. |
