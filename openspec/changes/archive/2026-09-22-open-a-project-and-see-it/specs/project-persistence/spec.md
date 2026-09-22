## ADDED Requirements

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
