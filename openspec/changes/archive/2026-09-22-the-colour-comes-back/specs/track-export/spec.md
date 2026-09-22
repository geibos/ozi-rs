## MODIFIED Requirements

### Requirement: A track exported to GPX and read back is the same track

A track written to GPX and imported again SHALL carry the same name, the same
division into segments, the same coordinates in the same order, the same point
timestamps, including their absence where there were none, and the same colour.

The segment boundary is part of this: a track whose sittings are joined on the
way back claims a straight line between them that was never walked.

The colour is carried as a GPX colour name, so what survives is the nearest
name rather than the exact bytes. A track that declares no colour, or one whose
declared name is not a GPX colour, SHALL keep the application's default rather
than be given a guess. Colours SHALL be matched to tracks by their order in the
document.

#### Scenario: A two-sitting track goes out and comes back

- **WHEN** a track of two segments, one of them timestamped, is exported to GPX and imported again
- **THEN** its name, its two segments, every coordinate in order and every timestamp are as they were

#### Scenario: A coloured track goes out and comes back

- **WHEN** a track whose colour is one of the GPX colour names is exported and imported again
- **THEN** it comes back that colour, not the default

#### Scenario: A file where only some tracks declare a colour

- **WHEN** a GPX holds a coloured track, then one with no colour, then another coloured one
- **THEN** each coloured track gets its own colour and the middle one keeps the default

#### Scenario: A colour name from another program

- **WHEN** an imported track declares a colour name that is not a GPX colour
- **THEN** the track keeps the default colour and the import succeeds
