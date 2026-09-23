# track-display Specification

## Purpose
How tracks are presented once they are in a project: per-track colour, line width, opacity and visibility; per-track statistics; the point-level view with per-point attributes; and the warning-only OK-standard name validation shown next to a track. Import, export and geometry edits belong to `track-import`, `track-export` and `track-editing`; the map engine belongs to `ui-shell` and `tile-rendering`.

### Decision history
- ADR-0013 (2026-03-29, accepted): `TrackStyle` (colour, width, visibility) is stored inside the `Track` domain entity rather than in a UI-side style table; rationale: style travels with the project file and round-trips through PLT `COLORREF` and GPX Garmin colour. Codified as: "Each track has independently controllable color, line width, opacity, and visibility"; the persistence and encoding details are codified in `project-persistence` and `track-export`.
- ADR-0019 (2026-04-25, accepted): render existing point timestamps in the point list, expose the existing colour and line-width mutations through compact track controls, show `YYYYMMDD_Callsign` validation as a warning that never blocks rename/save/export; rationale: docs described these as visible while the UI lacked them. Codified as: "Track points panel exposes per-point attributes", "Each track has independently controllable color, line width, opacity, and visibility", "Track name validation surfaces a non-blocking warning for non-conforming names", "Validation pattern is alphabet-agnostic and does not validate calendar dates" | style changes stay outside the undo stack per ADR-0019/ADR-0017 — the undo boundary is codified in `undo-redo`, not here.
- ADR-0020 (2026-04-28, accepted), Tracks section: many tracks at once, tens of thousands of points per track, per-track colour and line style, point-by-point walkthrough with per-point info, warning-only name validation; rationale: full-day SAR tracks routinely exceed 30k points. Codified as: the requirements above plus "System computes and surfaces per-track statistics" (archived `add-track-statistics-ui`); per-segment delivery is added by the active `revive-ui-cycle` change | next/previous walkthrough controls are not codified — points are click-selectable only (`docs/feature-status.md`, row "Track point walkthrough"); the scope itself is recorded in `product-scope`.

## Requirements

### Requirement: Each track has independently controllable color, line width, opacity, and visibility

The system SHALL store `color`, `line_width`, `opacity`, and `visible` per `Track`, and SHALL render each track using its own style. These mutations are immediate visual updates and are NOT recorded in undo history (see `undo-redo`).

#### Scenario: Color change applied immediately

- **WHEN** the user picks a new color in the track row of the Tracks panel
- **THEN** the track re-renders with the new color and the new color persists in the project

#### Scenario: Visibility toggle hides the track

- **WHEN** the user toggles a track's visibility off in the Tracks panel
- **THEN** the track polyline is removed from the map and the track row indicates hidden state

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

### Requirement: Track points panel exposes per-point attributes

The system SHALL provide a Track Points panel that lists each point's latitude, longitude, elevation (when present), segment index, and timestamp (when present).

#### Scenario: Point with timestamp

- **WHEN** the user opens the Track Points panel for a track whose points carry timestamps
- **THEN** each row displays the point's timestamp in a human-readable form

#### Scenario: Point without timestamp

- **WHEN** the user opens the Track Points panel for a track without timestamps
- **THEN** the timestamp column is omitted or empty without rendering placeholder text

### Requirement: Track name validation surfaces a non-blocking warning for non-conforming names

The system SHALL display a non-blocking warning indicator on tracks whose names do not match the LizaAlert OK-standard pattern `^\d{8}_.*\S.*$` (i.e. `YYYYMMDD_Callsign` with a non-empty callsign). The backend SHALL remain permissive: rename, save, and export operations SHALL NOT be blocked by a non-conforming name.

#### Scenario: Non-conforming name shows warning but does not block

- **WHEN** a track is renamed to `temp` (does not match the pattern)
- **THEN** the Tracks panel shows a warning indicator on that track and the rename, save, and export operations still succeed

#### Scenario: Conforming name has no warning

- **WHEN** a track is renamed to `20240601_Иванов`
- **THEN** the warning indicator is not shown

