## ADDED Requirements

### Requirement: Renaming a layer that is not there is an error

A rename aimed at a layer the project does not hold SHALL fail rather than
report success.

A command that answers `Ok` without doing anything still clears the redo
history and counts a mutation — the operator loses a redo they had, for an
edit that never happened.

#### Scenario: A rename of a layer that has been removed

- **WHEN** a rename names a layer that is not in the project
- **THEN** the command fails and the redo history is untouched
