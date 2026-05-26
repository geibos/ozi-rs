## ADDED Requirements

### Requirement: Selecting a Library row activates the row's owning layer

When the user clicks a Track row in the `LibraryRail` Tracks tab whose owning track layer is not the current active track layer, the system SHALL call `set_active_track_layer` with the row's owning layer ID before applying any selection effects (`$selectedTrack` update, MapView highlight). The same SHALL hold for Waypoint rows in the Waypoints tab and `set_active_waypoint_layer`.

This row-activation behavior is non-destructive and consistent with the existing "Selecting an active layer is non-destructive" requirement: no overlays SHALL be hidden, unloaded, or modified as a result of the active-layer switch — only the routing target for subsequent edits SHALL change.

The Library's active-layer selectors (in the Tracks and Waypoints tab headers) SHALL reflect the post-click active layer immediately.

#### Scenario: Clicking a Track row from a non-active layer flips the active layer

- **WHEN** track layer "A" is the active track layer AND the project has a track "T" in layer "B" AND the user clicks the row representing "T" in the Tracks tab
- **THEN** the system calls `set_active_track_layer` with B's ID before setting `$selectedTrack` to T; the Tracks tab's active-layer Select trigger label updates from "A" to "B"; both layers' overlays remain rendered on the map

#### Scenario: Clicking a Waypoint row from a non-active layer flips the active waypoint layer

- **WHEN** waypoint layer "A" is the active waypoint layer AND the user clicks a Waypoint row whose owning layer is "B"
- **THEN** the system calls `set_active_waypoint_layer` with B's ID before setting `$selectedWaypointId` to the row's waypoint ID; the Waypoints tab's active-layer Select trigger label updates from "A" to "B"; no waypoint markers disappear from the map

#### Scenario: Clicking a row whose layer is already active does not re-call the setter

- **WHEN** layer "A" is the active track layer AND the user clicks a Track row whose owning layer is also "A"
- **THEN** the system does NOT call `set_active_track_layer` (the active layer is unchanged); `$selectedTrack` is updated as usual
