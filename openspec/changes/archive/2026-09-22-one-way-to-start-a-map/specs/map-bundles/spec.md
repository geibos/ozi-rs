## ADDED Requirements

### Requirement: A map fetched in order to open it is visible wherever it was asked for

Every surface that can ask for a map SHALL, when opening it requires
downloading it first, show that download's progress and offer to cancel it, and
SHALL NOT act as though the map were open. The surfaces are the bundle loader,
the Library Maps tab and the command palette.

#### Scenario: Asking from the command palette for a map that is not on disk

- **WHEN** the operator opens a map from the palette's recent files and the map's bytes are not on disk
- **THEN** the download progress panel appears with a cancel, and the palette does not record the map as opened or navigate to it

#### Scenario: The map is already on disk

- **WHEN** the requested map is on disk
- **THEN** it opens directly and no progress panel appears

#### Scenario: The download ends

- **WHEN** the download that was started this way finishes
- **THEN** the progress panel is released, as for any other download
