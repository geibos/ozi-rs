## ADDED Requirements

### Requirement: The stand answers in the shapes the bindings declare

The browser stand SHALL type each command's answer against the generated
bindings, so that an answer whose shape the application does not expect fails
the type check rather than the screen. Any command whose answer deliberately
differs SHALL be listed as an exception with its reason.

#### Scenario: A stub drifts from its DTO

- **WHEN** a stand answer is written with fields the command's DTO does not declare
- **THEN** the type check fails, naming the command and the missing fields

#### Scenario: A deliberate difference

- **WHEN** an answer differs from its binding on purpose, as the tile commands do
- **THEN** it is declared as an exception rather than widening every answer's type
