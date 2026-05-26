# ui-shell Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
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

### Requirement: Bundle-download progress region has a stable layout

The bundle-download progress region in the bundle loader SHALL have an outer layout whose dimensions do not change as progress, file-ready, and download events arrive. Content slots inside the region (status message, current-file label, indeterminate bar, byte counters, progress bar, ready-files list, action buttons) MAY appear and disappear, but their containers SHALL reserve their footprint so that neighbouring UI does not reflow.

The total height of the progress region SHALL be a single CSS value reused both as the height of the region itself and as the bottom inset of the main bundle-loader grid.

#### Scenario: Progress events do not reflow the page

- **WHEN** a bundle download is in progress AND progress, file-ready, and download events arrive at 5+ events per second
- **THEN** the bottom edge of the progress region remains at a constant vertical position relative to the viewport AND the project list / map list above it does not shift

#### Scenario: Idle and busy states share the same outer height

- **WHEN** the application is idle (no download) AND when a download is in progress
- **THEN** the progress region occupies the same outer height in both states; toggles inside the region only change which content is visible, not the region's footprint

#### Scenario: Ready-files list grows inside its reserved slot

- **WHEN** files complete during a download and are appended to the ready-files list
- **THEN** the ready-files list scrolls inside its reserved slot rather than expanding the surrounding region

### Requirement: Bundle loader hydrates the project list from the local catalog cache before the first paint

The bundle loader (`src/routes/+page.svelte`) SHALL display the cached LizaAlert project catalog on its first paint when a cache exists. The hydration SHALL happen synchronously at frontend module initialization (in `src/lib/stores.ts`), so that by the time the page mounts, the projects store already contains the cached entries.

The filter input, the project-count badge, and the `{#each}`-rendered list rows SHALL be interactive in that first paint — the user SHALL NOT have to wait for any IPC round-trip to type, scroll, or select a project that is already in the cache.

When no cache exists (first-ever launch, cache cleared, cache failed to parse), the bundle loader SHALL render in its current empty-until-chunks-arrive state with no regression.

The background `loadProjects()` refresh SHALL continue to fire — the cache is a fast path, not a replacement for refresh. As `projects-chunk` events arrive, the displayed list SHALL update via the upsert-by-slug merge defined in `lizaalert-integration`.

#### Scenario: Returning user sees the catalog instantly

- **WHEN** the user previously completed a refresh in any prior session AND opens the application again
- **THEN** the bundle loader's project column renders the cached catalog within the first paint, the filter input accepts keystrokes immediately, and any background refresh updates the list in place without blocking input

#### Scenario: Reopening the loader from the workspace is instant

- **WHEN** the user is in the workspace (`/project`) and clicks the Sidebar "Maps…" button to navigate back to the bundle loader (`/`)
- **THEN** the cached catalog is visible on the first paint of `/`, without the empty-list flicker that today's mount cycle produces

#### Scenario: First-ever launch behaves like today

- **WHEN** the application starts on a machine with no prior cache for the catalog
- **THEN** the bundle loader paints with an empty project list AND `loadProjects()` populates it via `projects-chunk` events exactly as before this change

#### Scenario: Cache hydration does not contend with the IPC refresh

- **WHEN** the cached catalog is hydrated on mount AND a `loadProjects()` refresh begins immediately afterward
- **THEN** entries from the refresh merge into the displayed list without removing any cached entry, without re-creating list rows that did not change, and without blocking filter input

### Requirement: Bundle-loader status bar omits the "Open bundle now" affordance and the ready-files list

The bundle-loader status bar SHALL NOT include an "Open bundle now" button. The bundle-loader status bar SHALL NOT include a ready-files list enumerating individual files that have finished downloading during the active bundle download. Both surfaces were workarounds for delayed per-map availability and SHALL be replaced by the Maps column reflecting per-file readiness directly (see the `map-bundles` capability).

The status bar SHALL retain its other slots from change A: status-line, current-file label, progress bar, byte counters, and action slot. The remaining action slot SHALL continue to host the `Cancel` button while a download is in flight, and SHALL be empty otherwise.

The bar's outer height SHALL shrink by the height previously reserved for the ready-files row. Within a single session, the bar's outer height SHALL still NOT jitter as progress events arrive — the stable-layout guarantee for the remaining content from change A SHALL be preserved.

#### Scenario: No "Open bundle now" button at any point in a download

- **WHEN** the user starts a bundle download AND files start completing one by one
- **THEN** no "Open bundle now" button appears in the status bar at any point — neither before, during, nor after individual files complete; the user's affordance for opening a finished map is the Maps column row itself, which has flipped to `cached`

#### Scenario: No ready-files list at any point in a download

- **WHEN** a bundle download is in progress AND files are completing
- **THEN** no list of recently-completed files is rendered in the status bar; per-file completion is communicated solely through the corresponding Maps-column row's badge transition

#### Scenario: Cancel button still shown while a download is in flight

- **WHEN** a bundle download is in flight AND `activeDownloadId` is non-null
- **THEN** the action slot in the status bar contains a single `Cancel` button; no other action button is present in that slot

