## ADDED Requirements

### Requirement: A project saved by an earlier build still opens

A `.ozp` written by any earlier build of the application SHALL still load. The
format carries no version field and has no migration path, so every field added
to a persisted structure SHALL have a default, and the absence of a field in an
older file SHALL mean what it meant when the file was written.

This SHALL be enforced by a test that loads a file carrying only the fields
that have always existed, covering the project, its layers, tracks, segments,
points and waypoints, rather than by review.

#### Scenario: Opening a project from an early build

- **WHEN** a project file is loaded that carries only a track with segments and points, and a waypoint, without any field added since
- **THEN** it loads, the track is visible, and the waypoint has no symbol, no colour and is visible

#### Scenario: A field added without a default

- **WHEN** a persisted structure gains a field that older files cannot supply
- **THEN** the test suite fails and names the field
