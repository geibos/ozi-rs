## ADDED Requirements

### Requirement: The operator can create, rename and remove layers

The system SHALL let the operator create a track layer or a waypoint layer,
rename any layer, and remove any layer, from the library tab that lists it.
Creating a layer SHALL answer with the new layer's identifier so the caller
can make it active without reading the state back.

A day of recordings arrives as files and every file becomes a layer named
after its path. Without these the operator cannot tidy what the import made,
cannot discard a folder taken in by mistake, and cannot start an empty layer
to draw a plan into.

#### Scenario: A layer created from the library

- **WHEN** the operator creates a track layer and gives it a name
- **THEN** the layer appears in the list under that name and can be made active

#### Scenario: A layer renamed

- **WHEN** the operator renames a layer the import called `Imported tracks: /Volumes/…/day3.gpx`
- **THEN** the list, the active-layer control and the saved project all carry the new name

#### Scenario: A layer removed

- **WHEN** the operator removes a layer
- **THEN** the layer and everything in it leave the project, and the map stops drawing them

### Requirement: Removing the active layer leaves another one active

When the removed layer is the active one, the system SHALL make another layer
of the same kind active rather than leaving no active layer, so that drawing
and waypoint placement keep a target.

#### Scenario: Removing the active track layer

- **WHEN** the active track layer is removed and other track layers remain
- **THEN** one of the remaining track layers becomes active

#### Scenario: Removing the last layer of its kind

- **WHEN** the only track layer is removed
- **THEN** the project is left with no active track layer, and creating one makes it active again
