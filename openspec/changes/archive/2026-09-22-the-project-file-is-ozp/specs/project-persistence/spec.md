## ADDED Requirements

### Requirement: The save and open dialogs offer the project format

The dialog that saves a project SHALL offer the `.ozp` extension, and the
dialog that opens one SHALL list `.ozp` files. A filter hides what it does not
match, so a dialog filtering on anything else makes a project file unselectable
however correctly it was written.

The open dialog SHALL also accept the `json` extension, because projects saved
by earlier builds of this application carry it and must stay openable.

#### Scenario: Opening a project saved by another build

- **WHEN** the operator opens a project file with the `.ozp` extension
- **THEN** the open dialog lists it

#### Scenario: Saving a project for the first time

- **WHEN** the operator saves a never-saved project
- **THEN** the dialog offers the `.ozp` extension
