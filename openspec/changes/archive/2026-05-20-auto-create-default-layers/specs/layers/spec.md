## ADDED Requirements

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
