## ADDED Requirements

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
