## ADDED Requirements

### Requirement: Workspace renders through a 3-pane shell with named slots

The project workspace at `/project` SHALL render through a single shell component (`src/components/WorkspaceShell.svelte`) that lays out a 3-pane structure with three named regions:

1. **`library-rail`** — a left rail with a fixed width of 280px, intended to host the library content (project picker, layer tree, tracks list, waypoints list, "Maps…" affordance). In this change the slot SHALL render empty placeholder content; the `library-sidebar` change fills it.
2. **`canvas`** — a center region of flexible width that hosts `MapView`. `MapView` SHALL continue to be mounted from the root layout (`src/routes/+layout.svelte`); the canvas slot SHALL be the visual region the layout's `MapView` becomes visible inside while the workspace route is active.
3. **`inspector-rail`** — a right rail with a fixed width of 360px, intended to host selection-driven detail panels. In this change the slot SHALL render empty placeholder content; the `inspector-pane` change fills it. The right rail SHALL be present in the DOM only when its content store is non-empty; when empty, the canvas SHALL grow into the freed horizontal space without shifting the library rail.

Above the canvas the shell SHALL render a thin top context-bar containing four mode-chip placeholders labelled View / Draw / Edit / Measure and a single Cmd-K trigger button. Below the canvas the shell SHALL render a status bar of stable height. Neither the mode chips nor the Cmd-K trigger SHALL be wired to any state or behavior in this change — they are visual scaffolding only.

The previous workspace layout (the 2-column `Sidebar.svelte` + 3 floating panels above `MapView` at `src/routes/project/+page.svelte`) SHALL be replaced by the shell. `Sidebar.svelte` MAY remain in the repository tree for the `library-sidebar` change to reuse or delete; it SHALL NOT be mounted from `WorkspaceShell.svelte` in this change.

#### Scenario: Workspace renders the 3-pane shell with empty rail slots

- **WHEN** the user navigates to `/project` with an active map and the rail slots have no content from later changes
- **THEN** the workspace renders a 280px left rail with empty placeholder, the center canvas with `MapView`, the top context-bar with four mode-chip placeholders and a Cmd-K trigger button, and the bottom status bar with stable height; the right inspector rail is absent from the DOM (its content store is empty)

#### Scenario: Right rail appears only when inspector content is present

- **WHEN** a follow-up change populates the inspector content store
- **THEN** the right rail mounts at 360px and the canvas reflows to leave room for it; **WHEN** the inspector store is cleared, the right rail unmounts and the canvas reclaims the 360px of horizontal space without shifting the library rail

#### Scenario: Mode chips and Cmd-K trigger are inert scaffolding

- **WHEN** the user clicks any mode-chip placeholder or the Cmd-K trigger button
- **THEN** no application state changes and no palette opens (the actual wiring is deferred to `library-sidebar` and `inspector-pane` respectively)

#### Scenario: MapView remains mounted once across the redesign

- **WHEN** the user navigates between `/` and `/project` after this change is implemented
- **THEN** the same `MapView` instance is reused (the mount-once invariant from the prior `ui-shell` requirement is preserved); the shell component does NOT remount `MapView` on slot changes

### Requirement: Design tokens are declared in a dedicated tokens layer with Zinc neutrals and a single Teal accent

The application SHALL declare its design tokens in `src/lib/tokens.css` via Tailwind v4 `@theme`. The token layer SHALL declare:

- **Neutrals**: a Zinc scale (slate-tinted-cool), with the dark surface base set to off-black `#0a0a0a` (never pure `#000`).
- **Accent**: a single Teal hue, with saturation capped at 70%. The token layer SHALL NOT declare a secondary accent variable.
- **Radii**: at least `--radius-card` (1.5rem) and `--radius-pill` (full).
- **Shadow tokens**: at least `--shadow-elev-1`, `--shadow-elev-2`, `--shadow-elev-3` with subtle hue tinting (not flat gray drop shadow).
- **Inner-border token**: a 1px inner border colour resolving to `zinc-200/50` in light mode and `zinc-800/60` in dark mode.

The token layer SHALL be `@import`ed from `src/app.css` so utility classes (`bg-background`, `border-border`, etc.) resolve against the new tokens.

Inspector cards (introduced by the `inspector-pane` change) SHALL consume `--radius-card` for their outer corner radius.

#### Scenario: Token file declares the documented variables

- **WHEN** a static scan of `src/lib/tokens.css` reads its `@theme` block
- **THEN** the block declares the Zinc neutral scale, the Teal accent, `--radius-card`, `--radius-pill`, `--shadow-elev-1`, `--shadow-elev-2`, `--shadow-elev-3`, and the inner-border token; the block does NOT declare a secondary accent variable

#### Scenario: Off-black is used in place of pure black

