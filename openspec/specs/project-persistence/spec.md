# project-persistence Specification

## Purpose
Covers how a SAR project and the bounded app session reach disk and come back: the `.ozp` JSON project file (its shape, atomic writes, load-time normalization), the app session file that remembers only the last project path and the active map, and the startup restore flow including its degraded states when referenced files are missing. Map data itself is never part of a project file; see `map-bundles`.

### Decision history

- ADR-0004 (2026-03-29, accepted): persist projects as pretty-printed JSON `.ozp` files via `serde_json`, with transparent ID newtypes and `default`/`skip_serializing_if` on optional fields; rationale: human-readable, debuggable without tooling, round-trip stable, no custom parser to maintain. Codified as: Project is persisted as a JSON `.ozp` file; `.ozp` content is pretty-printed JSON with a stable top-level shape; Project load normalizes missing default layers. The ADR's follow-up `version` field was never added — the format has no schema-version field today (tracked separately as CJ-8). Reality note (resolved 2026-09-21): the dialogs used to filter on the `json` extension, which made an `.ozp` unselectable in the open dialog; both now read the extension from `src/lib/project-file.ts`, saving `.ozp` and opening `ozp` or `json`. The Rust layer still accepts any path.
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

### Requirement: The save and open dialogs offer the project format

The dialog that saves a project SHALL offer the `.ozp` extension, and the
dialog that opens one SHALL list `.ozp` files. A filter hides what it does not
match, so a dialog filtering on anything else makes a project file unselectable
however correctly it was written.

The open dialog SHALL also accept the `json` extension, because projects saved
by earlier builds of this application carry it and must stay openable.

#### Scenario: Opening a project saved by another build

- **WHEN** the operator opens a project file with the `.ozp` extension
- **THEN** the open dialog lists it

#### Scenario: Saving a project for the first time

- **WHEN** the operator saves a never-saved project
- **THEN** the dialog offers the `.ozp` extension

### Requirement: Recently opened projects are offered for reopening

The system SHALL remember the projects most recently opened or saved and offer
them for reopening without a file dialog, because a crew returns to the same
search for days and the one they want is almost always the one they had open
last.

A project SHALL be recorded when it is opened and when it is saved — a save is
where a never-saved project first receives a path — and SHALL NOT be recorded
when the save fails. The list SHALL be bounded and SHALL NOT list one project
twice.

A remembered path that no longer opens SHALL be removed from the list and the
failure reported, rather than left to fail again. An unreadable list SHALL
behave as an empty one rather than preventing the surface that offers it from
opening.

#### Scenario: Coming back to the same search

- **WHEN** the operator opens the command palette after having saved a project
- **THEN** that project is offered by name, and choosing it opens it without a file dialog

#### Scenario: A project that has moved

- **WHEN** a remembered project no longer opens
- **THEN** the operator is told and the entry is removed from the list

### Requirement: A project saved by an earlier build still opens

A `.ozp` written by any earlier build of the application SHALL still load. The
format carries no version field and has no migration path, so every field added
to a persisted structure SHALL be readable from a file that lacks it — by being
optional, or by carrying a default — and the absence of a field in an older file
SHALL mean what it meant when the file was written.

The application session file SHALL be held to the same rule. A session that
cannot be read costs the restored project and the active map as well, and an
unreadable session SHALL be distinguishable from an absent one, which is a
first run.

This SHALL be enforced by a test that loads a file carrying only the fields
that have always existed, covering the project, its layers, tracks, segments,
points and waypoints, rather than by review.

#### Scenario: Opening a project from an early build

- **WHEN** a project file is loaded that carries only a track with segments and points, and a waypoint, without any field added since
- **THEN** it loads, the track is visible, and the waypoint has no symbol, no colour and is visible

#### Scenario: A session from before a field was added

- **WHEN** an application session file is read that lacks a field added since it was written
- **THEN** it reads, and that field is absent rather than defaulted to a value the file never carried

#### Scenario: A field added without a default

- **WHEN** a persisted structure gains a field that older files cannot supply
- **THEN** the test suite fails and names the field

### Requirement: Opening a project puts its contents on screen

Opening a project file SHALL frame the map on what that project contains,
whether it was chosen from a dialog or from the recent-projects list. The frame
SHALL cover waypoints as well as tracks, so that a project holding only
waypoints is framed too. An extent too small to fit a camera to — a single
point, or a few metres across — SHALL be centred at a readable zoom rather than
fitted. A coordinate that is not a finite number SHALL be ignored rather than
allowed to spoil the frame.

#### Scenario: Reopening yesterday's search

- **WHEN** the operator opens a saved project whose tracks lie outside the current view
- **THEN** the map moves to show them, rather than leaving a view that looks like the project failed to load

#### Scenario: A project of waypoints

- **WHEN** the opened project holds waypoints and no track geometry
- **THEN** the map is framed on the waypoints

#### Scenario: A project holding one point

- **WHEN** everything the project contains sits within a few metres
- **THEN** the map is centred on it at a readable zoom, not at the maximum zoom

### Requirement: A saved project is reachable without the command palette

The screen the application opens on SHALL offer to open a saved project file,
and SHALL list the most recently opened projects as direct choices. No control
SHALL use the same words for opening a saved project and for opening the
LizaAlert catalogue, since they are different things.

#### Scenario: A crew arrives with yesterday's work

- **WHEN** the application is launched and no project is open
- **THEN** the first screen offers to open a saved project, and lists the recent ones, without the operator needing the command palette

#### Scenario: A recent project has moved

- **WHEN** an entry in that list no longer opens
- **THEN** the failure is reported and the entry is dropped, rather than failing again the next time it is chosen

#### Scenario: Opening from either surface

- **WHEN** a project is opened from the first screen or from the command palette
- **THEN** the same thing happens: it is loaded, remembered, and the map is framed on it

