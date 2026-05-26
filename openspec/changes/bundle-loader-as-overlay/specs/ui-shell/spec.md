## MODIFIED Requirements

### Requirement: Top-level surfaces live at distinct routes `/` and `/project`, and the bundle loader is additionally reachable as a Sheet inside `/project`

The two top-level surfaces SHALL each have a dedicated route. The bundle loader SHALL be served at `/` from `src/routes/+page.svelte`. The project workspace (`Sidebar` + panels) SHALL be served at `/project` from `src/routes/project/+page.svelte`. The route `/` SHALL be the cold-start surface (no active map) and SHALL render the bundle loader full-width.

Once the workspace is the active surface (`/project` with an active map present), the user SHALL access the bundle loader without leaving the workspace by opening a slide-in Sheet panel rather than by navigating away. The Sheet SHALL be the on-top-of-workspace variant of the bundle-loader UI; the `/` route SHALL NOT be used for in-workspace map switching.

The bundle-loader UI itself SHALL live in a single reusable component (`src/components/BundleLoader.svelte`) rendered identically in both surfaces: as the body of the `/` route page, and as the body of the workspace Sheet.

Transitions between `/` and `/project` SHALL continue to be performed client-side via `onMount` plus `goto()`, because prerender precludes runtime store access in route-level `load` functions.

#### Scenario: No bundle loaded — land on the loader

- **WHEN** the application starts with no active map in the store
- **THEN** the active URL is `/` and the bundle loader surface is rendered full-width

#### Scenario: Bundle already loaded — redirect into the workspace

- **WHEN** the application starts and the store already reports an active map (e.g. restored from session)
- **THEN** the user lands on `/` for one paint, `onMount` invokes `goto('/project')`, and the project workspace becomes the active surface

#### Scenario: User opens the bundle loader from the workspace

- **WHEN** the user is at `/project` with an active map AND clicks the "Maps…" button in the workspace sidebar
- **THEN** the active URL remains `/project` AND a Sheet panel slides in from the right rendering the same `BundleLoader` component used at `/`; no route navigation occurs

#### Scenario: Opening a map from the workspace Sheet closes the Sheet

- **WHEN** the user has the workspace Sheet open AND clicks a map row to activate that map
- **THEN** `openSelectedMap` is invoked, the active map in the store updates, the Sheet closes, and the user remains at `/project` with the new map rendered in the workspace

#### Scenario: Closing the Sheet preserves the workspace

- **WHEN** the user has the workspace Sheet open AND closes it via the X affordance, the Esc key, or click-outside
- **THEN** the URL remains `/project`, the workspace map remains the active surface, and no route navigation occurs

#### Scenario: Active map cleared while at `/project` falls back to the loader route

- **WHEN** the user closes the current project (or otherwise clears the active map) while the URL is `/project`
- **THEN** `onMount` on the project route invokes `goto('/')` and the bundle loader is shown as the full-width primary surface; the Sheet is not used for this case because there is no workspace to overlay

### Requirement: The frontend SHALL run as a single Tauri WebviewWindow

The application SHALL ship with exactly one `WebviewWindow` (label `main`). The previously separate `bundles` `WebviewWindow` SHALL be removed; the bundle-loader surface SHALL be reachable both as the `/` route and as a Sheet overlay inside `/project`, all inside the main window. `src-tauri/capabilities/default.json` SHALL list only `["main"]` under `windows` and SHALL NOT grant `core:window:*` or `core:webview:*` permissions beyond what `core:default` provides.

#### Scenario: Sidebar "Maps…" button opens the bundle loader within the main window

- **WHEN** the user clicks the "Maps…" button in the workspace sidebar
- **THEN** a Sheet panel slides in from the right side of the main window's viewport AND no new `WebviewWindow` is created AND the URL does not change

#### Scenario: Capabilities reflect single-window setup

- **WHEN** the project's `src-tauri/capabilities/default.json` is inspected
- **THEN** the `windows` array equals `["main"]` and no window-management or webview-creation permissions are listed

### Requirement: UI primitives are sourced from the shadcn-svelte library

The frontend SHALL provide a `src/lib/components/ui/` directory containing shadcn-svelte primitives generated via `shadcn-svelte init` and `shadcn-svelte add`. The initial primitive set SHALL include: button, dialog, popover, select, tabs, tooltip, switch, separator, scroll-area, input, label, sonner, slider, card, table, sheet. Each primitive SHALL import the `cn` helper from `$lib/utils`.

