## Context

Visual smoke of the redesigned `/project` workspace on the current `main` (commit `5eaf289`) revealed that the three completed redesign changes — `redesign-shell-layout`, `redesign-library-sidebar`, `redesign-inspector-pane` — landed without the affordance polish their specs contract for. The user saw seven undifferentiated text items in a row (4 mode chips + 3 tabs), no visible active Library tab, no Inspector handle in the empty state, truncated names spilling out of rows, a Cmd-K trigger that looks like an input field, and two semantically overlapping "Maps" entry points.

The root cause in each case is a small surface deviation, not an architectural miss:

- The shadcn `Tabs.Trigger` active-state classes assume a contrasting `bg-background` against a `bg-muted` `Tabs.List` container. The Library mounts its `Tabs.List` with `class="bg-card"` over a card-coloured rail, so the active-state `data-active:bg-background` resolves to the same colour as the rail itself — the contrast disappears.
- The mode chips were specced as "inert visual scaffolding" but render as ordinary chip buttons indistinguishable from the tabs across the bar. The spec did not pin their disabled-state styling.
- `InspectorRail` is mounted conditionally by `WorkspaceShell` (`{#if $inspectorOpen}` at line 121), so when nothing is selected the rail is absent and its edge handle / pin affordance has nowhere to live. The spec for `redesign-inspector-pane` says the rail is "mounted unconditionally" — implementation diverged.
- `LibraryRow` does not enforce truncation on the name cell. With long bundle/map names ("Спасатель — Поисково-спасательный отряд…") the row overflows.
- The Cmd-K trigger uses `min-width: 200px`, a border that resembles the shadcn `Input` border, and a placeholder string ("Search…") — three signals that read as a text input.
- The Maps tab header includes both the Maps tab content (a list of switchable maps) AND a `Maps…` button that opens the bundle-loader Sheet. Conceptually these overlap: the user thinks of "switch map" as one action. The bundle-loader Sheet has another entry via the Cmd-K palette (Switch project group), so a single Maps-tab affordance is enough.

## Goals / Non-Goals

**Goals:**
- Make the active Library tab visually unambiguous in both light and dark Zinc themes.
- Distinguish mode chips from tabs as a visually-grouped, clearly inert cluster.
- Restore the Inspector edge handle in the no-selection state by making the rail always-mounted.
- Truncate library row names gracefully with a fallback tooltip.
- Restyle the Cmd-K trigger so it cannot be mistaken for an input.
- Remove the duplicated Maps-tab-header bundle-loader entry.

**Non-Goals:**
- Wiring mode chips to a Mode store. They stay inert visual scaffolding; the spec change here only pins their disabled styling.
- Adding new design tokens. Everything resolves through existing tokens (`--accent`, `--ring`, `--muted`, `--inner-border`, `--radius-pill`, `--radius-card`).
- Reworking the bundle-loader Sheet itself.
- Inspector body subcomponent changes (TrackInspector, WaypointInspector, MapInspector are not touched).

## Decisions

### Decision 1: Active Library tab uses `data-state="active"` driving accent background + foreground swap + 2px Teal under-border

The shadcn `Tabs.Trigger` SHALL be augmented at the call site (or via a local utility class on `Tabs.List` inside `LibraryRail.svelte`) so that the active trigger renders with:

- `data-state="active"`: `bg-accent text-accent-foreground` (Teal-tinted neutral against contrasting text).
- A 2px bottom under-border in `hsl(var(--ring))` (the Teal accent) drawn via `box-shadow: inset 0 -2px 0 hsl(var(--ring))` so it does not affect layout.
- Inactive triggers keep `text-muted-foreground` and the default transparent background.

**Rationale**: the existing shadcn classes assume a `bg-muted` container providing visual contrast; the LibraryRail mounts its `Tabs.List` over the same `bg-card` surface as the rail itself, so the `data-active:bg-background` selector collides. Switching the active token to `bg-accent` reuses an already-themed token that contrasts in both light and dark Zinc, and adds an explicit Teal under-border so the active state survives at a glance.

**Alternatives considered**:
- _Restyling `Tabs.List` to `bg-muted`_: rejected — the rail looks crowded with a second background layer; the under-border + accent swap is enough.
- _Custom CSS variable for tab-active surface_: rejected — uses an existing token (`--accent`) rather than introducing a new one.

### Decision 2: Mode chips render as a disabled-styled cluster with a vertical separator before the Cmd-K trigger

