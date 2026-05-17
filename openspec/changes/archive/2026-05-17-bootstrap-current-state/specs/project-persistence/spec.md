## ADDED Requirements

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

## References

- ADR-0002: Separate Map Bundle from Project
- ADR-0004: JSON Project Persistence (.ozp Files)
- ADR-0019: Documentation Audit Reconciliation
