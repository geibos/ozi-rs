## MODIFIED Requirements

### Requirement: System provides a Catppuccin theme selector with five options

The system SHALL apply one of the Catppuccin palettes — Auto (follow OS), Latte, Frappé, Macchiato, or Mocha — to the UI via CSS custom properties (`--ctp-*`) and the semantic-token layer (`--background`, `--foreground`, `--primary`, …) introduced by `add-design-tokens-and-shadcn`. The Auto option SHALL track the OS light/dark preference dynamically. The theme picker SHALL render through the shadcn-svelte `Select` primitive, not a native `<select>` element.

#### Scenario: Pick a manual theme

- **WHEN** the user selects "Mocha" from the theme picker
- **THEN** the UI re-renders with the Mocha palette applied via both `--ctp-*` variables and the matching semantic tokens; every panel that uses Tailwind utility classes (`bg-background`, `text-foreground`, …) re-renders to the Mocha equivalent

#### Scenario: Auto follows OS

- **WHEN** the user selects "Auto" and the OS toggles between light and dark mode
- **THEN** the UI switches between Latte (light) and Mocha (dark) accordingly, and the `class="dark"` toggle on `<html>` is set whenever the active flavour is not Latte

### Requirement: All in-app panels render through shadcn-svelte primitives and Tailwind utility classes that consume semantic tokens

The system SHALL render every in-app panel (`Console`, `ThemePicker`, `SymbolPicker`, `SimplifyPanel`, `TracksPanel`, `WaypointsPanel`, `TrackPointsPanel`, `Sidebar`, `MapView` wrapper) through shadcn-svelte primitives — `Card`, `ScrollArea`, `Button`, `Select`, `Popover`, `Tooltip`, `Slider`, `Switch`, `Label`, `Separator`, `Dialog`, `Table`, `Tabs` — and SHALL style their static surfaces through Tailwind utility classes that read the semantic-token CSS variables (`bg-background`, `text-foreground`, `bg-popover`, `border-border`, `bg-card`, `bg-muted`, …).

Component-local `<style>` blocks SHALL be removed, except where a rule expresses a dynamic value sourced from the domain (e.g. a `TrackStyle.color` swatch). Such dynamic values SHALL be expressed as inline `style=` attributes, not as Tailwind classes.

`MapView.svelte` SHALL be migrated at the wrapper level only; MapLibre initialization, source/layer setup, drag handlers, click handlers, and the tile-protocol code SHALL NOT be modified by this requirement.

`TracksPanel.svelte` and `WaypointsPanel.svelte` SHALL retain their native `<input type="color">` controls for track and waypoint colour selection. These controls SHALL operate on the RGBA domain values stored in `TrackStyle.color` (and the equivalent waypoint property) and SHALL NOT be bound to `--ctp-*` palette variables or to semantic-token CSS variables.

Toast notifications SHALL be routed through `svelte-sonner` mounted in the root layout, rather than ad-hoc `alert()` or inline error surfaces.

#### Scenario: Panel surfaces follow the active flavour

- **WHEN** the user switches the active Catppuccin flavour while any panel is open
- **THEN** the panel's static surfaces (background, foreground text, borders, dividers, hover states) re-render through the new flavour's semantic-token values without any `<style>`-block override

#### Scenario: Track colour swatch is independent of the theme switch

- **WHEN** the user assigns a track a specific colour (e.g. `#ff8800`) via the native `<input type="color">` in `TracksPanel`, then switches the Catppuccin flavour
- **THEN** the track's colour swatch and its MapLibre rendering remain exactly `#ff8800` across all four flavours; no `--ctp-*` or semantic-token variable rebinds the swatch

#### Scenario: All four flavours pass the visual smoke pass

- **WHEN** the maintainer captures screenshots of `Sidebar` + `MapView` + an open `TracksPanel` (with at least one track loaded) across all four Catppuccin flavours via `ozi-rs-mcp`
- **THEN** each flavour renders without missing styles, without leftover hard-coded colours that bypass semantic tokens, and without regressions to MapLibre map content

#### Scenario: MapView wrapper migrates without touching MapLibre internals

- **WHEN** the maintainer reviews the `MapView.svelte` migration commit
- **THEN** the diff modifies only the outer wrapper container's classes (Tailwind utilities) and any Tailwind-replaced wrapper styles; MapLibre `new maplibregl.Map(...)` construction, source/layer setup, drag handlers, click handlers, and the tile-protocol code SHALL be byte-identical to before the commit

### Requirement: Theme choice persists across sessions via localStorage

The system SHALL persist the selected theme in browser localStorage and SHALL restore it on the next session. Theme is intentionally NOT stored in the Rust session file (see `project-persistence`).

#### Scenario: Theme survives restart

- **WHEN** the user selects "Frappé" and restarts the application
- **THEN** the UI starts in Frappé without prompting the user

### Requirement: Backtick key toggles an in-app developer console

The system SHALL toggle visibility of an in-app developer console whenever the user presses the backtick (`` ` ``) key while the application has focus. The console SHALL render through the shadcn-svelte `Card` primitive with a `ScrollArea` body and a `Button` close affordance.

#### Scenario: Open and close console

- **WHEN** the user presses backtick once and then again
- **THEN** the developer console appears on the first press (rendered as a `Card` with theme-aware semantic-token surfaces) and disappears on the second

### Requirement: F3 toggles an FPS counter overlay

The system SHALL toggle visibility of a frame-rate counter overlay whenever the user presses F3. The counter SHALL display real-time FPS computed from frame times.

#### Scenario: Toggle FPS overlay

- **WHEN** the user presses F3
- **THEN** an FPS overlay appears in a corner of the application window and updates continuously until F3 is pressed again
