## ADDED Requirements

### Requirement: The operator can capture a moment as a folder

The system SHALL let the operator capture, in one action, what is on screen
together with what the application knows: the window as an image, the recent
diagnostics as readable text, the application state, and which build on which
machine. These SHALL be written into one dated folder per capture, under a
single folder the operator can open, so that a session's worth is handed over
together.

The action SHALL be available in a release build, not only a debug one.

The system SHALL then offer to record what happened, in the operator's own
words, into the same folder. Declining SHALL leave the capture intact.

A capture whose screenshot cannot be taken SHALL still be written, and the
interface SHALL say what is missing rather than failing the whole report.

Each diagnostic SHALL carry the time it was recorded.

#### Scenario: Something goes wrong at four in the morning

- **WHEN** the operator asks to capture the moment
- **THEN** a folder appears holding the screen, the diagnostics with their
  times, the state and the build, and the interface offers to add a line about
  what happened

#### Scenario: The description comes after

- **WHEN** the operator writes what happened and saves it
- **THEN** it is written into the folder captured a moment before, not a new one

#### Scenario: Declining to describe it

- **WHEN** the operator dismisses the question
- **THEN** the folder is still there with everything but the description

#### Scenario: No permission to photograph the screen

- **WHEN** the screenshot cannot be taken
- **THEN** the rest of the report is written and the interface says which
  permission is missing
