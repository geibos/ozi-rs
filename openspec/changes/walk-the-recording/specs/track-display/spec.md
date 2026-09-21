## ADDED Requirements

### Requirement: A track's points can be stepped through

The operator SHALL be able to step to the next and previous point of a track
and SHALL be shown where in the recording they are. Reviewing a recording by
clicking each row loses the operator's place whenever the list scrolls.

Stepping SHALL move the map to the point, and SHALL follow the track's own
order across its segments: a recording made in two sittings is one walk.

Stepping SHALL stop at each end rather than wrapping, because a jump from the
last point to the first reads as a fault.

#### Scenario: Reviewing a two-sitting recording

- **WHEN** the operator steps forward past the last point of the first segment
- **THEN** the first point of the second segment is selected and the map moves to it

#### Scenario: At the end

- **WHEN** the last point of the track is selected
- **THEN** stepping forward is not offered
