## ADDED Requirements

### Requirement: Only the newest waypoint-list reload may be shown

The waypoint list SHALL show the result of the most recently started reload
only, and SHALL discard an earlier reload's rows and an earlier reload's
failure alike. Reloads overlap because the list reloads on every
application-state change.

#### Scenario: A reload overtaken while a bundle downloads

- **WHEN** a waypoint-list reload is still running and a newer one completes first
- **THEN** the rows shown are the newer reload's, and the older one changes nothing when it finishes

### Requirement: Waypoint layers are read together

A reload of the waypoint list SHALL request every layer's waypoints
concurrently rather than one layer after another, so that the cost of the list
does not grow with a round trip per layer.

#### Scenario: A project of many import-created layers

- **WHEN** the waypoint list is reloaded for a project with many waypoint layers
- **THEN** the layers are requested together