- **WHEN** the dark theme's base background colour is inspected
- **THEN** the resolved value is `#0a0a0a` (or its OKLCH/RGB equivalent), not `#000000`

### Requirement: UI font is Geist and numeric font is Geist Mono; Inter is banned

The application SHALL load Geist (variable) as the UI sans font and Geist Mono as the monospace font via the `geist` npm package, and SHALL wire both into the Tailwind theme so utility classes `font-sans` and `font-mono` resolve to Geist and Geist Mono respectively. Numeric content (counts, coordinates, byte sizes, elapsed times) SHALL use `font-mono tabular-nums`.

Inter SHALL NOT be imported, declared as a font-family, or referenced in any component's `<style>` block. The ban is explicit because Inter is the obvious default and prone to drift back in via copy-pasted snippets.

#### Scenario: Geist resolves through `font-sans`

- **WHEN** a primitive consuming `font-sans` (e.g. shadcn-svelte `Button`) is rendered
- **THEN** the computed `font-family` resolves to `Geist` (with the appropriate fallback chain), not `Inter` and not a system default

#### Scenario: Numbers in the status bar are tabular

- **WHEN** the status bar renders a numeric byte counter or elapsed time during a bundle download
- **THEN** the numeric glyphs render through `font-mono tabular-nums` so digit width is constant across digits 0-9

#### Scenario: No reference to Inter remains in the source tree

- **WHEN** a static scan greps the `src/` tree for case-insensitive `inter`
- **THEN** no font import, no `font-family: Inter`, and no Tailwind theme override referencing Inter is found

### Requirement: shadcn-svelte `sheet` and `command` primitives are available

The frontend SHALL provide shadcn-svelte primitive files for both `sheet` (slide-in side panel) and `command` (Cmd-K palette / fuzzy-search list) at `src/lib/components/ui/sheet/` and `src/lib/components/ui/command/`. Both primitives SHALL be generated via `npx shadcn-svelte@latest add sheet` and `npx shadcn-svelte@latest add command`, SHALL import `cn` from `$lib/utils`, and SHALL style their surfaces through Tailwind utility classes that resolve against the semantic tokens (`bg-background`, `text-foreground`, `border-border`, etc.).

This requirement covers primitive availability only. The call sites that mount the Sheet (the "Maps…" affordance in the workspace) and the Command palette (the Cmd-K palette) are wired by the follow-up changes `library-sidebar` and `inspector-pane` respectively; this change SHALL NOT introduce those call sites.

#### Scenario: Sheet primitive files are present

- **WHEN** the `src/lib/components/ui/sheet/` directory is inspected
- **THEN** the shadcn-svelte sheet primitive files exist, import `cn` from `$lib/utils`, and style surfaces through semantic-token Tailwind utilities

#### Scenario: Command primitive files are present

- **WHEN** the `src/lib/components/ui/command/` directory is inspected
- **THEN** the shadcn-svelte command primitive files exist, import `cn` from `$lib/utils`, and style surfaces through semantic-token Tailwind utilities

#### Scenario: No call site mounts these primitives in this change

- **WHEN** a static scan of `src/` for `import * as Sheet from "$lib/components/ui/sheet"` and the equivalent Command import is run
- **THEN** the primitive files are present but no application component imports them (the wiring lands in follow-up changes); the primitives are nonetheless tree-available for those changes to consume

### Requirement: Workspace status bar has a stable outer layout

The status bar at the bottom of the workspace shell SHALL have an outer height equal to a single CSS value that is also used as the bottom inset of the workspace grid. Content slots inside the bar (status message, progress chip, action affordance) MAY appear and disappear, but their containers SHALL reserve their footprint so that neighbouring UI (the canvas, the rails) does not reflow when status events arrive.

This requirement carries forward the stable-layout contract previously specified for the bundle-loader status bar and SHALL be preserved across this redesign.

#### Scenario: Status events do not reflow the workspace

- **WHEN** a bundle download is in progress while the user is at `/project` AND `download-progress` / `bundle-progress` events arrive at 5+ events per second
- **THEN** the bottom edge of the status bar remains at a constant vertical position relative to the viewport AND the canvas above it does not shift

#### Scenario: Idle and busy states share the same outer height

- **WHEN** no download is active AND when a download is in progress
- **THEN** the status bar occupies the same outer height in both states; toggles inside the bar only change which content is visible, not the bar's outer footprint

## MODIFIED Requirements

### Requirement: System provides a Catppuccin theme selector with five options

The system SHALL continue to offer the Catppuccin palette family — Auto (follow OS), Latte, Frappé, Macchiato, or Mocha — as an OPT-IN theme pack rather than the default. The default theme of the application SHALL be the native auto/light/dark Zinc + Teal system declared in `src/lib/tokens.css`; the Catppuccin selector SHALL become reachable from settings rather than being the front-line theme picker.