#### Scenario: Stable-layout invariant preserved for remaining slots

- **WHEN** a bundle download is in progress AND `download-progress` / `bundle-progress` / `state-changed` events arrive at 5+ per second
- **THEN** the bottom edge of the status bar remains at a constant vertical position relative to the viewport for the duration of the download; toggles inside the remaining slots only change which content is visible, not the bar's outer footprint

### Requirement: Tauri event listeners are owned by the root layout, one per event type

The frontend SHALL register exactly one `listen()` subscription per Tauri event type that drives store updates (`state-changed`, `download-progress`, `bundle-progress`, `bundle-file-ready`, `projects-chunk`). All such subscriptions SHALL live in `src/routes/+layout.svelte` and SHALL write into module-level stores in `src/lib/stores.ts`. Page-level components SHALL consume those stores; they SHALL NOT re-register `listen()` for the same event types.

#### Scenario: Single listener per event type

- **WHEN** a static scan of the `src/` tree counts `listen(<EVENT_NAME>, ...)` registrations grouped by event name
- **THEN** each of `state-changed`, `download-progress`, `bundle-progress`, `bundle-file-ready`, `projects-chunk` appears exactly once

#### Scenario: Page-level state does not duplicate layout listeners

- **WHEN** the bundle-loader page (`+page.svelte`) mounts
- **THEN** it does not call `listen()` for any of the layout-owned event types; instead it reads from stores fed by the layout's listeners

### Requirement: MapView re-renders only the slice of state that changed

`MapView.svelte` SHALL subscribe to three independent slice indicators — `activeMapRef`, `tracksFingerprint`, `waypointsFingerprint` — derived from `AppState` in `src/lib/stores.ts`. Each indicator SHALL change only when its corresponding domain slice changes. `MapView` SHALL run its `applyActiveMap` / `getTracksGeojson + updateTracksLayer` / waypoint-marker reconciliation paths only when the respective indicator changes from its last applied value.

#### Scenario: Download progress does not refresh tracks

- **WHEN** a bundle download is in progress and `download-progress` events are arriving at 5+ events per second
- **THEN** the IPC command `get_tracks_geojson` is invoked at most once for the entire download (matching the count of true track-state changes), not once per progress event

#### Scenario: Adding a waypoint does not rebuild every marker

- **WHEN** the user adds a single waypoint in a project that already has 50 waypoints across two layers
- **THEN** at most one new MapLibre `Marker` instance is constructed; the existing 50 markers are not destroyed and recreated

#### Scenario: Switching the active map does not touch waypoint markers

- **WHEN** the user switches the active map within the same bundle
- **THEN** the `activeMapRef`-driven effect updates the tile source and the waypoint markers are not destroyed or recreated

### Requirement: Waypoint markers are reconciled incrementally on the map

The waypoint-rendering path SHALL maintain its internal `Map<string, Marker>` (keyed by `${layerId}:${waypointId}`) and reconcile changes by:

- creating markers only for keys present in the incoming state and absent locally,
- removing markers only for keys present locally and absent in the incoming state,
- updating the coordinates / symbol / name only for existing markers whose corresponding fields differ.

A full clear-and-recreate path MAY exist as an explicit debugging affordance but SHALL NOT run on normal state changes.

#### Scenario: Waypoint coordinate update reuses the existing marker

- **WHEN** the user drags a waypoint and the new coordinates arrive via state update
- **THEN** the marker's `setLngLat(...)` is called on the existing marker instance; no new `Marker` is constructed and the old one is not removed

#### Scenario: Waypoint deletion removes one marker

- **WHEN** the user deletes a waypoint from a layer of three waypoints
- **THEN** exactly one marker is removed and the other two markers remain unchanged

### Requirement: Bundle-loader project filter debounces input

The project filter `<input>` in `src/routes/+page.svelte` SHALL feed a debounced state with a delay of at least 120 milliseconds. The `$derived` filtered list SHALL read from the debounced state, not from the raw input value.

#### Scenario: Rapid typing produces at most one filter pass per debounce window

- **WHEN** the user types six characters into the filter input within 200 milliseconds
- **THEN** the filter pass runs no more than twice in that window — once for the trailing debounce flush and at most one intermediate

### Requirement: Startup `loadProjects()` runs exactly once per session

The application SHALL invoke `loadProjects()` exactly once per session start. The invocation SHALL be initiated from the root layout's `onMount`. The bundle-loader page SHALL NOT independently call `loadProjects()` at mount time. The user-facing refresh button SHALL remain a valid additional entry point for re-running `loadProjects()`.

#### Scenario: Cold start fires loadProjects once

- **WHEN** the application starts and the user reaches the bundle loader for the first time without clicking refresh
- **THEN** the IPC command `load_projects` is invoked exactly once

#### Scenario: User refresh fires loadProjects again

- **WHEN** the user clicks the refresh button in the bundle loader
- **THEN** `load_projects` is invoked once more, in addition to the startup invocation

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

