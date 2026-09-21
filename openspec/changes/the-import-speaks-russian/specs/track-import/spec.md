## ADDED Requirements

### Requirement: A folder import reports in the operator's language

Importing a folder of recordings SHALL report what it did as counts — files
read, tracks imported, waypoints imported — and the interface SHALL put those
into words, rather than the backend sending a sentence to be displayed. Files
that could not be read SHALL be named, by file name rather than by path. A
folder in which some files were read and others were not SHALL be reported as a
success carrying a caveat, not as a failure.

#### Scenario: A day's archive imports

- **WHEN** the operator imports a folder and every file is read
- **THEN** the result is stated in the interface language, naming how many tracks and waypoints came from how many files

#### Scenario: One navigator's file is unreadable

- **WHEN** a folder imports with one file unread
- **THEN** the rest is reported as imported, and the unread file is named alongside rather than replacing the result
