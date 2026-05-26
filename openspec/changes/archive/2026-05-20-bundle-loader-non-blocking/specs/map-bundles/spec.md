## ADDED Requirements

### Requirement: Bundle catalog and cached maps remain interactive during an in-flight download

The system SHALL allow the user to interact with the bundle catalog and to open already-cached maps while another bundle's files are being downloaded. Only the specific map row whose file is being fetched right now SHALL remain non-interactive.

A request to switch to a different project while a download is in progress SHALL be honored by cancelling the in-flight download and starting the requested one; already-downloaded files for the cancelled bundle SHALL remain on disk so the user can resume later by re-selecting that project.

#### Scenario: Selecting a different project mid-download

- **WHEN** a download for project A is in progress AND the user clicks project B in the bundle catalog
- **THEN** the download for A is cancelled, B starts downloading, and the partially-downloaded files for A remain on disk

#### Scenario: Opening a cached map mid-download

- **WHEN** a download is in progress for any bundle AND the user clicks a map row that is marked `cached` (already on disk)
- **THEN** that map opens in the workspace, the workspace becomes the active surface, and the original download continues unaffected

#### Scenario: Currently-fetching map row stays disabled

- **WHEN** a download is in progress and one of the maps inside the active bundle is the file being fetched right now
- **THEN** that specific map row is non-interactive and shows a progress badge; other map rows in the same bundle that are already cached remain interactive

#### Scenario: Resuming a previously cancelled download

- **WHEN** the user re-selects a project whose download was cancelled earlier and whose partial files are still on disk
- **THEN** the download resumes by fetching only the files that are not yet on disk