The mode chips SHALL render with `aria-disabled="true"`, `tabindex="-1"`, and a static disabled look: `opacity: 0.55`, `cursor: not-allowed`, no hover state. The cluster SHALL be wrapped in a `role="group"` container and SHALL be visually separated from the Cmd-K trigger by a 1px vertical divider (`box-shadow: inset 1px 0 0 var(--inner-border)` on the trigger-side container, or a `<span class="divider" aria-hidden="true">` element with `width: 1px; height: 16px; background: var(--inner-border)`).

When the mode store lands in a future change, this requirement SHALL relax: the chips become interactive and the disabled styling is dropped. Until then, the chips look explicitly "not yet" rather than "broken peer of the tabs".

**Rationale**: rendering inert affordances as visually identical to working ones is worse than rendering them as visibly disabled. The user has already reported confusion. Disabled styling + a separator + the existing pill shape on the trigger leaves a clear three-zone reading: [inert chips] | [active Cmd-K pill].

**Alternatives considered**:
- _Hide the chips until wired_: rejected — the spec keeps the chips as placeholder scaffolding so the bar's height does not jump when they're wired. Hiding regresses the stable-layout property.
- _Wire chips here_: rejected — out of scope. The mode store is its own change.

### Decision 3: Inspector rail is always mounted; the edge handle is the no-selection state

`WorkspaceShell` SHALL drop the `{#if $inspectorOpen}` gate around the inspector aside and SHALL always render the inspector slot. `InspectorRail.svelte` SHALL gain two visual states:

- **Collapsed** (no selection AND not pinned): the rail collapses its body to zero width but renders an 8px-wide edge handle on the right edge of the viewport. The handle hosts the pin toggle (centred vertically) and has a subtle hover affordance (`cursor: ew-resize` → on click, set `pinned = true`).
- **Expanded** (selection present OR pinned): the rail renders at 360px (the current width), with the same body subcomponents already implemented.

The `inspectorOpen` store SHALL keep its current contract (`true` when the rail is expanded) so that `WorkspaceShell.writeCanvasInsets` continues to compute `--canvas-right` correctly. When the rail is collapsed, `--canvas-right` SHALL be set to `8px` (the handle width) rather than `0px`, so the canvas does not extend underneath the handle.

**Rationale**: the spec for `redesign-inspector-pane` says explicitly "The rail SHALL be mounted unconditionally for the lifetime of the workspace surface; only its body content SHALL appear and disappear in response to selection." The current implementation diverged. Restoring the unconditional mount + edge handle aligns the code with the spec and gives the user the manual open affordance they need.

**Alternatives considered**:
- _Floating handle outside the rail_: rejected — the handle would need its own absolute-positioning logic and would not benefit from the rail's slide animation; cleaner to keep it inside the rail.
- _Wider handle (16px)_: rejected — 8px (12px on hover) matches comparable affordances in modern desktop IDEs (VS Code edit-bar handles) and reserves screen real estate for the map.

### Decision 4: Library row truncation uses CSS-only `text-overflow: ellipsis` + native `title` attribute

`LibraryRow.svelte` SHALL apply `text-overflow: ellipsis`, `white-space: nowrap`, and `overflow: hidden` to the name span, with the parent flex item set to `min-width: 0` so flex shrinkage actually takes effect. The full name SHALL be set on the `title` attribute of the name span for a native browser tooltip on hover.

If the existing shadcn `Tooltip` wrapper is preserved on the row (it is, for design consistency), the `title` is harmless duplication that helps in cases where the Tooltip mounts late. The Tooltip remains the primary mechanism; `title` is the fallback.

**Rationale**: cheapest possible truncation; no JS, no layout measurement. `min-width: 0` is the standard fix for flex children that refuse to shrink below their intrinsic content width — without it, `text-overflow` never triggers.

**Alternatives considered**:
- _JS-based truncation with measured width_: rejected — premature optimisation; CSS is enough.
- _Two-line wrap_: rejected — destabilises row height, harms the dense row pattern.

### Decision 5: Cmd-K trigger renders as a button-pill with no input-like affordances

The `cmdk-trigger` SHALL:

