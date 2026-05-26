## 1. Library tab active-state contrast

- [ ] 1.1 In `src/components/LibraryRail.svelte`, augment each `<Tabs.Trigger>` (or wrap with a local class) so the active state renders `data-active:bg-accent`, `data-active:text-accent-foreground`, and an inset 2px under-border in `hsl(var(--ring))` (e.g. `data-active:shadow-[inset_0_-2px_0_hsl(var(--ring))]`); confirm `Tabs.List` keeps its `bg-card` background without competing with the active swap
- [ ] 1.2 Verify visually that the active tab is unambiguous in all three Zinc theme states (auto / light / dark) without modifying `src/lib/tokens.css`

## 2. Mode chips disabled styling + divider from Cmd-K trigger

- [ ] 2.1 In `src/components/WorkspaceShell.svelte`, set `aria-disabled="true"` and keep `tabindex="-1"` on each `.chip`; add `cursor: not-allowed`, `opacity: 0.55`, and drop the hover background change so chips read as inert
- [ ] 2.2 Add a 1px vertical divider between `.mode-chips` and `.cmdk-trigger` — either a `<span class="divider" aria-hidden="true">` (`width: 1px; height: 16px; background: var(--inner-border)`) or an inset box-shadow on the trigger-side container
- [ ] 2.3 Confirm the context-bar's 36px height is preserved (no layout shift introduced by the divider element)

## 3. Cmd-K trigger restyle (no input-mimicking signals)

- [ ] 3.1 In `src/components/WorkspaceShell.svelte` `.cmdk-trigger` rules: remove `min-width: 200px`, change `border: 1px solid hsl(var(--border))` to `border: 1px solid transparent`, set hover background to `hsl(var(--secondary))`, set focus styling via `:focus-visible { box-shadow: 0 0 0 2px hsl(var(--ring) / 0.45); outline: none; }`
- [ ] 3.2 Confirm the `<button type="button" class="cmdk-trigger" aria-label="Open command palette">` element does not carry `role="search"` and uses no `<input>` markup
- [ ] 3.3 Click-smoke: clicking the trigger still calls `commandPaletteOpen.set(true)`; tabbing focus into the trigger renders the Teal ring rather than an input-style outline

## 4. Inspector always-mounted with edge handle

- [ ] 4.1 In `src/components/WorkspaceShell.svelte`, remove the `{#if $inspectorOpen}` gate around the `<aside class="rail inspector">` block so the inspector aside is always present in the DOM
- [ ] 4.2 Update `writeCanvasInsets` to set `--canvas-right` to `8px` when `$inspectorOpen === false` (was `0px`) and `360px` when expanded; verify `INSPECTOR_WIDTH` constant covers the expanded path
- [ ] 4.3 Update the `.shell` grid-template-columns rules so the collapsed-inspector path renders `280px minmax(0, 1fr) 8px` and the expanded path keeps `280px minmax(0, 1fr) 360px`

## 5. Inspector rail edge-handle states

- [ ] 5.1 In `src/components/InspectorRail.svelte`, introduce a collapsed-state branch: when `activeKind === null && !pinned`, render only an edge handle (an 8px-wide vertical strip on the right edge with the pin button centred vertically); when expanded, render the existing header + body
- [ ] 5.2 The edge handle SHALL widen to ≈12px on hover with `cursor: ew-resize`; clicking the pin button on the handle SHALL set `pinned = true` and trigger the slide-in expansion
- [ ] 5.3 Confirm the existing `inspectorOpen` `$effect` continues to write `true / false` correctly across the collapsed ⇄ expanded transitions (the rune already reads from `activeKind` and `pinned`)
- [ ] 5.4 Visual smoke: from a cold workspace with no selection, confirm an 8px handle is visible on the right edge; clicking the pin keeps the rail expanded after the selection clears

## 6. Library row truncation

- [ ] 6.1 In `src/components/library/LibraryRow.svelte`, add `min-width: 0` to the flex parent containing the name span (the cell wrapping the primary name + optional subline) and apply `white-space: nowrap`, `overflow: hidden`, `text-overflow: ellipsis` to the name span itself
- [ ] 6.2 Set `title={name}` on the name span so the browser tooltip renders the full untruncated text on hover; the existing shadcn `Tooltip` (if any) is left as the primary affordance and `title` is the fallback
- [ ] 6.3 Apply the same truncation rule to the optional Tracks-row subline (`text-xs text-muted-foreground font-mono tabular-nums`) so distance / duration / point-count strings don't overflow either

## 7. Maps tab header — drop the bundle-loader Sheet trigger

- [ ] 7.1 In `src/components/library/MapsTab.svelte`, remove the entire `<header>` block containing the `Maps…` button
- [ ] 7.2 Remove the `bundleLoaderOpen` import from the script tag (the Sheet host elsewhere keeps its own subscription); remove the `openLoader` function
- [ ] 7.3 Keep the `<div class="flex-1 overflow-y-auto py-1">` map-list body intact; confirm the empty-state ("No maps in this project") still renders correctly
- [ ] 7.4 Confirm the bundle-loader Sheet remains reachable from the Cmd-K command palette (verify `src/components/CommandPalette.svelte` Switch project group still calls `bundleLoaderOpen.set(true)`)

## 8. Verification

- [ ] 8.1 Visual smoke at `/project`: capture screenshots in auto / light / dark Zinc and confirm: active Library tab is clearly distinct from inactive tabs; mode chips look inert and separated from the Cmd-K trigger; inspector edge handle visible with no selection; long row names truncate with ellipsis and surface full text on hover; Cmd-K trigger does not look like an input
- [ ] 8.2 Click-smoke: clicking a mode chip changes nothing; clicking the Cmd-K trigger opens the palette; clicking the inspector handle's pin expands the rail; clicking a Library row that triggers a selection auto-expands the rail
- [ ] 8.3 `just ci` passes (clippy, check, lint, test)
- [ ] 8.4 `openspec validate fix-redesign-visual-affordances --strict` passes
