## ADDED Requirements

### Requirement: Opening a saved project leaves the launcher for the work

When a saved project is opened from the launcher — through the file dialog or
from the recent list — the system SHALL close the launcher and show the
workspace, as it does when a map bundle is opened. An operator SHALL NOT be
left looking at the catalogue with their project loaded behind it.

A choice that did not open a project — a cancelled dialog, a file that could
not be read — SHALL leave the operator where they were, with the failure
reported.

#### Scenario: A colleague's project arrives

- **WHEN** the operator opens a `.ozp` from the launcher
- **THEN** the launcher closes, the workspace shows the project, and saving
  writes back to the same file without asking where

#### Scenario: A file that would not read

- **WHEN** the chosen file cannot be read
- **THEN** the failure is reported, the launcher is still on screen, and the
  path is dropped from the recent list if that is where it came from

#### Scenario: A cancelled dialog

- **WHEN** the operator opens the file dialog and cancels it
- **THEN** nothing happens and the launcher is still on screen

### Requirement: The recent-projects list shows what is recent

Any screen showing the recently opened projects SHALL show the list as it is,
not as it was when the screen was built. A project opened or saved while that
screen is on it SHALL appear on it.

#### Scenario: Opening from the screen that lists them

- **WHEN** the operator opens a project from the launcher
- **THEN** that project is at the top of the launcher's recent list
