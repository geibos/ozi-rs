## ADDED Requirements

### Requirement: Recently opened projects are offered for reopening

The system SHALL remember the projects most recently opened or saved and offer
them for reopening without a file dialog, because a crew returns to the same
search for days and the one they want is almost always the one they had open
last.

A project SHALL be recorded when it is opened and when it is saved — a save is
where a never-saved project first receives a path — and SHALL NOT be recorded
when the save fails. The list SHALL be bounded and SHALL NOT list one project
twice.

A remembered path that no longer opens SHALL be removed from the list and the
failure reported, rather than left to fail again. An unreadable list SHALL
behave as an empty one rather than preventing the surface that offers it from
opening.

#### Scenario: Coming back to the same search

- **WHEN** the operator opens the command palette after having saved a project
- **THEN** that project is offered by name, and choosing it opens it without a file dialog

#### Scenario: A project that has moved

- **WHEN** a remembered project no longer opens
- **THEN** the operator is told and the entry is removed from the list
