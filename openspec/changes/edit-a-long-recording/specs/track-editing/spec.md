## ADDED Requirements

### Requirement: Edit mode stays responsive on a long recording

Edit mode SHALL put draggable handles only on the selected track's points
that lie inside the visible map, and SHALL renew them each time the map stops
moving. When more than a thousand of the track's points are in view, it SHALL
put none and SHALL say how many points are in view and that zooming in
brings the handles. Entering edit mode on an 18 000-point track SHALL NOT
hold the interface for seconds.

#### Scenario: A phone's track

- **WHEN** edit mode is turned on for an 18 000-point track with the whole
  track in view
- **THEN** no handles are drawn, the map says how many points are in view
  and to zoom in, and the map pans freely

#### Scenario: Zoomed in

- **WHEN** the operator zooms in until fewer than a thousand points are in view
- **THEN** each of those points has a handle that can be dragged, and panning
  brings handles to the points that come into view
