## ADDED Requirements

### Requirement: Bringing everything into view frames the data

Framing the camera on everything the project holds SHALL use the tracks and the
marks as stored, not the markers that have been drawn on the map so far, so
that the result does not depend on how much of an asynchronous redraw has
finished. A layer whose marks could not be read SHALL fall back to what is
drawn for it.

#### Scenario: Asking for everything right after opening a project

- **WHEN** the operator asks to see everything before the markers have finished being drawn
- **THEN** the camera frames the marks as well as the tracks
