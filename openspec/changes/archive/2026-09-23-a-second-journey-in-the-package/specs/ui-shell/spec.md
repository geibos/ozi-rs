## ADDED Requirements

### Requirement: A layer select says which layer is active

The accessible name of a layer select SHALL carry the name of the layer that
is active, not only the kind of layer it selects.

A control announced as "track layer" tells a screen reader half of what it
is; the value itself lives in an element the packaged application's
accessibility tree does not publish, so it is unreadable from outside as well.

#### Scenario: A layer is active

- **WHEN** a track layer is active
- **THEN** the select's accessible name carries that layer's name

#### Scenario: No layer is active

- **WHEN** no layer is active
- **THEN** the select's accessible name is the kind of layer alone
