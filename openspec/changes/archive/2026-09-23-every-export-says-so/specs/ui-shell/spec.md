## ADDED Requirements

### Requirement: Every successful export names the file it wrote

The system SHALL confirm each completed export by naming the file that was
written, SHALL show the full path it was written to, and SHALL offer an action
that opens that file's location in the operating system's file manager.

This holds for every export the interface offers — tracks and waypoints, whole
layers and single rows — and whichever way the operator reached it: a row's
menu, an inspector, or the command palette. A confirmation present on one
route to an export and absent on another is the same silence, found later.

#### Scenario: A single track written to PLT

- **WHEN** the operator exports one track to `20260708_Veter2.plt` and the
  write succeeds
- **THEN** the interface says that `20260708_Veter2.plt` was written, shows the
  full path it was written to, and offers to show it

#### Scenario: Marks written to WPT

- **WHEN** the operator exports a waypoint layer to a `.wpt` file and the write
  succeeds
- **THEN** the interface names that file and offers to show it, in the same
  words the track exports use

#### Scenario: Showing the written file

- **WHEN** the operator takes the offered action after an export
- **THEN** the system opens a file manager on that file

#### Scenario: A failed export is not reported as written

- **WHEN** an export fails
- **THEN** the interface reports the failure and does not claim a file was
  written

### Requirement: The stand behaves as the application behaves

The stand SHALL reproduce the behaviour a screen depends on, not merely answer
the commands a screen sends. Specifically it SHALL emit `state-changed` after
exactly the commands whose real implementation emits it, SHALL track whether
the project has unsaved work across the commands that change it and the
commands that save it, and SHALL be able to raise a window close request.

A stand that answers a command and then tells the interface nothing happened
produces defects that do not exist in the product, which is worse than a gap:
it costs an afternoon and teaches whoever walked it to distrust what they see.

#### Scenario: An edit reaches the interface

- **WHEN** a screen on the stand sends a command whose real implementation
  emits `state-changed`
- **THEN** the stand emits it too, and the interface refreshes as it would in
  the packaged application

#### Scenario: Unsaved work is visible on the stand

- **WHEN** an operator makes an edit on the stand and then saves
- **THEN** the unsaved-changes indicator turns after the edit and clears after
  the save

#### Scenario: The close guard can be walked

- **WHEN** a close request is raised on the stand while the project has unsaved
  work
- **THEN** the guard holds the window open and asks, and the window is
  destroyed only once the operator confirms
