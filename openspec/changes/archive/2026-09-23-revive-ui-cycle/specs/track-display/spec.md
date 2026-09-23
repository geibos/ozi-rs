## ADDED Requirements

### Requirement: Track geometry is delivered and rendered per segment

`get_tracks_geojson` SHALL return a typed FeatureCollection in which each visible track is one Feature whose geometry is a `MultiLineString` with one part per track segment. Segments with fewer than two points SHALL be omitted; a track with no remaining parts SHALL be omitted. The response type SHALL be exported through specta as `TracksGeoJsonDto` rather than `JsonValue`. The map SHALL therefore never draw a connecting line between the last point of one segment and the first point of the next.

#### Scenario: Segment gap is not bridged

- **WHEN** a track has two segments recorded on different days 5 km apart
- **THEN** the map shows two separate polylines and no straight line joining them

#### Scenario: Split becomes visible

- **WHEN** the user splits a segment at a point and the track geometry refreshes
- **THEN** the Feature for that track has one more `MultiLineString` part than before

#### Scenario: Degenerate segment is skipped

- **WHEN** a track contains a segment with a single point
- **THEN** that segment produces no part and the remaining segments render normally

## MODIFIED Requirements

### Requirement: System computes and surfaces per-track statistics

The system SHALL compute, for each track, total distance in kilometres, elapsed duration when point timestamps are present, and total point count, and SHALL surface these in the track row UI. Elapsed duration SHALL be the span between the earliest and latest point timestamps across all segments; the UI SHALL label it as such via a tooltip ("from first to last point") and SHALL format spans of 24 hours or more as days and hours (`Nd Hh`) so multi-day recordings are not mistaken for errors.

A track holding fewer than two points SHALL surface its point count alone. It has no length and no elapsed span, so a distance and a duration for it are not measurements of anything: the row read "0.0 км · 0мин · 1 тчк", two such numbers standing in front of the one that means something. A point count of one SHALL use the singular unit where the language has one.

This last part is new because such tracks had no row at all until the track list stopped being derived from the map's geometry, which omits what it cannot draw.

#### Scenario: Track with timestamps

- **WHEN** a track has GPS timestamps on its points
- **THEN** the track row shows distance in km, elapsed duration, and point count

#### Scenario: Track without timestamps

- **WHEN** a track has no point timestamps
- **THEN** the track row shows distance and point count; duration is omitted

#### Scenario: Track row shows distance, duration, and point count

- **WHEN** the user opens a project containing a track with three timestamped segments totalling 12.3 km, 1 hour 24 minutes, and 156 points
- **THEN** the Tracks panel row for that track displays distance "12.3 km", duration "1h 24m", and point count "156 pts" alongside the track name

#### Scenario: Track without timestamps hides duration

- **WHEN** the user displays a track whose points have no timestamps
- **THEN** the Tracks panel row shows distance and point count but omits the duration field

#### Scenario: Multi-day span is formatted in days

- **WHEN** a track's first and last timestamps are 26 days and 5 hours apart
- **THEN** the row shows "26d 5h" and hovering the duration shows the tooltip "from first to last point"

#### Scenario: A track of a single point

- **WHEN** the track list shows a track whose only segment holds one point
- **THEN** its statistics read as a point count alone, without a distance or a duration