#### Scenario: A primitive renders against the active theme

- **WHEN** a shadcn-svelte `Button` is mounted anywhere in the app and the active flavour is Mocha
- **THEN** its computed styles resolve through the semantic CSS variables to Mocha-derived HSL colours, with no per-component theme wiring required

#### Scenario: `cn()` helper is available at `$lib/utils`

- **WHEN** any primitive or feature component imports `cn` from `$lib/utils`
- **THEN** the helper is defined as `twMerge(clsx(...))` and exists at that import path (locked by `components.json` from `shadcn-svelte init`)

#### Scenario: Sheet primitive is present and theme-aware

- **WHEN** the `sheet` primitive in `src/lib/components/ui/sheet/` is mounted with `side="right"`
- **THEN** the slide-in panel uses the semantic-token CSS variables for its background, foreground, and border colours, matching the active Catppuccin flavour without per-instance theme wiring

## ADDED Requirements

### Requirement: Workspace bundle-loader Sheet preserves the workspace map and overlay interactivity

The workspace SHALL expose a slide-in Sheet panel hosting the bundle-loader UI. The Sheet SHALL be opened from the workspace sidebar's "Maps…" button, SHALL slide in from the right side of the viewport, and SHALL have a fixed width of 480 pixels (`w-[480px]`).

While the Sheet is open, the workspace map (`MapView`) SHALL remain mounted, visible, and interactive — panning, zooming, and any other map gestures SHALL continue to work. The Sheet's backdrop SHALL NOT block pointer events to the underlying workspace map.

The Sheet SHALL be closable by:

1. clicking the built-in X affordance in the Sheet header,
2. pressing the Esc key while focus is anywhere inside the Sheet,
3. clicking outside the Sheet content area (the default shadcn `Sheet` close-on-outside behavior).

The Sheet's open / closed state SHALL be backed by the `bundleLoaderOpen` writable store in `src/lib/stores.ts` and SHALL NOT be encoded in the URL or in route history.

#### Scenario: Map stays interactive while Sheet is open

- **WHEN** the user has the Sheet open AND the user drags or scrolls on the visible portion of the workspace map
- **THEN** the map pans / zooms in response; the Sheet does not consume the events; no console errors are emitted

#### Scenario: MapView is not remounted when the Sheet toggles

- **WHEN** the user opens and then closes the Sheet
- **THEN** the same `MapView` instance is used throughout; no `new maplibregl.Map(...)` call is made on Sheet toggle and no re-registration of tile protocols occurs

#### Scenario: Sheet closes via Esc

- **WHEN** the Sheet is open AND the user presses the Esc key
- **THEN** the Sheet animates closed, `bundleLoaderOpen` becomes `false`, and the workspace remains the active surface

#### Scenario: Sheet closes via X affordance

- **WHEN** the Sheet is open AND the user clicks the X close button in the Sheet header
- **THEN** the Sheet animates closed, `bundleLoaderOpen` becomes `false`, and the workspace remains the active surface

#### Scenario: Sheet closes via click-outside

- **WHEN** the Sheet is open AND the user clicks anywhere on the workspace map outside the Sheet content area
- **THEN** the Sheet animates closed, `bundleLoaderOpen` becomes `false`, and the click on the map is consumed by the Sheet's close-on-outside handler (not by the map itself)

#### Scenario: Sheet survives an in-flight download being closed

- **WHEN** a bundle download is in progress AND the user closes the Sheet
- **THEN** the download continues to completion in the background AND re-opening the Sheet shows the up-to-date progress, ready files, and any new state that arrived while the Sheet was closed

### Requirement: Bundle-loader markup is extracted into a reusable component

The bundle-loader UI (project list, maps column, status region, downloads progress) SHALL be implemented as a single reusable component at `src/components/BundleLoader.svelte`. The route `/` and the workspace Sheet SHALL each render exactly this component as their body; neither SHALL inline duplicate markup. The component SHALL NOT own any routing decisions; navigation away from a successful map open SHALL be delegated to the surrounding surface (the route page redirects via `goto`, the Sheet host closes the Sheet).

#### Scenario: Both surfaces render the same component

- **WHEN** the `/` route page and the workspace Sheet are both inspected
- **THEN** each contains exactly one `<BundleLoader />` element and no duplicate copies of the project-list / maps-column / status-region markup
