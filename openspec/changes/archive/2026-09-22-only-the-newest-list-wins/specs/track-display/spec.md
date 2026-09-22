## ADDED Requirements

### Requirement: Only the newest track-list reload may be shown

The track list SHALL show the result of the most recently started reload only,
and SHALL discard an earlier reload's rows and an earlier reload's failure
alike, so that the list never goes backwards. Reloads overlap because the list
reloads on every application-state change.

#### Scenario: A reload overtaken while a bundle downloads

- **WHEN** a track-list reload is still running and a newer one completes first
- **THEN** the rows shown are the newer reload's, and the older one changes nothing when it finishes