### Requirement: Validation pattern is alphabet-agnostic and does not validate calendar dates

The system SHALL accept any 8 leading digits followed by an underscore and a non-empty callsign in any alphabet (including Cyrillic). The system SHALL NOT validate that the leading 8 digits form a real calendar date.

#### Scenario: Cyrillic callsign

- **WHEN** a track is named `20240601_Иванов`
- **THEN** no warning is shown

#### Scenario: Impossible date passes pattern

- **WHEN** a track is named `20249999_Иванов`
- **THEN** no warning is shown — the warning checks pattern only, not calendar validity

### Requirement: Track visibility can be changed for all tracks at once

The system SHALL provide commands to show every track and to hide every track in
the project, each applied as one operation rather than per-track toggles. The
Tracks tab SHALL expose both. Visibility remains a non-undoable style mutation,
so a bulk change SHALL mark the project dirty without adding undo entries.

#### Scenario: Hiding every track

- **WHEN** a project holds twenty-six visible tracks across several layers and the operator chooses hide-all
- **THEN** every track becomes hidden in a single state update, and the map redraws once

#### Scenario: Showing every track again

- **WHEN** some tracks are hidden and the operator chooses show-all
- **THEN** every track in every layer becomes visible

#### Scenario: Bulk visibility does not fill the undo stack

- **WHEN** the operator hides all tracks and then presses undo
- **THEN** the undo stack is unchanged by the bulk operation, consistent with per-track visibility toggles

### Requirement: One track can be isolated from the rest

The system SHALL provide an action that makes one track visible and hides every
other track in the project in a single operation, reachable from that track's
row.

#### Scenario: Isolating a track for inspection

- **WHEN** the operator picks "only this one" on the row for `20260709-ЛИСА15`
- **THEN** that track is visible and every other track in every layer is hidden

#### Scenario: Isolating a hidden track shows it

- **WHEN** the chosen track is itself hidden at the time of the action
- **THEN** it becomes visible while the others are hidden

### Requirement: Every track in the project has a row

The track list SHALL show one row for every track in the project, including a
track the map cannot draw because it has fewer than two points in every
segment. Such a track is part of the project and is exported with it, so it
SHALL be selectable, renamable and deletable like any other.

The list SHALL NOT be derived from the map's features, whose omissions are
correct for drawing and wrong for a listing.

#### Scenario: A track of a single point

- **WHEN** the project contains a track whose only segment holds one point
- **THEN** the track list shows a row for it, and the map draws nothing for it

### Requirement: The track list is fetched without track geometry

The data behind the track list SHALL carry only what a row shows. Track
coordinates SHALL NOT be transferred in order to render the list, so that a
project of many long recordings costs the list nothing beyond its rows.

#### Scenario: A project of long recordings

- **WHEN** the track list is loaded for a project whose tracks hold hundreds of thousands of points
- **THEN** no track coordinates are transferred

### Requirement: Only the newest track-list reload may be shown

The track list SHALL show the result of the most recently started reload only,
and SHALL discard an earlier reload's rows and an earlier reload's failure
alike, so that the list never goes backwards. Reloads overlap because the list
reloads on every application-state change.

#### Scenario: A reload overtaken while a bundle downloads

- **WHEN** a track-list reload is still running and a newer one completes first
- **THEN** the rows shown are the newer reload's, and the older one changes nothing when it finishes

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

### Requirement: The inspector shows the track's elevation against distance

The track inspector SHALL draw the recorded elevation of the selected track
against distance travelled along it, and SHALL state the range in metres. A
point carrying no elevation SHALL still count towards the distance, so a gap in
the data does not move the samples around it. A track with fewer than two
elevation readings SHALL be reported as carrying no elevation rather than
drawn.

#### Scenario: A recording with elevation

- **WHEN** the operator selects a track whose points carry elevation
- **THEN** the inspector draws the profile across the card and states the lowest and highest readings

#### Scenario: A recording without elevation

- **WHEN** the selected track carries no elevation, or only one reading
- **THEN** the card says the recording carries no elevation, and no chart is drawn

