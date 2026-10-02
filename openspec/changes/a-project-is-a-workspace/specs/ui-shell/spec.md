## MODIFIED Requirements

### Requirement: Top-level surfaces live at distinct routes `/` and `/project`

The two top-level surfaces SHALL each live at a dedicated route. The bundle loader (`BundleLoaderView`) SHALL be served at `/` from `src/routes/+page.svelte`. The project workspace (`Sidebar` + panels) SHALL be served at `/project` from `src/routes/project/+page.svelte`. Transitions between the two SHALL be performed client-side via `onMount` plus `goto()`, because prerender precludes runtime store access in route-level `load` functions.

Both routes SHALL decide by the same rule: the workspace is worth opening when a map is active or a project file is open. The workspace route SHALL decide only once the application state has arrived, not on the empty state before it.

#### Scenario: No bundle loaded — land on the loader

- **WHEN** the application starts with no active map and no project file in the store
- **THEN** the active URL is `/` and the bundle loader surface is rendered

#### Scenario: Bundle already loaded — redirect into the workspace

- **WHEN** the application starts and the store already reports an active map (e.g. restored from session)
- **THEN** the user lands on `/` for one paint, `onMount` invokes `goto('/project')`, and the project workspace becomes the active surface

#### Scenario: A project restored without a map

- **WHEN** the application starts and the store reports a project file but no active map
- **THEN** the project workspace becomes the active surface, not the loader

#### Scenario: User navigates back to the loader without an active map

- **WHEN** the user closes the current project and the store reports neither an active map nor a project file while the URL is `/project`
- **THEN** the project route invokes `goto('/')` and the bundle loader is shown
