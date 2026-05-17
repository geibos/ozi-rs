## 1. Validate deltas

- [x] 1.1 Run `openspec validate migrate-panels-to-shadcn --strict` and confirm the `ui-shell` MODIFIED requirement parses with at least one scenario.
- [x] 1.2 Confirm the full-text copy of every MODIFIED requirement matches the current `openspec/specs/ui-shell/spec.md` body before any new requirements / scenarios are added.
- [x] 1.3 Confirm both predecessor changes (`migrate-to-sveltekit` and `add-design-tokens-and-shadcn`) are merged before starting Section 2.

## 2. Console.svelte (step 3.1)

- [x] 2.1 Replace the outer `<div class="console">` with `Card` (header + content), the log body with `ScrollArea`, and the close `<button>` with `Button` (icon variant).
- [x] 2.2 Delete the component-local `<style>` block; replace with Tailwind utility classes that read semantic tokens (`bg-card`, `text-card-foreground`, `border-border`).
- [x] 2.3 Keep the backtick / Escape `onkeydown` handler and the `$effect`-driven autoscroll untouched.
- [ ] 2.4 Verify the Console looks correct in all four Catppuccin flavours via `ozi-rs-mcp` screenshots (Latte, Frappé, Macchiato, Mocha).
- [ ] 2.5 Smoke-test: press backtick to open, press backtick again to close; trigger a diagnostic and confirm autoscroll lands at the bottom of the new line.

## 3. ThemePicker.svelte (step 3.2)

- [x] 3.1 Replace the native `<select>` with `Select.Root` + `Select.Trigger` + `Select.Content` + `Select.Item` from `$lib/components/ui/select`.
- [x] 3.2 Delete the component-local `<style>` block; trigger inherits styling from the primitive.
- [x] 3.3 Keep the `applyTheme($selectedTheme)` `$effect` and the `prefers-color-scheme` listener for Auto mode untouched.
- [ ] 3.4 Verify the ThemePicker dropdown looks correct in all four Catppuccin flavours via `ozi-rs-mcp` screenshots.
- [ ] 3.5 Smoke-test: switch flavour Latte → Frappé → Macchiato → Mocha → Auto; confirm `--ctp-*` AND semantic tokens update on `<html>` for each step.

## 4. SymbolPicker.svelte (step 3.3)

- [ ] 4.1 Replace the open/close-state-driven `<div>` overlay with `Popover.Root` + `Popover.Trigger` + `Popover.Content`; the trigger uses `Button` (icon-only ghost variant).
- [ ] 4.2 Wrap each symbol option in `Tooltip` so its label appears on hover.
- [ ] 4.3 Delete the component-local `<style>` block; grid layout moves to Tailwind utilities (`grid grid-cols-5 gap-2 p-2`).
- [ ] 4.4 Keep the `SYMBOLS` data array and `getEmoji()` helper untouched (this is data, not chrome).
- [ ] 4.5 Verify the popover renders correctly in all four flavours via `ozi-rs-mcp` screenshots; confirm it portals above the MapLibre canvas.
- [ ] 4.6 Smoke-test: open the picker over the map, pick a symbol, confirm the trigger updates and the popover closes.

## 5. SimplifyPanel.svelte (step 3.4)

- [ ] 5.1 Replace the tolerance numeric input/range with `Slider`; replace the "preview" toggle with `Switch`; pair each with `Label`.
- [ ] 5.2 Replace the confirm / cancel `<button>`s with `Button` (`default` and `outline` variants).
- [ ] 5.3 Delete the component-local `<style>` block; layout moves to Tailwind utilities.
- [ ] 5.4 Keep the debounced `getSimplifiedPreview` `$effect` and the `simplifyTrack` call site untouched.
- [ ] 5.5 Verify the panel looks correct in all four flavours via `ozi-rs-mcp` screenshots.
- [ ] 5.6 Smoke-test: open Simplify on a track, drag the slider, confirm preview updates after debounce, confirm; confirm result lands on the track.

## 6. TracksPanel.svelte (step 3.5)

- [ ] 6.1 Replace the action buttons (toggle visibility, export GPX, export PLT, rename) with `Button` (appropriate variants); use `lucide-svelte` icons where they improve clarity over emoji/text.
- [ ] 6.2 Replace ad-hoc dividers between rows with `Separator`; wrap row actions in `Tooltip`.
- [ ] 6.3 **Retain the native `<input type="color">` for the per-track colour swatch.** Confirm it binds to `TrackStyle.color` (RGBA bytes via `setTrackColor`) and not to any `--ctp-*` or semantic token.
- [ ] 6.4 Keep the dynamic colour preview using inline `style="background: {track.color}"` — NOT a Tailwind class.
- [ ] 6.5 Delete the component-local `<style>` block; row layout, hover, selected state all move to Tailwind utility classes with semantic tokens.
- [ ] 6.6 Verify the panel looks correct in all four flavours via `ozi-rs-mcp` screenshots; specifically verify each track's colour swatch is identical across all four flavours (mitigation for the "colour tied to theme" risk).
- [ ] 6.7 Smoke-test: rename a track, change its colour, toggle visibility, export GPX, export PLT — all from the migrated panel.