#### Scenario: A gap in the elevation data

- **WHEN** a point between two readings carries no elevation
- **THEN** the later reading keeps its distance from the track's start, rather than moving towards the earlier one

### Requirement: The selected track is picked out on the map

The map SHALL distinguish the selected track from the others without changing
the colour that identifies it, so that a row in the list and a route on the map
can be matched by eye while the map carries no names.

Selecting nothing SHALL leave every track drawn as it was.

#### Scenario: One route among a day's

- **WHEN** a track is selected while a day's recordings are on the map
- **THEN** that route is visibly marked out, and still drawn in its own colour

#### Scenario: Stepping down the list

- **WHEN** the operator moves from one track to the next
- **THEN** the mark follows the selection, one route at a time

#### Scenario: Nothing selected

- **WHEN** no track is selected
- **THEN** no route is marked

### Requirement: Only the newest track-geometry fetch may be drawn

The map SHALL draw the geometry of the most recently started fetch only, and
SHALL discard an earlier fetch's result, so that the map does not fall behind
the track list it is meant to match. Two changes in quick succession leave two
fetches in flight.

#### Scenario: Two track edits in quick succession

- **WHEN** a second track-geometry fetch is started before the first answers
- **AND** the first answers last
- **THEN** the map draws the second fetch's geometry

### Requirement: A track carries its name on the map

Every visible track that has a name SHALL show that name on the map, drawn in
the track's own colour, positioned at the point half way along the route
measured by length.

A day of recordings is a dozen coloured lines. Without names the only way to
find one is to read its colour off the list and hunt.

The position SHALL be measured by distance rather than by point index: a crew
that stopped for twenty minutes logs a crowd of points at the rest stop, and
an index midpoint puts the name there rather than on the route.

#### Scenario: A day's routes on the map

- **WHEN** the operator looks at a day of imported tracks
- **THEN** each visible track has its name on it, in its own colour

#### Scenario: A track that stopped for a break

- **WHEN** a track's points are crowded at one end and sparse across the rest
- **THEN** its name sits half way along the distance walked, not among the crowded points

### Requirement: Names that do not fit are dropped, not stacked

Names that would overlap on screen SHALL be reduced to those that fit. A
selected track SHALL keep its name; otherwise the longer route SHALL keep it.

Below the zoom at which a whole district fits on screen, no names SHALL be
drawn: at that scale they are a smear rather than information.

#### Scenario: Two tracks crossing at the same place

- **WHEN** two tracks' midpoints fall within a name's width of each other
- **THEN** one name is drawn, not two overlapping

#### Scenario: The operator selects a track in a crowd

- **WHEN** a track is selected and its name would otherwise be dropped
- **THEN** its name is the one drawn

### Requirement: A name sits on a line the crew actually walked

A track's name SHALL be placed half way along the longest segment of that
track, not half way along its segments taken as one run.

A track is split where the recording stopped. Half the combined length falls
in the gap whenever the two stretches are far apart, and a name floating over
empty forest belongs to nothing.

#### Scenario: A track recorded in two stretches a kilometre apart

- **WHEN** a track has two segments with a kilometre of nothing between them
- **THEN** its name sits on one of the segments, not between them

### Requirement: Which name survives is decided by distance walked

When names compete for room, the system SHALL keep the selected track's name
first and then the name of the track whose crew walked furthest, measured in
distance rather than in the number of recorded points.

A navigator logging once a second at a rest stop records more points than a
long route logged once a minute. Whose callsign stays on the map must not be
decided by how often somebody's GPS wrote a line.

#### Scenario: A rest stop against a long route

- **WHEN** a short track with a thousand crowded points competes with a long track with two
- **THEN** the long track keeps its name

### Requirement: Names are kept apart by the room they take

Overlap SHALL be judged by the space each name occupies on screen, not by the
distance between their anchor points.

#### Scenario: Two long names sixty pixels apart

- **WHEN** two long names' anchors are further apart than the minimum spacing but their text overlaps
- **THEN** only one of them is drawn

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
