# ui-shell Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
### Requirement: System provides a Catppuccin theme selector with five options

The system SHALL apply one of the Catppuccin palettes — Auto (follow OS), Latte, Frappé, Macchiato, or Mocha — to the UI via two coexisting CSS custom-property layers on the root element:

1. **Palette layer** — hex variables `--ctp-<colour>` for every named colour in `@catppuccin/palette` (e.g. `--ctp-base`, `--ctp-red`, `--ctp-mauve`).
2. **Semantic layer** — HSL-triplet variables (no `hsl()` wrapper) for shadcn-svelte primitives: `--background`, `--foreground`, `--card`, `--card-foreground`, `--popover`, `--popover-foreground`, `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`, `--muted`, `--muted-foreground`, `--accent`, `--accent-foreground`, `--destructive`, `--destructive-foreground`, `--border`, `--input`, `--ring`.

The semantic-layer values SHALL be derived from the active flavour through two distinct mapping tables — `SEMANTIC_MAP_LIGHT` for Latte and `SEMANTIC_MAP_DARK` for Frappé / Macchiato / Mocha — to keep surface semantics correct in both light and dark modes (e.g. `--popover` resolves to `base` in light and `surface0` in dark).

The root element SHALL carry the class `dark` whenever the resolved flavour is not Latte, so shadcn-svelte's Tailwind dark-variant utilities apply correctly.

The Auto option SHALL continue to track the OS light/dark preference dynamically, re-running both layers on every change.

#### Scenario: Pick a manual theme

- **WHEN** the user selects "Mocha" from the theme picker
- **THEN** the UI re-renders with the Mocha palette applied via `--ctp-*` variables AND every semantic token from `SEMANTIC_MAP_DARK` is written to the root element as an HSL triplet AND the root element gains the `dark` class

#### Scenario: Auto follows OS

- **WHEN** the user selects "Auto" and the OS toggles between light and dark mode
- **THEN** the UI switches between Latte (light, root element has no `dark` class) and Mocha (dark, root element has `dark` class) accordingly, with both the palette and semantic layers re-applied on each transition

#### Scenario: Switching from Latte to Mocha updates semantic tokens

- **WHEN** the user switches the flavour from Latte to Mocha
- **THEN** the semantic CSS variables (e.g. `--background`, `--popover`, `--border`) on the root element change from the Latte-mapped HSL triplets to the Mocha-mapped HSL triplets in the same tick

#### Scenario: shadcn Select renders with theme-correct colours after flavour change

- **WHEN** a shadcn-svelte `Select` is open while the user switches the active flavour
- **THEN** the trigger, popover surface, and items pick up the new flavour's colours through Tailwind utilities reading the semantic CSS variables, with no remount required

#### Scenario: Track colour input is not affected by theme switch

- **WHEN** the user has set a per-track or per-waypoint colour via a domain colour input AND the user then switches the active Catppuccin flavour
- **THEN** the per-track / per-waypoint colour stored in the domain (RGBA bytes) and rendered on the map remains exactly the same value; the theme system SHALL NOT read or write any track or waypoint colour

#### Scenario: Semantic tokens are HSL triplets without `hsl()` wrapper

- **WHEN** the system writes any semantic token to the root element
- **THEN** the value matches the pattern `H S% L%` (e.g. `220 23% 95%`), not `hsl(H, S%, L%)`, so Tailwind utility classes such as `bg-background/80` can apply alpha via the configured `hsl(var(--background) / <alpha-value>)` pattern

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

### Requirement: Frontend is bootstrapped via SvelteKit with adapter-static

The frontend SHALL be bootstrapped via SvelteKit using `@sveltejs/adapter-static`. SSR SHALL be disabled (`ssr = false`) and prerender SHALL be enabled (`prerender = true`) at the root layout level. The Tauri shell SHALL load the prebuilt static output from the adapter (`build/`) as its `frontendDist`.

#### Scenario: Production build produces static output Tauri can load

- **WHEN** the developer runs `npm run tauri build`
- **THEN** SvelteKit emits static HTML/JS/CSS into `build/`, and Tauri packages that directory as the application's frontend without requiring a Node runtime

#### Scenario: Dev workflow runs through SvelteKit

- **WHEN** the developer runs `npm run tauri dev`
- **THEN** the Vite dev server is launched via SvelteKit's `sveltekit()` plugin, hot module replacement works for files under `src/routes/` and `src/lib/`, and the Tauri window connects to that dev server

### Requirement: Top-level surfaces live at distinct routes `/` and `/project`

The two top-level surfaces SHALL each live at a dedicated route. The bundle loader (`BundleLoaderView`) SHALL be served at `/` from `src/routes/+page.svelte`. The project workspace (`Sidebar` + panels) SHALL be served at `/project` from `src/routes/project/+page.svelte`. Transitions between the two SHALL be performed client-side via `onMount` plus `goto()`, because prerender precludes runtime store access in route-level `load` functions.

#### Scenario: No bundle loaded — land on the loader

- **WHEN** the application starts with no active map in the store
- **THEN** the active URL is `/` and the bundle loader surface is rendered

#### Scenario: Bundle already loaded — redirect into the workspace

- **WHEN** the application starts and the store already reports an active map (e.g. restored from session)
- **THEN** the user lands on `/` for one paint, `onMount` invokes `goto('/project')`, and the project workspace becomes the active surface

#### Scenario: User navigates back to the loader without an active map

