## ADDED Requirements

### Requirement: The points table shows each leg's distance and speed

Under each point after the first of a segment, the points table SHALL show the
distance from the previous point and, when both points carry a time, the
speed over that leg in km/h. The first point of a segment SHALL show neither:
the line between two segments is a gap, not a leg.

#### Scenario: A leg with times

- **WHEN** a point is 351 m from the previous one and recorded 20 s later
- **THEN** its row shows 351 m and 63 km/h

#### Scenario: The first point of a segment

- **WHEN** a point opens the second segment of a track
- **THEN** its row shows no distance and no speed

### Requirement: A track's jumps and outliers are listed for review

The inspector SHALL list the places in the selected track where the recording
is suspect, in track order, within each segment:

- a **jump** is a leg at least 100 m long and at least ten times the track's
  median leg;
- an **outlier** is a point whose legs in and out are each at least 30 m and
  three median legs, while its two neighbours lie closer to each other than
  0.35 of the two legs together.
- a **break** is a leg whose two points are six hours or more apart, however
  far — the tail of a previous search on a navigator nobody cleared; it is
  listed as a break, not as a jump.

Each entry SHALL show the distance and, when known, the speed of the legs
around it. Choosing an entry SHALL select its point and take the map to it.
A jump SHALL offer to split the segment between its two points; an outlier
SHALL offer to delete the point, and to delete it and split the segment there;
a break SHALL say how long it was and when either side was recorded, and SHALL
offer to delete everything before it or everything after it.
The list SHALL NOT change a track by itself.

#### Scenario: A spike is found by its apex

- **WHEN** a track goes 10 m a step and one point sits 400 m to the side and
  comes back
- **THEN** the list holds one outlier, at that point, and nothing at its
  neighbours

#### Scenario: A gap is found as a jump

- **WHEN** a track going 10 m a step has one leg of 900 m
- **THEN** the list holds one jump, at the point after that leg

#### Scenario: The tail of the last search

- **WHEN** a track's first five points were recorded on 26 September and the
  rest on 8 October
- **THEN** the list holds a break at the sixth point, and «Удалить всё до»
  leaves the track starting there

#### Scenario: Choosing an entry

- **WHEN** the operator chooses an entry
- **THEN** its point becomes the selected point and the map centres on it

### Requirement: An outlier can be cut out of a track as one undoable step

The system SHALL provide a `CutOutTrackPoint` command that removes a point
from the middle of a segment and splits the segment between the points that
were on either side of it. The left part SHALL keep the segment's id; the
right part SHALL get the id `max segment id + 1` within the track and follow
it directly. One undo SHALL restore the single segment with the point in its
place. The first and last points of a segment SHALL be rejected — there is
nothing to split; deleting them is a plain delete.

#### Scenario: Cut out a middle point

- **WHEN** a segment holds points `10, 11, 12, 13` and the operator cuts out `11`
- **THEN** the track holds a segment `10` followed by a segment `12, 13`, and
  one undo gives back `10, 11, 12, 13` in one segment

#### Scenario: An end point is not cut out

- **WHEN** the operator cuts out the first or last point of a segment
- **THEN** the command is refused and the track is unchanged
