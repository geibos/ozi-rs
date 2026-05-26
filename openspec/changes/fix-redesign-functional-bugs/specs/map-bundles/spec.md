## ADDED Requirements

### Requirement: Bundle download progress is observable from every surface that lists the downloading project's maps

While a bundle download is in flight, every UI surface that lists the maps belonging to the downloading project SHALL render an in-progress indicator on the affected rows, sourced from the `downloadingMaps` and `downloadProgress` stores in `src/lib/stores.ts`. This includes at minimum: the `BundleLoader` component's maps column (existing behaviour, unchanged) AND the Library Rail Maps tab (`src/components/library/MapsTab.svelte`, newly wired by this change).

The progress data SHALL be sourced from the existing layout-level listeners on `download-progress` and `bundle-progress` events (`src/routes/+layout.svelte`). No surface SHALL register its own `listen()` for either event type.

#### Scenario: Library Maps tab mirrors BundleLoader progress

- **WHEN** a bundle download is in progress for the current project AND the user has the Library Maps tab visible AND opens the bundle loader Sheet side-by-side
- **THEN** each map row in the Library Maps tab shows the same in-progress indicator as the corresponding row inside the bundle loader's maps column, both reflecting the same `downloadProgress` payload within one animation frame of each event arrival

#### Scenario: Status bar mirrors bundle-loader progress text

- **WHEN** a bundle download is in progress
- **THEN** the workspace status bar shows the same `bundleProgress` text that the bundle loader's status region shows, both sourced from the same `bundleProgress` store

#### Scenario: Per-map progress disappears when the map is cached

- **WHEN** a single file inside the bundle completes AND the backend emits `bundle-file-ready` AND `state-changed` AND `appState.refresh()` flips `currentProject.maps[i].downloaded` to true for that map
- **THEN** the in-progress indicator on the affected row is replaced by the `cached` badge AND the row no longer reads from `downloadProgress`

#### Scenario: Progress wiring does not duplicate event subscriptions

- **WHEN** the static count of `listen("download-progress", ...)`, `listen("bundle-progress", ...)`, and `listen("bundle-file-ready", ...)` registrations in the `src/` tree is taken
- **THEN** each event name appears exactly once across the entire frontend, in `src/routes/+layout.svelte`
