## ADDED Requirements

### Requirement: A track exported to GPX and read back is the same track

A track written to GPX and imported again SHALL carry the same name, the same
division into segments, the same coordinates in the same order, and the same
point timestamps, including their absence where there were none.

The segment boundary is part of this: a track whose sittings are joined on the
way back claims a straight line between them that was never walked.

#### Scenario: A two-sitting track goes out and comes back

- **WHEN** a track of two segments, one of them timestamped, is exported to GPX and imported again
- **THEN** its name, its two segments, every coordinate in order and every timestamp are as they were
