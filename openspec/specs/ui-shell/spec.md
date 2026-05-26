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

The system SHALL render every in-app panel (`Console`, `ThemePicker`, `SymbolPicker`, `LibraryRail`, `MapView` wrapper) through shadcn-svelte primitives — `Card`, `ScrollArea`, `Button`, `Select`, `Popover`, `Tooltip`, `Slider`, `Switch`, `Label`, `Separator`, `Dialog`, `Table`, `Tabs`, `Sheet`, `DropdownMenu` — and SHALL style their static surfaces through Tailwind utility classes that read the semantic-token CSS variables (`bg-background`, `text-foreground`, `bg-popover`, `border-border`, `bg-card`, `bg-muted`, …).

Component-local `<style>` blocks SHALL be removed, except where a rule expresses a dynamic value sourced from the domain (e.g. a `TrackStyle.color` swatch or a waypoint symbol glyph). Such dynamic values SHALL be expressed as inline `style=` attributes, not as Tailwind classes.

`MapView.svelte` SHALL be migrated at the wrapper level only; MapLibre initialization, source/layer setup, drag handlers, click handlers, and the tile-protocol code SHALL NOT be modified by this requirement.

The `LibraryRail` Track-row color swatch and Waypoint-row symbol control SHALL retain their domain-driven inputs — a native `<input type="color">` for tracks (operating on the RGBA bytes in `TrackStyle.color`) and the existing `SymbolPicker` popover for waypoints — and SHALL NOT bind to `--ctp-*` palette variables or to semantic-token CSS variables.

Toast notifications SHALL be routed through `svelte-sonner` mounted in the root layout, rather than ad-hoc `alert()` or inline error surfaces.

The legacy floating-panel components — `Sidebar.svelte`, `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, `SimplifyPanel.svelte` — SHALL NOT exist in the codebase. Their functionality has been relocated to the `LibraryRail` (Tracks and Waypoints tabs, with Simplify as an inline popover on the Track row's actions menu) and to the Inspector pane / Cmd-K palette delivered by a separate change.

#### Scenario: Panel surfaces follow the active flavour

- **WHEN** the user switches the active Catppuccin flavour while any panel is open
- **THEN** the panel's static surfaces (background, foreground text, borders, dividers, hover states) re-render through the new flavour's semantic-token values without any `<style>`-block override

#### Scenario: Track colour swatch is independent of the theme switch

- **WHEN** the user assigns a track a specific colour (e.g. `#ff8800`) via the native `<input type="color">` inside the `LibraryRail` Tracks tab, then switches the Catppuccin flavour
- **THEN** the track's colour swatch and its MapLibre rendering remain exactly `#ff8800` across all four flavours; no `--ctp-*` or semantic-token variable rebinds the swatch

#### Scenario: All four flavours pass the visual smoke pass

- **WHEN** the maintainer captures screenshots of `LibraryRail` (with the Tracks tab active and at least one track loaded) plus `MapView` across all four Catppuccin flavours via `ozi-rs-mcp`
- **THEN** each flavour renders without missing styles, without leftover hard-coded colours that bypass semantic tokens, and without regressions to MapLibre map content

#### Scenario: MapView wrapper migrates without touching MapLibre internals

- **WHEN** the maintainer reviews the `MapView.svelte` migration commit
- **THEN** the diff modifies only the outer wrapper container's classes (Tailwind utilities) and any Tailwind-replaced wrapper styles; MapLibre `new maplibregl.Map(...)` construction, source/layer setup, drag handlers, click handlers, and the tile-protocol code SHALL be byte-identical to before the commit

#### Scenario: Legacy floating-panel files are absent

