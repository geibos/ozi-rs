## ADDED Requirements

### Requirement: Frontend is bootstrapped via SvelteKit with adapter-static

The frontend SHALL be bootstrapped via SvelteKit using `@sveltejs/adapter-static`. SSR SHALL be disabled (`ssr = false`) and prerender SHALL be enabled (`prerender = true`) at the root layout level. The Tauri shell SHALL load the prebuilt static output from the adapter (`build/`) as its `frontendDist`.

#### Scenario: Production build produces static output Tauri can load

- **WHEN** the developer runs `pnpm tauri build`
- **THEN** SvelteKit emits static HTML/JS/CSS into `build/`, and Tauri packages that directory as the application's frontend without requiring a Node runtime

#### Scenario: Dev workflow runs through SvelteKit

- **WHEN** the developer runs `pnpm tauri dev`
- **THEN** the Vite dev server is launched via SvelteKit's `sveltekit()` plugin, hot module replacement works for files under `src/routes/` and `src/lib/`, and the Tauri window connects to that dev server

### Requirement: Top-level surfaces live at distinct routes `/` and `/project`

The two top-level surfaces SHALL each live at a dedicated route. The bundle loader (`BundleLoaderView`) SHALL be served at `/` from `src/routes/+page.svelte`. The project workspace (MapView + Sidebar + panels) SHALL be served at `/project` from `src/routes/project/+page.svelte`. Transitions between the two SHALL be performed client-side via `onMount` plus `goto()`, because prerender precludes runtime store access in route-level `load` functions.

#### Scenario: No bundle loaded — land on the loader

- **WHEN** the application starts with no bundle loaded in the store
- **THEN** the active URL is `/` and the bundle loader surface is rendered

#### Scenario: Bundle already loaded — redirect into the workspace

- **WHEN** the application starts and the store already reports a loaded bundle (e.g. restored from session)
- **THEN** the user lands on `/` for one paint, `onMount` invokes `goto('/project')`, and the project workspace becomes the active surface

#### Scenario: User navigates back to the loader without a bundle

- **WHEN** the user closes the current project and the store reports no loaded bundle while the URL is `/project`
- **THEN** `onMount` on the project route invokes `goto('/')` and the bundle loader is shown
