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

The system SHALL NOT modify, hide, or unload layers when the user changes the active layer; only the routing target for new edits SHALL change. This applies uniformly to track layers and waypoint layers: overlays from inactive layers SHALL remain rendered on the map according to their own per-layer and per-item visibility flags.

#### Scenario: Switching active layer keeps overlays visible

- **WHEN** the user has two track layers visible on the map and switches the active track layer
- **THEN** both track layers remain rendered and visible; only new track creation targets the newly active layer

#### Scenario: Switching active waypoint layer keeps other waypoint markers visible

- **WHEN** the project has two waypoint layers A (active, two waypoints) and B (three waypoints), and the user changes the active waypoint layer from A to B
- **THEN** all five waypoint markers remain rendered on the map; only the "active for editing" affordance moves to B; no waypoints from A disappear

#### Scenario: Editing routes to active layer only

- **WHEN** layer A is the active waypoint layer and the user clicks the map in waypoint placement mode
- **THEN** the new waypoint is appended to layer A; waypoints in layer B are unaffected

### Requirement: An open project always contains at least one track layer and one waypoint layer

The system SHALL enforce the invariant that, whenever a `Project` is the currently-open project in `AppState`, it contains at least one `TrackLayer` and at least one `WaypointLayer`. This invariant SHALL be maintained at every construction path, including the empty / default constructor and the load-from-disk path.

When a project is constructed without explicit layers, the system SHALL append one default `TrackLayer` named `"Tracks"` and one default `WaypointLayer` named `"Waypoints"` before exposing the project to the application layer.

Default layers are ordinary layers in every other respect — they may be renamed, hidden, or have content added to them like any user-created layer.

#### Scenario: Default constructor produces both default layers

- **WHEN** the backend constructs a project via the default / empty path
- **THEN** the resulting project contains exactly one track layer named `"Tracks"` and exactly one waypoint layer named `"Waypoints"`, each empty

#### Scenario: Loaded project already has layers

- **WHEN** the backend loads a `.ozp` file that already contains two track layers and three waypoint layers
- **THEN** the loaded project preserves all five layers and does not append additional defaults

#### Scenario: Loaded project is missing one layer kind

- **WHEN** the backend loads a `.ozp` file that contains two track layers and zero waypoint layers
- **THEN** the loaded project preserves both track layers and gains one default waypoint layer named `"Waypoints"`

### Requirement: Active layer IDs are non-null whenever a project is open

The system SHALL guarantee that `AppStateDto.active_track_layer_id` and `AppStateDto.active_waypoint_layer_id` are non-null whenever an `AppStateDto` exposes a `current_project`. Frontend code consuming these IDs SHALL be entitled to assume non-null in any handler that runs while a project is open.

When no project is open, both IDs SHALL be null.

#### Scenario: Sidebar Create Track works on a fresh project

- **WHEN** the user opens a bundle for the first time and clicks "Create Track"
- **THEN** the drawing mode activates against the default track layer and the click handler does not early-return for a null layer ID

#### Scenario: Add Waypoint works on a fresh project

- **WHEN** the user opens a bundle for the first time, activates "Add Waypoint" mode, and clicks on the map
- **THEN** a waypoint is created in the default waypoint layer at the clicked coordinates

