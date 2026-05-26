## ADDED Requirements

### Requirement: Library Maps tab rows reflect in-flight download progress per map

The Library Rail Maps tab (`src/components/library/MapsTab.svelte`) SHALL render an in-progress indicator on every map row whose package name is present in the `downloadingMaps` store. The indicator SHALL be derived from `downloadProgress.get(mapName)` so the row reflects the latest `download-progress` payload (downloaded_bytes / total_bytes or completed / total — whichever the payload provides).

When `downloadingMaps` no longer contains a map (download cancelled, completed, or never started) AND `m.downloaded` is true (per `currentProject.maps[].downloaded`), the row SHALL display the existing `cached` badge. When neither condition holds the row SHALL display its neutral state.

Switching between the three states (`in-progress` → `cached` → neutral) SHALL be driven by the existing stores fed by the layout-level listeners; the Maps tab SHALL NOT register any new `listen()` subscription.

#### Scenario: Map is currently downloading

- **WHEN** the user views the Library Maps tab AND `downloadingMaps.has(m.name)` is true AND `downloadProgress.get(m.name)` reports 47% progress
- **THEN** the row for `m.name` shows an in-progress indicator (badge, percentage, or progress bar) reflecting 47%

#### Scenario: Map transitions from downloading to cached

- **WHEN** a map's download completes AND the backend emits `bundle-file-ready` followed by `state-changed`, which causes `appState.refresh()` to flip `m.downloaded` to true AND removes the map from `downloadingMaps`
- **THEN** the row's indicator transitions from the in-progress display to the `cached` badge within one animation frame

#### Scenario: Maps tab does not duplicate event listeners

- **WHEN** the Maps tab mounts
- **THEN** the static count of `listen("download-progress", ...)` and `listen("bundle-progress", ...)` calls in the `src/` tree is unchanged from before this change (single owner remains `src/routes/+layout.svelte`)

#### Scenario: Selecting a project does not throw a JSON parse error

- **WHEN** the user clicks a project row in the Library Maps tab (or its parent Projects pane) AND the project's bundle is selected via `load_project`
- **THEN** no Sonner toast with `data-testid="ipc-error"` appears in a dev build (i.e. the IPC payload validates cleanly) AND the project becomes the current project
