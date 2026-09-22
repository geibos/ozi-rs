## ADDED Requirements

### Requirement: The catalogue refresh says how far it has got

While a catalogue refresh runs, the interface SHALL state how many projects are
already listed, so that the decision to stop the refresh can be made on what is
there rather than on elapsed time alone.

#### Scenario: A crew watching the refresh

- **WHEN** a catalogue refresh is running and projects have arrived
- **THEN** the refreshing hint states how many are listed so far
