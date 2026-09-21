## ADDED Requirements

### Requirement: No user-facing string is written into a component

Every user-facing string SHALL come from the interface's dictionaries rather
than being written as a literal in a component, and this SHALL be enforced by a
test over the components rather than by review, because the strings that slip
through are the ones nobody sees while writing them.

An exception SHALL be recorded in the test with its reason rather than left
implicit.

#### Scenario: A label added in one language

- **WHEN** a component is given an `aria-label`, `title` or `placeholder` written as a literal
- **THEN** the test suite fails and names the file and the value

#### Scenario: Editing a track on the map in Russian

- **WHEN** the point context menu is opened while the interface is Russian
- **THEN** its entries are in Russian
