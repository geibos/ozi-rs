# project-persistence Specification

## Purpose
Covers how a SAR project and the bounded app session reach disk and come back: the `.ozp` JSON project file (its shape, atomic writes, load-time normalization), the app session file that remembers only the last project path and the active map, and the startup restore flow including its degraded states when referenced files are missing. Map data itself is never part of a project file; see `map-bundles`.

### Decision history

- ADR-0004 (2026-03-29, accepted): persist projects as pretty-printed JSON `.ozp` files via `serde_json`, with transparent ID newtypes and `default`/`skip_serializing_if` on optional fields; rationale: human-readable, debuggable without tooling, round-trip stable, no custom parser to maintain. Codified as: Project is persisted as a JSON `.ozp` file; `.ozp` content is pretty-printed JSON with a stable top-level shape; Project load normalizes missing default layers. The ADR's follow-up `version` field was never added — the format has no schema-version field today (tracked separately as CJ-8). Reality note: the project open dialog (`src/components/CommandPalette.svelte:166`) filters on the `json` extension, not `ozp`; the Rust layer accepts any path.
- ADR-0002 (2026-03-29, accepted): a project is a standalone file that references its active map and does not own the bundle directory; rationale: switching maps must not lose tracks, and bundles of tens of gigabytes must be shared across projects. Codified as: Project model is independent from map bundle data; Missing referenced files degrade to a non-panicking state (the ADR's open follow-up on vanished bundle paths). The ADR's "eframe persistent storage" for the active-map reference is superseded by the JSON session file (`PersistedAppSession` in `src-tauri/src/infrastructure/persistence.rs`).
- Code, no ADR (2026-07, merged with the m0-data-loss fixes): every project and session write goes through temp file + fsync + rename so a failed save never truncates the existing file; rationale: a failed in-place write previously destroyed the only copy of the project. Codified as: Project and session files are written atomically.
- Legacy doc `docs/persistence-session.md` (2026-04): session restore is bounded to the last project path and the active map reference; rationale: predictable startup without stale UI state. Codified as: Startup session restore is bounded to last project and active map; Specific UI and history state is intentionally NOT restored; Missing referenced files degrade to a non-panicking state; Session restore registers the active map layer so the workspace lands at calibrated bounds. Not codified: the session-file location (`app_data_dir()/session.json`, with a macOS legacy-path fallback — `src-tauri/src/lib.rs`, `resolve_session_path`) because it is not an ADR decision; the doc itself is stale (it names `default_app_session_path` and `new_with_session_path`, which no longer exist).
## Requirements
### Requirement: Project is persisted as a JSON `.ozp` file

The system SHALL serialize a project — including its layer composition, tracks, waypoints, and per-track style — as JSON to a user-chosen `.ozp` file via a Save action, and SHALL deserialize the same format via a Load action.

#### Scenario: Save then load round-trip

- **WHEN** the user saves a project to `mission.ozp` and subsequently loads `mission.ozp`
- **THEN** the loaded project contains the same tracks, waypoints, layer composition, and per-track style as at save time

### Requirement: Project model is independent from map bundle data

The system SHALL NOT embed map tile data, raster pixel data, or bundle directory contents in `.ozp` files. A project SHALL reference its active map by path or identifier only, so one bundle MAY be shared across multiple projects.

#### Scenario: Sharing a bundle across projects

- **WHEN** two projects reference the same bundle on disk
- **THEN** both projects can be opened with the same bundle without copying or duplicating bundle files

### Requirement: Startup session restore is bounded to last project and active map

The system SHALL persist between sessions only the path of the last opened project and the active map reference/path. On startup it SHALL restore those two items and nothing else.

#### Scenario: Successful restore

- **WHEN** the application is restarted after closing with a project open
- **THEN** the previously opened project file and previously active map are reloaded and become the current project and active map

### Requirement: Specific UI and history state is intentionally NOT restored

The system SHALL NOT restore, between sessions, the map viewport, selected entities, panel visibility, undo/redo history, bundle-loader window state, or unsaved edits. Theme is excluded from the Rust session file but MAY be persisted via the theme's own localStorage (see `ui-shell`).

#### Scenario: Viewport reset on restart

- **WHEN** the user pans/zooms the map, closes the app, and reopens it
- **THEN** the map opens at the default viewport for the active map, not at the user's last viewport

#### Scenario: Undo history cleared on restart

- **WHEN** the user performs several edits, closes the app, and reopens it
- **THEN** the undo/redo stack is empty even though the persisted project content reflects the edits

### Requirement: Missing referenced files degrade to a non-panicking state

The system SHALL handle the case where the persisted project file or active map reference no longer exists on disk by starting in a degraded state (fresh project, or restored project with a warning) and SHALL NOT panic.

#### Scenario: Last project file deleted

- **WHEN** the application starts and the persisted last-project path no longer exists
- **THEN** the application starts in a fresh-state mode and surfaces a non-blocking warning explaining the missing file

#### Scenario: Last active map missing

- **WHEN** the application restores the last project but the persisted active map file is missing
- **THEN** the project loads with no active map and a non-blocking warning is surfaced

### Requirement: Project load normalizes missing default layers

The system SHALL normalize a loaded `Project` so that it satisfies the default-layers invariant declared by the `layers` capability. If a deserialized `.ozp` contains zero track layers, the loader SHALL append one default track layer before returning the project. If a deserialized `.ozp` contains zero waypoint layers, the loader SHALL append one default waypoint layer. Existing layers SHALL NOT be modified or reordered.

This normalization SHALL run on every load path (Save / Load action, session-restore on startup, bundle-driven project loads). The `.ozp` save format SHALL NOT change as part of this normalization; projects are written with whatever layers the in-memory project contains at save time.

#### Scenario: Legacy project with no layers loads with defaults appended

- **WHEN** the user loads a `.ozp` file that contains zero track and zero waypoint layers
- **THEN** the loaded project exposes one default track layer named `"Tracks"` and one default waypoint layer named `"Waypoints"`, both empty

#### Scenario: Normalization does not rewrite the file on disk

- **WHEN** the user loads a legacy `.ozp` file that gets normalized in memory but does not save afterwards
- **THEN** the file on disk remains byte-for-byte identical to its pre-load contents

#### Scenario: Normalized project saves with explicit defaults

- **WHEN** the user loads a legacy `.ozp` that was normalized in memory and then saves it
- **THEN** the saved file now contains the previously-appended default layers as ordinary layer entries

### Requirement: Session restore registers the active map layer so the workspace lands at calibrated bounds

When the application restores a persisted active map on startup (`Application::restore_session` in `src-tauri/src/application/mod.rs`), the restored `ActiveMapSelection` SHALL be pushed through the same active-map registration code path that a user-initiated `open_selected_map` invokes — i.e. the function that registers the tile-source layer and prepares the metadata that the frontend `MapView` reads to call `fitBounds(meta.bounds)`. The restored map SHALL NOT merely be assigned to `self.lizaalert.active_map` without registration.

Errors from the registration step SHALL be reported via `update_status` with `DiagnosticLevel::Error` and the active map SHALL be cleared, matching the existing failure mode of the click-open path. The session-restore flow SHALL NOT silently leave a half-restored active map (selection set, layer not registered) on the way out.

#### Scenario: Cold-start restores last map at fit-to-bounds

- **WHEN** the user has previously opened an OZI map AND the session file records it as the active map AND the user relaunches the application AND the map file still exists at its recorded path
- **THEN** the workspace `/project` route renders with the map's calibrated bounds visible (i.e. the same viewport the user would see immediately after a fresh click-open) AND no JSON parse / stringify error toast is shown AND no `data-testid="ipc-error"` toast is shown in dev builds

#### Scenario: Cold-start matches click-open viewport

- **WHEN** the user clicks an OZI map row in the bundle loader from a cold start (no session) AND records the resulting MapLibre viewport (`map.getBounds()`) AND then closes the app AND relaunches it
- **THEN** the viewport after relaunch matches the viewport from the click-open (within MapLibre's animation tolerance) AND not the default MapLibre `{ center: [0,0], zoom: 0 }`

#### Scenario: Restored map missing on disk falls back cleanly

- **WHEN** the session file records an active map whose `local_path` no longer exists on disk AND the application relaunches
- **THEN** the existing diagnostic "Session restore skipped missing active map: …" is emitted AND the workspace either falls back to the bundle-loader cold-start surface (no active map) AND no half-registered active map remains in `self.lizaalert.active_map`

#### Scenario: Active map registration error during restore is surfaced

- **WHEN** session restore validates the selection and the layer-registration step fails (e.g. tile-source registration returns an error)
- **THEN** the error is reported via `update_status` with `DiagnosticLevel::Error` AND `self.lizaalert.active_map` is reset to `None` AND the workspace falls back to the cold-start bundle loader

