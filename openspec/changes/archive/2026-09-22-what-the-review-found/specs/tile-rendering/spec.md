## ADDED Requirements

### Requirement: A cold launch with no link adds no remote basemap

When the machine reports that it has no network at all, the map SHALL NOT add
a remote raster source. The local maps of the project are what the operator
opened the application for, and a remote basemap offline renders as nothing
while still costing a request per visible tile.

#### Scenario: Launching in a field camp

- **WHEN** the map loads and the machine reports no network
- **THEN** no remote tile source is added and local maps render as before

### Requirement: Opening a map reports a failure and stays retryable

Applying the active map SHALL read its metadata before changing what is on the
screen, SHALL report a read that failed rather than dropping it, and SHALL
count the map as applied only once the read has succeeded, so that a failure
does not block a later attempt at the same map.

An apply that another map's apply has overtaken SHALL NOT change the map.

#### Scenario: Metadata that cannot be read

- **WHEN** the metadata of the map being opened cannot be read
- **THEN** the operator is told, the map on screen is left alone, and opening the same map again tries again

#### Scenario: Switching maps during the read

- **WHEN** a second map is chosen while the first one's metadata is still being read
- **THEN** only the second map is applied
