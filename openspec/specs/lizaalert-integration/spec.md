# lizaalert-integration Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
### Requirement: System fetches the LizaAlert project list as a stream

The system SHALL fetch the list of available projects from `maps.lizaalert.ru` and SHALL deliver results to the frontend in chunks via a `projects-chunk` event so the UI can render progressively.

#### Scenario: Streaming project list

- **WHEN** the frontend invokes the project list refresh
- **THEN** the backend emits one or more `projects-chunk` events carrying partial project summaries, and the UI appends each chunk to the displayed list

### Requirement: User can download a LizaAlert project bundle

The system SHALL allow the user to select a LizaAlert project from the streamed list and download the associated bundle into the configured bundles root.

#### Scenario: Download a project bundle

- **WHEN** the user selects a LizaAlert project and triggers download
- **THEN** the backend downloads the bundle archive, extracts it under the configured bundles root, and makes the bundle available as if opened locally

### Requirement: Download progress is observable

The system SHALL emit `download-progress` events carrying `package_name`, `downloaded_bytes`, and optionally `total_bytes` during bundle downloads, and SHALL emit `bundle-progress` events carrying a `phase` identifier during bundle extraction.

#### Scenario: Download in progress

- **WHEN** a bundle download is active
- **THEN** the UI receives periodic `download-progress` events suitable for rendering a progress indicator

#### Scenario: Extraction phase

- **WHEN** the downloaded bundle is being extracted
- **THEN** the UI receives `bundle-progress` events whose `phase` field reflects the current extraction step

### Requirement: Failed downloads degrade gracefully

The system SHALL surface download or extraction failures as user-facing errors, SHALL NOT panic, and SHALL NOT leave the bundles root in a partially-extracted unusable state.

#### Scenario: Network failure mid-download

- **WHEN** a download is interrupted by a network error
- **THEN** the system reports the failure to the user, leaves the bundle in either a fully-extracted or fully-removed state, and remains usable for retry

