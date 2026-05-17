## ADDED Requirements

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
