## ADDED Requirements

### Requirement: The bundle loader keeps the operator's place while it is closed

The loader SHALL restore the operator's browsing position when it is reopened
within the same session, covering the catalogue filter, the "only downloaded"
restriction, the selected project, the bundle contents the operator has cleared
and the scroll offset in the project list. The position SHALL NOT outlive the
session, so that a later run starts on the current search rather than an old
one.

#### Scenario: Closing the loader to look at the map

- **WHEN** the operator has filtered the catalogue and selected a project, closes the loader, and opens it again
- **THEN** the filter, the selection and the place in the list are as they left them

#### Scenario: A fresh session

- **WHEN** the application is started and the loader is opened for the first time
- **THEN** the catalogue is unfiltered and nothing is selected
