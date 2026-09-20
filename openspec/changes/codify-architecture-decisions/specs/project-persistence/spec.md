## ADDED Requirements

### Requirement: Project and session files are written atomically

The system SHALL write every `.ozp` project file and the app session file through a write-to-temp-then-rename sequence: the contents go to a sibling file named `<target>.tmp`, that file is flushed and fsynced, and it is then renamed over the target path. A save that fails at any step SHALL leave a pre-existing file at the target path byte-for-byte unchanged and SHALL remove its own temp file. A failed project save SHALL be reported through `update_status` with `DiagnosticLevel::Error` (`Save failed: …`) and SHALL NOT update the current project path nor clear the unsaved-changes state. No `.tmp` sibling SHALL remain after a successful save.

#### Scenario: Failed save keeps the previous file intact

- **WHEN** a project was previously saved to `mission.ozp` AND a later save to the same path fails (for example the parent directory is read-only, so the temp file cannot be created)
- **THEN** the save returns an error, `mission.ozp` still loads to the previously saved project, no `mission.ozp.tmp` exists, and the status shows `Save failed: …`

#### Scenario: Successful save leaves no temp sibling

- **WHEN** a save to `mission.ozp` succeeds
- **THEN** `mission.ozp` holds the new contents and `mission.ozp.tmp` does not exist

#### Scenario: Failed rename cleans up the temp file

- **WHEN** the final rename fails (for example a directory occupies the target path)
- **THEN** the save returns an error and no `.tmp` sibling is left behind

#### Scenario: Session snapshot uses the same atomic write

- **WHEN** the app writes its session snapshot after a successful project save
- **THEN** the session file is written through the same temp-fsync-rename sequence; if that write fails, the previous session file is intact and a `Session save failed: …` diagnostic is pushed with `DiagnosticLevel::Error`

### Requirement: `.ozp` content is pretty-printed JSON with a stable top-level shape

The `.ozp` file SHALL be pretty-printed UTF-8 JSON whose top-level value is an object with the keys `id`, `name`, `map_layers`, `track_layers` and `waypoint_layers`. Identifier newtypes (`ProjectId`, `LayerId`, `TrackId`, `TrackSegmentId`, `TrackPointId`, `WaypointId`) SHALL serialize as bare JSON integers. Optional track-point fields (`elevation`, `timestamp`) SHALL be omitted when unset. Fields with defaults (`style` on a track, `visible` on a waypoint) SHALL be tolerated when absent on load, so files written before those fields existed remain readable.

#### Scenario: Saved file is a JSON object with the expected keys

- **WHEN** the user saves a project named `Untitled Project`
- **THEN** the file parses as a JSON object, its `name` is `"Untitled Project"`, and it has the keys `id`, `name`, `map_layers`, `track_layers` and `waypoint_layers`

#### Scenario: Minimal hand-written file loads

- **WHEN** a file containing `{"id": 1, "name": "Legacy Project", "map_layers": [], "track_layers": [], "waypoint_layers": []}` is loaded
- **THEN** the load succeeds and the project is named `Legacy Project` (default layers are appended in memory per the normalization requirement)

#### Scenario: Waypoint without a `visible` field loads as visible

- **WHEN** a `.ozp` file contains a waypoint object with no `visible` key
- **THEN** the loaded waypoint is visible

#### Scenario: Unset optional point fields are not written

- **WHEN** a track point without elevation or timestamp is saved
- **THEN** its JSON object contains `id`, `latitude` and `longitude` and contains neither an `elevation` nor a `timestamp` key
