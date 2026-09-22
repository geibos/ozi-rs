## ADDED Requirements

### Requirement: A project saved by an earlier build still opens

A `.ozp` written by any earlier build of the application SHALL still load. The
format carries no version field and has no migration path, so every field added
to a persisted structure SHALL be readable from a file that lacks it — by being
optional, or by carrying a default — and the absence of a field in an older file
SHALL mean what it meant when the file was written.

The application session file SHALL be held to the same rule. A session that
cannot be read costs the restored project and the active map as well, and an
unreadable session SHALL be distinguishable from an absent one, which is a
first run.

This SHALL be enforced by a test that loads a file carrying only the fields
that have always existed, covering the project, its layers, tracks, segments,
points and waypoints, rather than by review.

#### Scenario: Opening a project from an early build

- **WHEN** a project file is loaded that carries only a track with segments and points, and a waypoint, without any field added since
- **THEN** it loads, the track is visible, and the waypoint has no symbol, no colour and is visible

#### Scenario: A session from before a field was added

- **WHEN** an application session file is read that lacks a field added since it was written
- **THEN** it reads, and that field is absent rather than defaulted to a value the file never carried

#### Scenario: A field added without a default

- **WHEN** a persisted structure gains a field that older files cannot supply
- **THEN** the test suite fails and names the field