When the user enables the Catppuccin pack from settings AND selects a flavour, the application SHALL apply the Catppuccin layers exactly as before this change: a palette layer (`--ctp-<colour>` hex variables for every named colour in `@catppuccin/palette`) and a semantic layer (HSL-triplet variables for shadcn-svelte primitives: `--background`, `--foreground`, `--card`, `--card-foreground`, `--popover`, `--popover-foreground`, `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`, `--muted`, `--muted-foreground`, `--accent`, `--accent-foreground`, `--destructive`, `--destructive-foreground`, `--border`, `--input`, `--ring`). The semantic-layer values SHALL be derived from the active flavour through the same two mapping tables — `SEMANTIC_MAP_LIGHT` for Latte and `SEMANTIC_MAP_DARK` for Frappé / Macchiato / Mocha.

The Catppuccin CSS variables SHALL live in `src/lib/themes/catppuccin.css`, loaded only when the pack is enabled. The root element SHALL carry the class `dark` whenever the resolved flavour is not Latte. The Auto option within the pack SHALL continue to track the OS light/dark preference dynamically.

The default native theme SHALL also support auto-tracking OS light/dark and explicit light/dark overrides.

#### Scenario: Default native theme on first launch

- **WHEN** the user launches the application for the first time after this change is implemented
- **THEN** the active theme is the native Zinc + Teal system tracking the OS light/dark preference, AND no Catppuccin variable (`--ctp-*`) is present on the root element

#### Scenario: User opts into Catppuccin from settings

- **WHEN** the user enables the Catppuccin theme pack from settings AND selects "Mocha"
- **THEN** the UI re-renders with the Mocha palette applied via `--ctp-*` variables AND every semantic token from `SEMANTIC_MAP_DARK` is written to the root element as an HSL triplet AND the root element gains the `dark` class

#### Scenario: Auto follows OS within the native default

- **WHEN** the active theme is the native default AND the OS toggles between light and dark mode
- **THEN** the UI switches between the native light Zinc surface and the native dark Zinc surface accordingly, with the Teal accent re-resolving against the new neutral

#### Scenario: Auto follows OS within the Catppuccin pack

- **WHEN** the Catppuccin pack is enabled AND the user selects "Auto" AND the OS toggles between light and dark mode
- **THEN** the UI switches between Latte (light, root element has no `dark` class) and Mocha (dark, root element has `dark` class) accordingly, with both the palette and semantic layers re-applied on each transition

#### Scenario: Switching from native default to Catppuccin updates semantic tokens

- **WHEN** the user switches from the native default to a Catppuccin flavour
- **THEN** the semantic CSS variables on the root element change from the native Zinc-derived triplets to the Catppuccin-flavour-derived HSL triplets in the same tick

#### Scenario: Track colour input is not affected by theme switch

- **WHEN** the user has set a per-track or per-waypoint colour via a domain colour input AND the user then switches the active theme (native to Catppuccin, between Catppuccin flavours, or between native light/dark)
- **THEN** the per-track / per-waypoint colour stored in the domain (RGBA bytes) and rendered on the map remains exactly the same value; the theme system SHALL NOT read or write any track or waypoint colour

#### Scenario: Semantic tokens are HSL triplets without `hsl()` wrapper

- **WHEN** the system writes any semantic token to the root element (native default or Catppuccin pack)
- **THEN** the value matches the pattern `H S% L%` (e.g. `220 23% 95%`), not `hsl(H, S%, L%)`, so Tailwind utility classes such as `bg-background/80` can apply alpha via the configured `hsl(var(--background) / <alpha-value>)` pattern

### Requirement: UI primitives are sourced from the shadcn-svelte library

The frontend SHALL provide a `src/lib/components/ui/` directory containing shadcn-svelte primitives generated via `shadcn-svelte init` and `shadcn-svelte add`. The primitive set SHALL include: button, dialog, popover, select, tabs, tooltip, switch, separator, scroll-area, input, label, sonner, slider, card, table, **sheet**, **command**. Each primitive SHALL import the `cn` helper from `$lib/utils`.

#### Scenario: A primitive renders against the active theme

- **WHEN** a shadcn-svelte `Button` is mounted anywhere in the app and the active theme is the native default light
- **THEN** its computed styles resolve through the semantic CSS variables to the native light Zinc + Teal palette, with no per-component theme wiring required

#### Scenario: `cn()` helper is available at `$lib/utils`

- **WHEN** any primitive or feature component imports `cn` from `$lib/utils`
- **THEN** the helper is defined as `twMerge(clsx(...))` and exists at that import path (locked by `components.json` from `shadcn-svelte init`)

#### Scenario: Sheet and Command primitives are in the set

- **WHEN** the `src/lib/components/ui/` directory is listed
- **THEN** both `sheet/` and `command/` subdirectories exist alongside the previously-required primitives
