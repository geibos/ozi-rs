## ADDED Requirements

### Requirement: Marks that could not be read are not marks that are gone

When the marks of a layer cannot be read, the system SHALL treat the layer's
contents as unknown rather than empty: the markers already on the map SHALL
stay, and the operator SHALL be told the read failed.

#### Scenario: One layer's read fails during a refresh

- **WHEN** reading one layer's marks fails while the others succeed
- **THEN** that layer's markers remain on the map, the others are refreshed, and the failure is reported
