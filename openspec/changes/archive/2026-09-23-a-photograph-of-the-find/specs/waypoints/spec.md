## ADDED Requirements

### Requirement: A mark carries the files that belong to it

A waypoint SHALL carry a list of files — a photograph of the find, a scan, a
note taken at the spot — and the operator SHALL be able to attach one, show it
in the file manager, and detach it.

The files SHALL be stored as paths beside the project, not as bytes inside it.
A `.ozp` is exchanged between headquarters and read by people; a photograph
inside it would make it unreadable. Attaching a file SHALL NOT copy or move it.

The list SHALL be replaced as one undoable step. Blank paths SHALL be dropped
and the same path SHALL NOT be held twice.

A project saved before this existed SHALL load with nothing attached.

The files SHALL NOT be written to `.wpt` or GPX exports: neither format has a
place for them.

#### Scenario: Photographing a find

- **WHEN** the operator attaches two photographs to a mark
- **THEN** both are listed on the mark, and each can be shown or detached

#### Scenario: Detaching one of several

- **WHEN** the operator detaches the first of two files
- **THEN** exactly that one is gone and the other remains

#### Scenario: Undo puts the list back

- **WHEN** the operator attaches a file and invokes undo
- **THEN** the mark carries the list it had before

#### Scenario: The same file twice

- **WHEN** a path already attached is attached again
- **THEN** it is held once
