## ADDED Requirements

### Requirement: A notification's message comes from the dictionary

Every notification shown to the operator SHALL take its message from the
interface dictionary rather than from a string written into a component or
built by the backend. The detail beside it MAY be raw — a backend error's own
words are evidence, and translating them would hide what failed.

This SHALL be enforced by a test over the source rather than by review, since
the same defect has reached the screen three times.

#### Scenario: An ordinary edit fails

- **WHEN** renaming a waypoint, hiding a track, exporting, simplifying or deleting fails
- **THEN** the message is in the interface language, and the failure's own text appears as the detail

#### Scenario: A message typed into a component

- **WHEN** a notification is raised with a message written as a literal string
- **THEN** the build fails, naming the file and line
