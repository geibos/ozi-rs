## ADDED Requirements

### Requirement: A waypoint file in another datum says so

When an imported OziExplorer waypoint file declares a datum that is not in the
WGS 84 family, the system SHALL warn that the coordinates may be displaced.

This application does not transform between datums, and does not intend to.
Taking the coordinates in silence puts another headquarters' marks 100–150 m
from where they meant them, and nobody finds out until a crew is standing
there.

#### Scenario: A file from a headquarters working in Pulkovo 1942

- **WHEN** a `.wpt` declaring a non-WGS 84 datum is imported
- **THEN** the marks are imported and a warning about the possible displacement is recorded