- **WHEN** the maintainer inspects `src/components/` after this change lands
- **THEN** none of `Sidebar.svelte`, `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, or `SimplifyPanel.svelte` are present AND no `import` statement anywhere in the codebase references any of them

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

### Requirement: The workspace `library-rail` slot is filled by a three-tab Library always-visible component

The `library-rail` slot of the workspace shell SHALL host a single component `src/components/LibraryRail.svelte` that renders three persistent tabs — Maps, Tracks, Waypoints — backed by the shadcn `Tabs` primitive. All three tab triggers SHALL be visible at all times whenever the workspace surface is active; the rail SHALL NOT collapse, accordion, or hide any of the three tabs. Exactly one tab's content panel SHALL be visible at a time.

The active tab SHALL be backed by a session-scoped Svelte store (`libraryActiveTab`) holding one of `'maps' | 'tracks' | 'waypoints'`. The store SHALL NOT be persisted to localStorage or to the Rust session file. Default value SHALL be `'maps'` on a fresh app launch.

#### Scenario: Library is mounted in the library-rail slot

- **WHEN** the user is at `/project` with an active map and inspects the workspace shell
- **THEN** the `library-rail` slot contains exactly one `<LibraryRail />` component, and `<LibraryRail />` renders three `Tabs.Trigger` elements labelled "Maps", "Tracks", "Waypoints"

#### Scenario: All three tab triggers are always visible

- **WHEN** the workspace is the active surface AND a project is open
- **THEN** the Maps, Tracks, and Waypoints tab triggers are all rendered and clickable, regardless of which tab is currently active, regardless of whether the project contains zero tracks or zero waypoints

#### Scenario: Active tab is session-scoped and not persisted across restarts

- **WHEN** the user opens the Tracks tab in one session AND restarts the app
- **THEN** the next session starts with the Maps tab active (default), not with the Tracks tab the previous session ended on; no localStorage or session-file entry records the choice

### Requirement: Library rows follow a unified visibility / color-or-symbol / name / actions pattern

Every row in the Tracks and Waypoints tabs SHALL be rendered through a shared component `src/components/library/LibraryRow.svelte` exposing four ordered cells:

1. **Visibility toggle** — a shadcn icon `Button` with the Lucide `Eye` or `EyeOff` icon reflecting the row's current visibility. Click SHALL toggle visibility through the existing API (`set_track_visibility` for tracks, the equivalent waypoint API for waypoints). The button SHALL carry an `aria-label` that describes the current state ("Hide track Foo" / "Show track Foo").
2. **Color swatch (tracks)** OR **Symbol button (waypoints)** — for tracks: a 16x16 circular swatch styled via inline `style="background-color: …"` reading the row's RGBA domain colour. Click SHALL open a shadcn `Popover` containing a native `<input type="color">`. For waypoints: a 16x16 button rendering the waypoint's current symbol glyph. Click SHALL open the existing `SymbolPicker` popover.
3. **Name (plus optional subline)** — the row's display name, truncated with `text-overflow: ellipsis`. The full name SHALL appear in a shadcn `Tooltip` on hover. Double-click SHALL switch the row into an inline-rename mode (`<input>` bound to a local writable; commit on Enter or blur, cancel on Esc). For Tracks rows only, a second line SHALL render in `text-xs text-muted-foreground font-mono tabular-nums` showing distance / duration / point-count separated by middle-dots (e.g. `12.4 km · 02:31 · 412 pts`).
4. **Actions menu** — a shadcn `DropdownMenu` triggered by a `⋯` icon button. The menu items SHALL be:
   - Tracks: Export GPX, Export PLT, Set line width, Simplify…, Delete.
   - Waypoints: Export WPT, Delete.

Each menu item SHALL invoke the same API function the legacy floating panel called for the equivalent operation; no operation SHALL be removed, only relocated.

Maps tab rows SHALL diverge from this pattern: no visibility toggle, no color swatch — a "cached" badge SHALL occupy the swatch position when the map's tiles are fully cached locally. The actions menu items SHALL be: Reveal in Finder, Switch to. The active map SHALL be visually highlighted (e.g. a `bg-accent` background on the row).

#### Scenario: Track row toggles visibility on the map

- **WHEN** the user clicks the visibility-toggle icon on a Track row that is currently visible
- **THEN** the row's icon switches from `Eye` to `EyeOff`, `set_track_visibility` is called with `visible=false`, and the track disappears from the MapView overlay without remounting the map

#### Scenario: Track row opens a color-picker popover

- **WHEN** the user clicks the color swatch on a Track row
- **THEN** a popover opens anchored to the swatch, containing a native `<input type="color">` initialised to the row's current colour; changing the value calls `set_track_color` with the new RGBA and the MapView re-renders the track in the new colour

#### Scenario: Track row renames inline via double-click

- **WHEN** the user double-clicks the name cell of a Track row AND types a new name AND presses Enter
- **THEN** the row enters rename mode, accepts text input, calls the rename API on Enter, and exits rename mode showing the new name; pressing Esc instead SHALL discard the edit and exit rename mode showing the original name

#### Scenario: Waypoint row symbol button opens the SymbolPicker

- **WHEN** the user clicks the symbol cell on a Waypoint row
- **THEN** the existing `SymbolPicker` popover opens anchored to the symbol cell; selecting a symbol writes through to the waypoint's domain symbol field and the MapView marker updates

#### Scenario: Map row marks the active map and offers Reveal-in-Finder

- **WHEN** the user inspects the Maps tab AND the project has an active map plus two non-active cached maps
- **THEN** the active map row has a visible highlight (e.g. `bg-accent`), the two non-active rows show a "cached" badge, and each row's `⋯` menu contains a "Reveal in Finder" item that opens the OS file browser at the map's path

### Requirement: Library tabs host the active-layer selectors in their headers

The Tracks tab's content panel SHALL render a shadcn `Select` at the top, above the row list, exposing the project's track layers and bound to the active-track-layer ID (`$activeTrackLayerId`). The Waypoints tab's content panel SHALL render the analogous shadcn `Select` bound to `$activeWaypointLayerId`. The Maps tab SHALL NOT contain a layer selector — its header SHALL contain only the "Maps…" button (see the workspace-bundle-loader-Sheet requirement).

The selectors SHALL change the active layer via the existing `set_active_track_layer` / `set_active_waypoint_layer` API calls. The selectors SHALL NOT modify, hide, or unload any layer; they SHALL only retarget where new edits land, consistent with the `layers` capability's non-destructive-selection invariant.

Clicking a row in the Tracks tab whose owning layer differs from the current active track layer SHALL trigger the same `set_active_track_layer` call before applying the row selection. The same SHALL hold for Waypoint rows and `set_active_waypoint_layer`. This row-activation behavior keeps the active-layer flag aligned with the row the user has just engaged with.

#### Scenario: Tracks tab header shows the active track layer selector

- **WHEN** the user opens the Tracks tab AND the project has two track layers "A" and "B" with "A" active
- **THEN** the top of the Tracks tab content shows a `Select` reading "A" as its current value, and opening the dropdown lists both "A" and "B"; choosing "B" calls `set_active_track_layer` with B's ID

#### Scenario: Clicking a row from a different layer switches the active layer

- **WHEN** layer "A" is the active track layer AND the user clicks a row in the Tracks tab whose owning layer is "B"
- **THEN** the system calls `set_active_track_layer` with B's ID before applying the row selection AND the Tracks tab's active-layer Select updates its trigger label to "B"

#### Scenario: Switching active layer keeps overlays visible

- **WHEN** the user changes the active track layer via the tab-header `Select` from "A" to "B"
- **THEN** both layers' track overlays remain rendered on the map; the change is purely a routing change for subsequent edits, consistent with the `layers` non-destructive-selection invariant

### Requirement: Project lifecycle and mode toggles do not live in the Library

The `LibraryRail` and its three tabs SHALL NOT render any of the following:

- Open project, Save project, Undo, Redo (project lifecycle).
- Create Track / Add Waypoint mode toggles, or any other mode toggle that mutates map-click interpretation.
- Console toggle, Theme picker, or other application chrome.

These affordances belong on the top context-bar and / or in the Cmd-K palette, which are owned by a separate change. The Library is for **objects** (maps, tracks, waypoints), not for project-lifecycle verbs, modes, or chrome.

Underlying state stores driving the modes (`drawingModeActive`, `addWaypointMode`, and their companions) SHALL NOT be modified by this change. Removing the buttons from the legacy `Sidebar` SHALL NOT change the stores' semantics; keyboard shortcuts wired to toggle these modes (where any exist) continue to work.

The four deprecated panel-open writable stores — `tracksPanelOpen`, `waypointsPanelOpen`, `trackPointsPanelOpen`, `simplifyPanelOpen` — SHALL be removed from `src/lib/stores.ts`. The Library tabs are always visible; these stores have no analogue in the new Library.

#### Scenario: Library tabs contain no project-lifecycle buttons

- **WHEN** the maintainer inspects every `Tabs.Content` panel in `LibraryRail.svelte` and its sub-components
- **THEN** none of them render a button labelled or icon-tagged Open, Save, Undo, or Redo; none of them render a Console toggle or a Theme picker

#### Scenario: Library tabs contain no mode-toggle buttons

- **WHEN** the maintainer inspects every `Tabs.Content` panel in `LibraryRail.svelte` and its sub-components
- **THEN** none of them render a "Create Track" toggle or an "Add Waypoint" toggle; these affordances exist only on the top context-bar (added by a separate change)

#### Scenario: Deprecated panel-open stores are removed

- **WHEN** the maintainer greps for `tracksPanelOpen`, `waypointsPanelOpen`, `trackPointsPanelOpen`, and `simplifyPanelOpen` across `src/`
- **THEN** zero matches are found; the stores have been removed from `src/lib/stores.ts` and no consumer remains

### Requirement: Simplify is invoked as an inline popover from the Track row, not as a floating panel

The Simplify operation SHALL be invoked from a Track row's `⋯` actions menu via a "Simplify…" item. Selecting the item SHALL open a shadcn `Popover` anchored to the row. The popover SHALL contain the algorithm selector, tolerance slider, preview button, commit button, and cancel button that the legacy `SimplifyPanel.svelte` exposed. The popover SHALL operate on the row's track ID — there SHALL be no ambiguity about "which track is being simplified".

The popover SHALL be dismissable via the Esc key and via click-outside, consistent with the shadcn `Popover` defaults.

`SimplifyPanel.svelte` SHALL NOT exist in the codebase after this change.

#### Scenario: Simplify opens anchored to a Track row

- **WHEN** the user opens the `⋯` menu on a Track row AND clicks "Simplify…"
- **THEN** a popover opens anchored to that row containing the algorithm selector, tolerance slider, preview / commit / cancel controls; the popover operates on the same track ID as the row

#### Scenario: Simplify popover dismisses via Esc

- **WHEN** the Simplify popover is open AND the user presses Esc
- **THEN** the popover closes without applying any simplification; the track's points are unchanged

#### Scenario: SimplifyPanel.svelte is absent

- **WHEN** the maintainer inspects `src/components/`
- **THEN** `SimplifyPanel.svelte` is not present and no `import` statement references it

### Requirement: TrackPointsPanel data helpers are preserved in a standalone module

The data-loading helpers that `TrackPointsPanel.svelte` exposed — functions that fetch track point arrays, format coordinates, format timestamps, compute per-segment speeds, and paginate the points list — SHALL be extracted to a new module `src/lib/track-points.ts` before `TrackPointsPanel.svelte` is deleted. The module SHALL be a pure-TypeScript utility: no Svelte runes, no store subscriptions, no DOM access.

`TrackPointsPanel.svelte` itself SHALL NOT exist in the codebase after this change. The track-points UI lives in the Inspector pane delivered by a separate change, which SHALL import from `src/lib/track-points.ts`.

#### Scenario: track-points module exists and exports the helpers

- **WHEN** the maintainer inspects `src/lib/track-points.ts`
- **THEN** the module exports the data-loading and formatting helpers previously co-located in `TrackPointsPanel.svelte`, with no Svelte-specific or DOM-specific code in the module

#### Scenario: TrackPointsPanel.svelte is absent

- **WHEN** the maintainer inspects `src/components/`
- **THEN** `TrackPointsPanel.svelte` is not present and no `import` statement references it

### Requirement: The workspace `inspector-rail` slot hosts a context-sensitive Inspector that is collapsed by default and slides in on Library selection

The right-side `inspector-rail` slot of the workspace shell (introduced by `redesign-shell-layout`) SHALL be filled by a single component `src/components/InspectorRail.svelte`. The rail SHALL be mounted unconditionally for the lifetime of the workspace surface; only its body content SHALL appear and disappear in response to selection.

In its default state (no Library selection), the rail SHALL render only an edge affordance (a thin pinnable handle on the right edge of the viewport) and SHALL occupy zero horizontal width inside the centre pane (i.e. the `MapView` extends to the right edge of the workspace minus the edge affordance's narrow strip).

When the Library selection store transitions from null to a non-null value of any of the three supported types (Track, Waypoint, Map), the rail SHALL slide in from the right edge over 240ms, fade its body content in, and render the appropriate Inspector subcomponent for the selected type. The slide-in animation SHALL respect the motion-intensity-6 tokens established by `redesign-shell-layout`.

When the selection transitions between two non-null values (regardless of whether the types match), the rail SHALL remain open and SHALL swap its body subcomponent via the spring transition (stiffness 100, damping 20) established by `redesign-shell-layout`; the rail itself SHALL NOT re-run its slide-in animation on such transitions.

When the selection transitions back to null AND the rail is not pinned, the rail SHALL slide its body out and return to the collapsed default. When the rail is pinned (the user has clicked the pin affordance on the edge handle), the rail SHALL remain open with an empty body in this case.

The rail's fixed expanded width SHALL be ~360px, matching the slot dimension reserved by `redesign-shell-layout`. It SHALL NOT be user-resizable in v1.

#### Scenario: Cold workspace — Inspector starts collapsed

- **WHEN** the user opens a project workspace AND no Library row is selected
- **THEN** the `inspector-rail` slot renders only the edge affordance, the `MapView` extends to the workspace's right edge minus the edge strip, and no Inspector body is mounted

#### Scenario: Selecting a Track row opens Track Inspector

- **WHEN** the user clicks a Track row in the Library AND `$selectedTrack` transitions from null to a track
- **THEN** the rail slides in over 240ms from the right edge, the Track Inspector subcomponent mounts inside the rail, and the rail's body fades in

#### Scenario: Switching from a Track to a Waypoint swaps Inspector body without re-animating the rail

- **WHEN** the rail is open showing a Track Inspector AND the user clicks a Waypoint row in the Library
- **THEN** the rail itself remains expanded with no slide animation re-fire; the rail's body subcomponent swaps from Track Inspector to Waypoint Inspector via the spring transition

#### Scenario: Clearing selection collapses the rail

- **WHEN** the rail is open AND the Library selection transitions back to null (no row highlighted) AND the rail is not pinned
- **THEN** the rail slides its body out and returns to the collapsed default state with only the edge affordance visible

#### Scenario: Pinning the rail keeps it open across selection changes

- **WHEN** the user clicks the pin affordance on the rail's edge handle AND subsequently clears the Library selection
- **THEN** the rail remains expanded at full width with its body rendering an empty state, ready for the next selection to populate it without re-running the slide-in animation

### Requirement: Track Inspector contains a stats card, an inline segments / points table, an elevation-chart placeholder, and an actions row

When the Inspector rail renders the Track Inspector subcomponent, the subcomponent SHALL lay out the following sections in order:

1. **Header** — track name (read-only display, with a colour swatch matching the track's `TrackStyle.color`) and a visibility toggle.
2. **Stats card** — distance, duration, point count, start time, all rendered with mono-spaced numerals on a rounded-`[1.5rem]` card surface.
3. **Segments / points table** — the table SHALL be implemented as a child component `src/components/inspector/TrackSegmentsTable.svelte` that ports the logic of the retired `TrackPointsPanel` (parked by `redesign-library-sidebar`). The table SHALL subscribe to the exact same stores the legacy panel did: `$selectedPointId`, `$activeTrackLayerId`, and `$editModeActive`. Bidirectional highlight between the map and the table SHALL be preserved. The edit-mode toggle SHALL stay on the `$editModeActive` store and SHALL NOT introduce a new store.
4. **Elevation chart placeholder slot** — a labeled empty container reserving space for a future elevation chart. It SHALL display a static message ("Elevation chart — coming in a follow-up change") and SHALL NOT call any new IPC or charting library in this change.
5. **Actions row** — Export GPX, Export PLT, Set line width, Simplify, Delete. Each action SHALL dispatch through the existing `ProjectCommand`-shaped endpoints in `src/lib/api.ts` (`exportTrackGpx`, `exportTrackPlt`, `updateTrackStyle`, `simplifyTrack`, `deleteTrack`).

#### Scenario: Selecting a track populates the stats card

- **WHEN** the user clicks a Track row in the Library
- **THEN** the Track Inspector mounts AND the stats card renders distance, duration, point count, and start time from `getTrackDetail` for the selected track

#### Scenario: Map point click highlights the corresponding table row

- **WHEN** Track Inspector is open with the segments table visible AND the user clicks a point on the map that belongs to the selected track
- **THEN** the corresponding row in the segments table is highlighted (matching the behaviour of the retired `TrackPointsPanel`)

#### Scenario: Table row click highlights the corresponding map point

- **WHEN** Track Inspector is open with the segments table visible AND the user clicks a row in the segments table
- **THEN** the corresponding point on the map is highlighted and the map view scrolls / centres the point as the retired `TrackPointsPanel` did

#### Scenario: Edit mode toggle uses the existing `$editModeActive` store

- **WHEN** the user toggles the edit-mode switch in the Track Inspector's segments table header
- **THEN** the `$editModeActive` store flips AND the table's per-row edit affordances appear / disappear (same UI semantics as the retired panel)

#### Scenario: Elevation chart placeholder renders without loading data

- **WHEN** Track Inspector mounts AND the segments table has loaded
- **THEN** an elevation-chart placeholder is visible below the table AND no IPC call to fetch elevation data is made AND no charting library is loaded

#### Scenario: Track actions dispatch through `ProjectCommand` endpoints

- **WHEN** the user clicks any action in the Track Inspector's actions row (Export GPX, Export PLT, Set line width, Simplify, Delete)
- **THEN** the corresponding `src/lib/api.ts` endpoint is invoked AND the call resolves through the existing `ProjectCommand` pipeline (no direct store mutation, no bypass of the command bus)

### Requirement: Waypoint Inspector edits inline (name, symbol, lat/lng readout, visibility) without a dialog

When the Inspector rail renders the Waypoint Inspector subcomponent, the subcomponent SHALL provide inline editing for the selected waypoint. There SHALL be no separate modal dialog or popout form.

The subcomponent SHALL lay out:

1. **Header** — waypoint name as an editable text input AND a symbol picker (re-using the existing `SymbolPicker` component) AND a visibility toggle.
2. **Location card** — lat / lng readout (read-only display) AND a "Move on map" action that delegates to the existing map-driven move flow.
3. **Actions row** — Export WPT, Delete.

All edits SHALL dispatch through the existing `ProjectCommand`-shaped endpoints in `src/lib/api.ts` (e.g. `updateWaypoint`, `deleteWaypoint`). No direct store mutation. No bypass of the command bus.

#### Scenario: Renaming a waypoint inline persists through ProjectCommand

- **WHEN** the user types a new name in the Waypoint Inspector header AND blurs / commits the input
- **THEN** `updateWaypoint` is invoked through `src/lib/api.ts` AND the waypoint's new name is persisted via the `ProjectCommand` pipeline (no separate dialog mounts at any point)

#### Scenario: Changing the symbol inline

- **WHEN** the user picks a symbol from the inline `SymbolPicker` in the Waypoint Inspector
- **THEN** `updateWaypoint` is invoked with the new symbol AND the change is reflected on the map without a dialog opening or closing

### Requirement: Map Inspector is read-only and shows calibration metadata plus a Reveal in Finder action

When the Inspector rail renders the Map Inspector subcomponent, the subcomponent SHALL display the active map's calibration metadata read-only (CRS, bounds, resolution — sourced via the existing `getOziMetadata` endpoint in `src/lib/api.ts`) and SHALL expose a "Reveal in Finder" action that opens the map file's directory in the host OS file manager via existing Tauri shell primitives.

The Map Inspector SHALL NOT expose any write affordances for calibration data in this change.

#### Scenario: Selecting a map shows its calibration

- **WHEN** the user opens Map info for the active map from the Library Maps tab
- **THEN** the Map Inspector renders inside the rail showing CRS, bounds, and resolution from `getOziMetadata`, all as read-only displays

#### Scenario: Reveal in Finder opens the host file manager

- **WHEN** the user clicks the "Reveal in Finder" action in the Map Inspector
- **THEN** the host OS file manager opens to the active map file's parent directory (no new Tauri command is added; existing shell primitives are used)

### Requirement: A global Cmd-K command palette is available everywhere with grouped results, primary actions on Enter, and optional secondary actions via keyboard chords

A single instance of `src/components/CommandPalette.svelte` SHALL be mounted in `src/routes/+layout.svelte`, sibling to `MapView`. Its open / closed state SHALL be backed by a writable store `commandPaletteOpen` in `src/lib/stores.ts`.

The palette SHALL be opened by the keyboard chord `⌘K` (macOS) or `Ctrl+K` (Windows / Linux), registered as a `window`-level `keydown` listener at the layout level. The chord SHALL fire from any focus state in the application — including inside form inputs — except when focus is already inside the palette itself.

The palette SHALL also be openable via a button in the top context-bar of the workspace shell.

The palette SHALL close on:

1. pressing `Esc` while focus is anywhere inside the palette,
2. clicking outside the palette content area,
3. successful resolution of any primary or secondary action.

The palette SHALL render a single search input at the top (auto-focused on open) and a results list below. The results list SHALL be grouped, in the following fixed order:

1. **Open map** — maps in the active project (sourced from existing project / map stores).
2. **Switch project** — projects in the LizaAlert catalog (sourced from the catalog cache established by `cache-project-catalog-locally`).
3. **Find track / waypoint** — tracks and waypoints in the active project, merged into one group; result row icon distinguishes the type.
4. **Project actions** — Open, Save, Undo, Redo.
5. **Settings** — theme, units (placeholder), GPS (placeholder).
6. **Recent files** — sourced from the localStorage entry `ozi:recent-files:v1`.

Each result row SHALL share the row pattern used by the Library (icon column + label + optional meta line). Groups with zero matching results after the current search input SHALL be hidden entirely (no empty headers). If all groups are empty, the palette SHALL show a single "No matches" state.

Each result SHALL have a primary action that runs on `Enter` while the result is highlighted. A result MAY additionally have a secondary action exposed via a keyboard chord:

- `⌘E` — secondary action for track and waypoint results (Export).
- `⌘R` — secondary action for map results (Reveal in Finder).

Arrow keys SHALL move the highlight up and down through the visible results. `Tab` SHALL jump to the first result of the next group.

The palette SHALL adapt to context: when the user is at the cold-start surface (no active project), the groups that depend on active-project state (Open map, Find track / waypoint, Project actions) SHALL be hidden. Only Switch project, Settings, and Recent files SHALL render in that context.

#### Scenario: Cmd-K opens the palette from any focus state

- **WHEN** the user presses `⌘K` (macOS) or `Ctrl+K` (Windows / Linux) AND focus is anywhere in the application except inside the palette itself
- **THEN** the palette opens with its search input focused, and the previously-focused element does not receive the keystroke as text input

#### Scenario: Search filters across all groups

- **WHEN** the palette is open AND the user types a query in the search input
- **THEN** each group filters its results against the query AND empty groups disappear entirely AND the highlight defaults to the first visible result

#### Scenario: Enter runs primary action

- **WHEN** the palette is open AND a result is highlighted AND the user presses `Enter`
- **THEN** the result's primary action runs (open map / switch project / focus track / run project action / open theme picker / open recent file) AND the palette closes after the action resolves

#### Scenario: Cmd-E exports the highlighted track or waypoint

- **WHEN** the palette is open AND a track or waypoint result is highlighted AND the user presses `⌘E` (`Ctrl+E` on Windows / Linux)
- **THEN** the secondary "Export" action runs for that result AND the palette closes after the action resolves

#### Scenario: Cmd-R reveals the highlighted map file

- **WHEN** the palette is open AND a map result is highlighted AND the user presses `⌘R` (`Ctrl+R` on Windows / Linux)
- **THEN** the secondary "Reveal in Finder" action runs for that result AND the palette closes after the action resolves

#### Scenario: Esc closes the palette

- **WHEN** the palette is open AND the user presses `Esc`
- **THEN** the palette closes AND focus returns to the element that was focused before the palette opened

#### Scenario: Click-outside closes the palette

- **WHEN** the palette is open AND the user clicks outside the palette content area
- **THEN** the palette closes AND no action runs

#### Scenario: Cold-start surface hides project-dependent groups

- **WHEN** the user is at the cold-start surface (no active project) AND presses `⌘K`
- **THEN** the palette opens AND shows only Switch project, Settings, and Recent files groups; the Open map, Find track / waypoint, and Project actions groups are not rendered

### Requirement: Recent files are persisted in localStorage under a versioned key

The system SHALL provide a helper `src/lib/recentFiles.ts` that maintains a localStorage entry keyed `ozi:recent-files:v1`. The entry's value SHALL be a JSON array of records `{ projectSlug: string, mapPath: string, mapName: string, openedAt: number }`, capped at the most recent 8 records.

On every successful resolution of `openSelectedMap`, the helper SHALL append a new record, deduplicating by `mapPath` (an existing entry with the same path SHALL move to the front of the list rather than appearing twice). When the cap is exceeded, the oldest record SHALL be dropped.

The helper SHALL handle the following failure modes silently (logged via `console.warn`, no user-facing error):

- localStorage quota exceeded — drop the oldest record, retry once; if still failing, leave the list unchanged.
- Stored JSON unparseable — treat as empty list, overwrite on next successful write.

The Cmd-K palette's Recent files group SHALL read from this helper. The helper's key SHALL be versioned (`:v1`) so that any future schema change writes to a new key (`:v2`) rather than silently invalidating existing data.

The system SHALL NOT store recent-files data in the Rust session file. Recent files are a UX convenience, per-machine, and SHALL NOT be conflated with domain state.

#### Scenario: Opening a map appends to recent files

- **WHEN** the user opens a map via any path (bundle-loader sheet, Library, or palette) AND `openSelectedMap` resolves successfully
- **THEN** a record `{ projectSlug, mapPath, mapName, openedAt }` is appended to `ozi:recent-files:v1` AND if a record with the same `mapPath` already existed, it is moved to the front rather than duplicated

#### Scenario: Recent files survive restart

- **WHEN** the user opens at least one map in a session AND restarts the application
- **THEN** the palette's Recent files group renders the previously-opened maps in most-recent-first order from `ozi:recent-files:v1`

#### Scenario: Cap is enforced at 8 records

- **WHEN** the user opens a 9th distinct map
- **THEN** `ozi:recent-files:v1` contains exactly 8 records AND the oldest is no longer present

#### Scenario: Unparseable storage falls back to empty

- **WHEN** the localStorage entry `ozi:recent-files:v1` contains a non-JSON or schema-invalid value
- **THEN** the helper treats the recent files list as empty AND overwrites the entry on the next successful map open AND a `console.warn` is emitted

#### Scenario: Recent files do not enter the session file

- **WHEN** the Rust session file (per `project-persistence`) is inspected after any sequence of map opens
- **THEN** the file contains no recent-files data; that information lives exclusively in localStorage

### Requirement: Dev builds expose IPC failures via a structured error toast

In development builds (`import.meta.env.DEV === true`), every IPC call SHALL surface its rejection through a Sonner toast whose root element carries `data-testid="ipc-error"`. The toast SHALL display the IPC command name as its title and the full, JSON-stringified error payload as its description. The toast SHALL be sticky (no auto-dismiss) so the user can read the payload before it disappears.

In production builds the dev toast SHALL NOT render; existing user-facing `toast.error(...)` call sites at IPC call sites continue to render their friendly messages unchanged in both dev and production.

The dev toast SHALL be additive — it stacks alongside any user-facing toast a call site already emits.

#### Scenario: An IPC call rejects in a dev build

- **WHEN** the frontend invokes any Tauri command AND the backend rejects (e.g. payload-shape mismatch, missing field) AND `import.meta.env.DEV` is true
- **THEN** a Sonner toast appears whose root carries `data-testid="ipc-error"` AND whose title equals the IPC command name AND whose description equals the JSON-stringified error payload

#### Scenario: A production build hides the dev toast

- **WHEN** the same IPC rejection occurs in a production build (`import.meta.env.DEV` is false)
- **THEN** no `data-testid="ipc-error"` toast is rendered AND the existing user-facing `toast.error(...)` (if any at the call site) renders unchanged

#### Scenario: The error payload itself is unstringifiable

- **WHEN** an IPC rejection's error object throws when passed to `JSON.stringify` (circular reference, `BigInt`, etc.)
- **THEN** the dev toast description falls back to `String(error)` AND the toast still carries `data-testid="ipc-error"` AND no second uncaught exception is thrown

### Requirement: Workspace status bar reflects in-flight bundle download progress

The workspace status bar element (`data-testid="workspace-status-bar"` in `src/components/WorkspaceShell.svelte`) SHALL render the active bundle's progress text whenever `bundleProgress` in `src/lib/stores.ts` is non-null. The text SHALL include the phase (`scanning` / `downloading` / `extracting` / `indexing`) and a numeric progress hint (completed / total OR downloaded_bytes / total_bytes, whichever is available on the payload), mirroring the text rendered by `BundleLoader.svelte` for the same payload.

When `bundleProgress` is null the status bar SHALL render its idle content (today: empty).

#### Scenario: Download in progress

- **WHEN** a bundle download is in flight AND the backend emits `bundle-progress` with phase `downloading` and `completed: 3, total: 12`
- **THEN** the workspace status bar shows a line containing "downloading" and the "3 / 12" (or equivalent) progress hint within one animation frame of the event arriving

#### Scenario: Status bar clears when the download completes

- **WHEN** the download completes AND the backend emits a final `bundle-progress` or `state-changed` event that resets `bundleProgress` to null
- **THEN** the status bar returns to its idle content with no leftover progress text

#### Scenario: Status bar is shared with the bundle loader, not duplicated

- **WHEN** the bundle loader Sheet is open AND a download is in progress
- **THEN** both the status bar and the loader's own progress region show the same `bundleProgress` payload sourced from the same store; no second listener is registered

### Requirement: "Maps…" trigger ignores re-entry while a refresh is in flight

The "Maps…" button in the Library Rail Maps tab (and any other affordance that triggers `load_projects` from the workspace) SHALL be inert while `busy` in `src/lib/stores.ts` is true. The button SHALL be visibly disabled in that state so the user observes the click had no effect. A second click during an in-flight `load_projects` SHALL NOT send a second IPC.

#### Scenario: Double-click during refresh is a no-op

- **WHEN** the user clicks "Maps…" AND a `load_projects` IPC is already in flight (`$busy === true`) AND the user clicks "Maps…" a second time within 5 seconds
- **THEN** exactly one `load_projects` IPC is sent AND the UI does not freeze AND the button is visibly disabled during the freeze window

#### Scenario: Button re-enables after refresh completes

- **WHEN** the in-flight `load_projects` resolves and `busy` transitions back to false
- **THEN** the "Maps…" button becomes interactive again on the next render

### Requirement: Cmd-K trigger is visually a button, not a text input

The Cmd-K trigger in `src/components/WorkspaceShell.svelte` SHALL be implemented as a `<button type="button">` with the following discipline:

- It SHALL NOT contain or render as an `<input>` element.
- It SHALL NOT use a blinking text caret (`cursor: text` is forbidden on the trigger and its children).
- It SHALL NOT use a focus-ring style that visually replicates input focus (no inset border, no input-like outline).
- It SHALL NOT use placeholder-style copy as its label (e.g. "Search…" with the trailing ellipsis suggesting an input awaiting text). Acceptable labels include "Open command palette", "Command palette", an icon-only affordance, or the `⌘K` chord alone.
- The `⌘K` chord SHALL be rendered as a `<kbd>` element styled as a key-cap badge, visually heavier than text placeholder copy.
- On hover / focus the trigger SHALL use button-style affordances (background tint, ring) matching other buttons in the shell.

Activating the trigger (click, Enter, Space) SHALL open the command palette dialog, whose internal `cmdk` `Command.Input` takes focus automatically.

#### Scenario: Trigger does not look like an input

- **WHEN** the user inspects the workspace context bar in any theme
- **THEN** the Cmd-K trigger does not render a blinking caret AND its computed `cursor` is not `text` AND its label is not "Search…" or any other placeholder-style copy AND its focus ring matches the shell's button-focus style, not an input-focus style

#### Scenario: Trigger opens the palette

- **WHEN** the user clicks the Cmd-K trigger (or presses Enter / Space while it is focused)
- **THEN** the command palette dialog opens AND focus moves into the dialog's internal `Command.Input` AND typing characters there filters the palette results