- Drop the `min-width: 200px` (sized to content via `padding-inline`).
- Use `border: 1px solid transparent` by default (NOT the `--border` token, which mimics `Input`'s border).
- Set the hover background to `hsl(var(--secondary))` and the label colour to `hsl(var(--muted-foreground))`.
- Set focus styling to a 2px Teal ring (`box-shadow: 0 0 0 2px hsl(var(--ring) / 0.45)`) rather than an input-style outline — this signals "button focus", not "input focus".
- Render its label as `"Search…"` followed by the `⌘K` kbd pill — same content as today, but the container is unambiguously button-shaped.

Critically: the trigger SHALL NOT carry `role="search"`, SHALL NOT use `<input>` markup, and SHALL NOT expose any `aria-` attribute that hints at text entry. The button retains `aria-label="Open command palette"`.

**Rationale**: the user reported trying to type into the trigger. Three signals contributed: input-mimicking border, wide min-width, "Search…" placeholder text. Removing the border-token and the min-width while keeping the label inside a `<button>` resolves the ambiguity without changing the user-facing copy.

**Alternatives considered**:
- _Replace label with "Command palette"_: rejected — `Search…` is the established shadcn convention for Cmd-K triggers (matches `cmdk` upstream).
- _Show only the `⌘K` glyph_: rejected — discoverability suffers; the label tells first-time users what the button does.

### Decision 6: Maps-tab header has no bundle-loader Sheet trigger

The `<header>` block inside `src/components/library/MapsTab.svelte` SHALL be removed entirely. The `<Button>` opening the bundle-loader Sheet SHALL be removed. The `bundleLoaderOpen` import inside this component SHALL also be removed (the Sheet host elsewhere keeps its own subscription).

The Maps tab body SHALL continue to list the active project's maps with the active map highlighted, exactly as today. Switching maps within the active project is the Maps tab's purpose; switching projects / loading new bundles is the Cmd-K palette's (`commandPaletteOpen` already wires the Switch project group to `bundleLoaderOpen.set(true)` in `src/components/CommandPalette.svelte:142`).

**Rationale**: the user perceived two affordances ("Maps tab" + "Maps…" button) as semantically duplicated for the same action. They are: both lead the user to "pick a different map". The redundant header button is the cheaper of the two to drop, and Cmd-K already provides the bundle-loader entry for power users.

**Alternatives considered**:
- _Keep the Maps button, drop the Maps tab body_: rejected — the Maps tab body is the always-visible inline access to currently-loaded maps; dropping it makes map-switching modal-only.
- _Relabel the Maps button to "Load more maps…"_: rejected — still a duplicate entry point; copy alone does not resolve semantic overlap.
- _Move the Maps button to the top context-bar_: rejected — would re-introduce the spec-violating "Library has chrome" pattern. Project-lifecycle and bundle-loader entries belong on the context-bar or Cmd-K only; the Library is for **objects in the current project** (see `ui-shell` spec, "Project lifecycle and mode toggles do not live in the Library").

## Risks / Trade-offs

- **Risk**: removing the `{#if $inspectorOpen}` gate in `WorkspaceShell` means the inspector aside is always in the DOM, increasing the workspace tree's node count slightly. **Mitigation**: the rail's body content is still conditionally rendered (the existing rune `activeKind` already gates this), so the cost is purely the rail chrome — header + handle. Negligible at the workspace scale.
- **Risk**: dropping the Maps-tab-header button could surprise users who learned the old entry point. **Mitigation**: the Cmd-K palette is the discoverable replacement, and the `⌘K` trigger now reads as a button (Decision 5) so its discoverability improves in the same change.
- **Risk**: the 8px handle is narrow and could be a hit-target problem on touch devices. **Mitigation**: this is a desktop-only Tauri app; touch is not a target. On hover, the handle widens to 12px and the cursor changes — both within Fitts's-law reasonable bounds for a mouse pointer.
- **Trade-off**: rendering mode chips as visibly disabled signals "this is not done yet" to the user. Acceptable: the alternative (hiding them) regresses the stable-layout property of the context-bar; visible-but-disabled keeps the bar's height invariant when chips are wired in a later change.

## Migration Plan

Single-step implementation, no data migration required:

1. Update `tabs-trigger.svelte` (or LibraryRail-local override) to add the active-state classes; verify against `data-state="active"` attribute on the rendered `<button>`.
2. Update `WorkspaceShell.svelte` to mark the mode chips disabled and add the divider before the Cmd-K trigger; restyle the Cmd-K trigger to drop the input-mimicking border and min-width.
3. Remove the `{#if $inspectorOpen}` gate around `<aside class="rail inspector">` in `WorkspaceShell.svelte` and update `writeCanvasInsets` to write `8px` (collapsed) / `360px` (expanded).
4. Restructure `InspectorRail.svelte` to render an edge handle in the collapsed state; keep `inspectorOpen` writes intact.
5. Add truncation rules to `LibraryRow.svelte`'s name span; ensure flex parent has `min-width: 0`.
6. Remove the `<header>` block from `MapsTab.svelte`; remove the now-unused `bundleLoaderOpen` import.
7. Visual smoke: capture screenshots in the auto / light / dark Zinc themes and verify each affordance.

No backend changes, no IPC changes, no store schema changes. Existing `inspectorOpen` consumers continue to read the same boolean semantics.