- **WHEN** the user closes the current project and the store reports no active map while the URL is `/project`
- **THEN** `onMount` on the project route invokes `goto('/')` and the bundle loader is shown

### Requirement: The frontend SHALL run as a single Tauri WebviewWindow

The application SHALL ship with exactly one `WebviewWindow` (label `main`). The previously separate `bundles` `WebviewWindow` SHALL be removed; the bundle-loader surface SHALL be reachable as the `/` route in the main window. `src-tauri/capabilities/default.json` SHALL list only `["main"]` under `windows` and SHALL NOT grant `core:window:*` or `core:webview:*` permissions beyond what `core:default` provides.

#### Scenario: Sidebar "Maps…" button navigates within the main window

- **WHEN** the user clicks the "Maps…" button in the workspace sidebar
- **THEN** the active URL becomes `/` inside the existing main window, and no new `WebviewWindow` is created

#### Scenario: Capabilities reflect single-window setup

- **WHEN** the project's `src-tauri/capabilities/default.json` is inspected
- **THEN** the `windows` array equals `["main"]` and no window-management or webview-creation permissions are listed

### Requirement: MapView is mounted once in the root layout and persists across route changes

`MapView` SHALL be mounted inside `src/routes/+layout.svelte` so that navigating between `/` and `/project` does not destroy and re-create the MapLibre map. The layout SHALL toggle the `MapView`'s visibility based on the active route (visible on `/project`, hidden on `/`), without unmounting the component.

#### Scenario: Round-trip between routes preserves the MapLibre map

- **WHEN** the user navigates from `/project` to `/` and back to `/project`
- **THEN** the same MapLibre map instance is used both times, no re-initialisation cost is paid, and prior view state (zoom, pan, registered protocols) is preserved

### Requirement: UI primitives are sourced from the shadcn-svelte library

The frontend SHALL provide a `src/lib/components/ui/` directory containing shadcn-svelte primitives generated via `shadcn-svelte init` and `shadcn-svelte add`. The initial primitive set SHALL include: button, dialog, popover, select, tabs, tooltip, switch, separator, scroll-area, input, label, sonner, slider, card, table. Each primitive SHALL import the `cn` helper from `$lib/utils`.

#### Scenario: A primitive renders against the active theme

- **WHEN** a shadcn-svelte `Button` is mounted anywhere in the app and the active flavour is Mocha
- **THEN** its computed styles resolve through the semantic CSS variables to Mocha-derived HSL colours, with no per-component theme wiring required

#### Scenario: `cn()` helper is available at `$lib/utils`

- **WHEN** any primitive or feature component imports `cn` from `$lib/utils`
- **THEN** the helper is defined as `twMerge(clsx(...))` and exists at that import path (locked by `components.json` from `shadcn-svelte init`)

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

### Requirement: Form panels use felte with zod resolvers

The frontend SHALL provide a `src/lib/forms/create-form.ts` module wrapping `felte`'s `createForm` with a `zod` validation resolver, plus a `src/lib/forms/schemas/` directory ready to hold per-feature zod schemas. Form-based panels added or migrated after this change SHALL build their forms through this module rather than ad-hoc input bindings.

`superforms` is explicitly NOT adopted — it requires SvelteKit form-action runtime which this application does not use.

#### Scenario: A feature panel composes a form

- **WHEN** a feature panel calls the helper exported from `src/lib/forms/create-form.ts` with a zod schema
- **THEN** it receives a typed form whose validation errors and submit handler are wired through felte and zod, without manual schema-to-form glue

### Requirement: Track and waypoint colours are isolated from the theme system

Per-track and per-waypoint colours are domain data (RGBA bytes, interoperable with OziExplorer `COLORREF` import and Garmin named-colour GPX export). The theme system SHALL NOT bind to, read, or write these colours. The semantic-token map SHALL cover only UI chrome (`background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `border`, `input`, `ring`). Per-track and per-waypoint colour inputs SHALL continue to operate on the domain RGBA representation directly.

#### Scenario: Track colour persists across flavour changes

- **WHEN** the user has assigned a specific RGBA colour to a track AND then switches the Catppuccin flavour
- **THEN** the track's stored colour value (in the domain and on disk) is unchanged AND the track renders on the map with that same RGBA colour

#### Scenario: Theme code does not reference track or waypoint colour fields

- **WHEN** the theme module (`src/lib/theme.ts`) is inspected statically
- **THEN** it imports no track or waypoint type and references no domain colour field; the semantic-token allow-list does not overlap with any track or waypoint colour name

### Requirement: Third-party dependencies are credited

The repository SHALL maintain a `THIRD_PARTY_LICENSES.md` file listing the new UI-kit dependencies and their licences (at minimum: Tailwind, shadcn-svelte, bits-ui, lucide-svelte, svelte-sonner, tailwind-variants, clsx, tailwind-merge, felte, zod, `@tailwindcss/typography`, `tailwindcss-animate`, `autoprefixer`, `postcss`, `prettier-plugin-tailwindcss`). The Tauri bundle configuration SHALL reference this file via `tauri.bundle.licenseFile`, and the project README SHALL include a Credits section linking to it.

#### Scenario: A new dependency lands

- **WHEN** a new UI-kit dependency is added to `package.json`
- **THEN** an entry for it exists in `THIRD_PARTY_LICENSES.md` before the change is merged

#### Scenario: The bundle ships the licence file

- **WHEN** `tauri build` produces a release bundle
- **THEN** the bundle includes `THIRD_PARTY_LICENSES.md` via the configured `tauri.bundle.licenseFile` pointer