## 7. WaypointsPanel.svelte (step 3.6)

- [ ] 7.1 Reuse the primitives from step 6 (`Button`, `Separator`, `Tooltip`), then add `Dialog` for any rename / delete / symbol-change confirm flow that today is inline.
- [ ] 7.2 If a per-waypoint colour swatch is exposed, it stays on native `<input type="color">` and binds to domain RGBA only (same constraint as TracksPanel; see D6 in parent design).
- [ ] 7.3 Keep `SymbolPicker` (already migrated in step 4) as the symbol-choice surface inside the row.
- [ ] 7.4 Delete the component-local `<style>` block; layout moves to Tailwind utilities.
- [ ] 7.5 Verify the panel looks correct in all four flavours via `ozi-rs-mcp` screenshots; verify dialog portals above the MapLibre canvas.
- [ ] 7.6 Smoke-test: add a waypoint via the map, rename it via the panel, toggle visibility, change symbol, delete it via the dialog confirm.

## 8. TrackPointsPanel.svelte (step 3.7)

- [ ] 8.1 Replace the manually-rendered rows with `Table` (`Table.Root` / `Table.Header` / `Table.Body` / `Table.Row` / `Table.Cell`); wrap the body in `ScrollArea`.
- [ ] 8.2 Wrap per-row action buttons in `Tooltip`.
- [ ] 8.3 Delete the component-local `<style>` block; selected-row / hover state moves to Tailwind utility classes.
- [ ] 8.4 Keep the segment-expansion `$state` map (`expandedSegments`) and the `getTrackDetail` call site untouched.
- [ ] 8.5 Verify the panel looks correct in all four flavours via `ozi-rs-mcp` screenshots.
- [ ] 8.6 Smoke-test: select a track, open Track Points, expand/collapse segments, select a point, confirm the map highlights it.

## 9. Sidebar.svelte (step 3.8)

- [ ] 9.1 Replace the panel-switcher chrome with `Tabs.Root` + `Tabs.List` + `Tabs.Trigger` + `Tabs.Content`; the per-tab content wraps the already-migrated panels from steps 2–8.
- [ ] 9.2 Wrap the scrollable body of each tab in `ScrollArea`.
- [ ] 9.3 Delete the component-local `<style>` block; sidebar chrome (width, border, background) moves to Tailwind utilities (`w-80 border-r border-border bg-background`).
- [ ] 9.4 Verify the Sidebar looks correct in all four flavours via `ozi-rs-mcp` screenshots.
- [ ] 9.5 Smoke-test: switch between Tracks / Waypoints / Track Points tabs, confirm the active tab content renders and scrolls.

## 10. MapView.svelte (step 3.9 — wrapper only)

- [ ] 10.1 Migrate ONLY the outer wrapper `<div>` that hosts the MapLibre canvas to Tailwind utilities (`relative h-full w-full`).
- [ ] 10.2 Delete the wrapper portion of the component-local `<style>` block. Preserve any rules that target the MapLibre canvas itself if they're load-bearing for the map.
- [ ] 10.3 Confirm the MapLibre instance, source/layer setup, drag handlers, click handlers, and the tile-protocol code are **not touched** in this commit.
- [ ] 10.4 Verify the map renders correctly in all four flavours via `ozi-rs-mcp` screenshots.
- [ ] 10.5 Smoke-test: pan, zoom, click a track, click a waypoint, draw a new track point — all behaviours unchanged from before migration.

## 11. Icons audit + toasts

- [ ] 11.1 Audit text/emoji icons across the migrated panels and replace with `lucide-svelte` icons where they improve clarity (export, import, delete, visibility, rename, undo, redo). NOT a blanket replace — leave SymbolPicker's domain emoji set as-is.
- [ ] 11.2 Route any existing `alert()` / `console.error` notifications surfaced to the user through `svelte-sonner` toasts (the `<Toaster />` host lives in `routes/+layout.svelte` from Change 2).
- [ ] 11.3 Verify icon alignment and toast appearance across all four flavours via `ozi-rs-mcp` screenshots.

## 12. Documentation + final QA

- [ ] 12.1 Update `docs/frontend-architecture.md` to describe the post-migration panel structure (primitives consumed, semantic tokens, retained native colour inputs).
- [ ] 12.2 Update `docs/project-map.md` to reflect any moved imports (`$lib/components/ui/...`) and the deletion of per-component `<style>` blocks.
- [ ] 12.3 Run `just lint`, `just check`, `just clippy`, `just test` — all green.
- [ ] 12.4 Run `tauri build` to confirm the bundled app still launches.
- [ ] 12.5 Final visual smoke pass via `ozi-rs-mcp`: capture one screenshot per flavour (Latte, Frappé, Macchiato, Mocha) with Sidebar + MapView + at least one open panel; compare against the per-step screenshots for regressions.
- [ ] 12.6 Run `openspec validate migrate-panels-to-shadcn --strict` one more time before archive.
