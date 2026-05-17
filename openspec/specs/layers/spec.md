# layers Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
### Requirement: Project is composed of independent map, track, and waypoint layers

The system SHALL represent `MapLayer`, `TrackLayer`, and `WaypointLayer` as distinct concepts within a `Project`, each carrying its own identifier and contents. A project MAY contain multiple track layers and multiple waypoint layers.

#### Scenario: Multiple track layers in one project

- **WHEN** the backend constructs a project with two track layers and one waypoint layer
- **THEN** `AppStateDto` exposes layer summaries reflecting each layer separately

### Requirement: One track layer and one waypoint layer are designated as active

The system SHALL maintain a notion of an active track layer and an active waypoint layer for editing workflows (track creation, waypoint placement, drawing, point list display).

#### Scenario: Active layer selection routes new edits

- **WHEN** the user selects an active track layer in the sidebar and creates a new track
- **THEN** the new track is added to the selected active layer and not to other track layers

### Requirement: Backend exposes layer summaries to the UI

The system SHALL expose layer composition (identifiers, names, child counts) to the frontend via `AppStateDto` so the UI can render layer-aware selection controls.

#### Scenario: UI reads layer summaries

- **WHEN** the frontend fetches `AppStateDto`
- **THEN** the response includes a description of each layer sufficient to render the active-layer selector and per-layer counts

### Requirement: Selecting an active layer is non-destructive

The system SHALL NOT modify, hide, or unload layers when the user changes the active layer; only the routing target for new edits SHALL change.

#### Scenario: Switching active layer keeps overlays visible

- **WHEN** the user has two track layers visible on the map and switches the active track layer
- **THEN** both track layers remain rendered and visible; only new track creation targets the newly active layer

