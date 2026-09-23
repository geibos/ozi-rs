## ADDED Requirements

### Requirement: Simplifying shows its cost before it is applied

The simplify control SHALL show how many points the track has and how many it
would keep, from the moment it opens, at whatever tolerance it opens with, and
from every entrance.

Without it the operator can apply a simplification having never been told what
it would remove, which is the one thing a live preview exists to prevent.

#### Scenario: Opened from the Track Inspector

- **WHEN** the operator opens the simplify control from the inspector
- **THEN** it shows the current and resulting point counts without them having to move anything

#### Scenario: Changing the tolerance

- **WHEN** the operator moves the tolerance
- **THEN** the counts follow it, and one settled change fetches one preview

### Requirement: An edit that changes a track's shape invalidates what is drawn from it

Every operation that changes a track's geometry SHALL invalidate the cached
track detail and the drawn line.

The statistics, the segment table and the map are three views of one track. An
operation that moves one and not the others reads as having done nothing,
which is worse than an error.

#### Scenario: Simplifying a track

- **WHEN** the operator simplifies a track
- **THEN** the statistics, the segment table and the line on the map all show the simplified track
