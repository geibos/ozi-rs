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

The system SHALL emit `download-progress` events carrying `package_name`, `downloaded_bytes`, optional `total_bytes`, `file_index`, `file_count`, and the originating `download_id` during bundle downloads, and SHALL emit `bundle-progress` events carrying a `phase` identifier during bundle extraction. For multi-file bundle downloads the system SHALL additionally:

- schedule files by their leading numeric prefix (`00-`, `10-`, `20-`, …) so smaller-prefix files start first;
- run downloads concurrently with a bounded worker pool (default ≥3) while still respecting the prefix order at scheduling time;
- emit one `bundle-file-ready` event per file as soon as that file is fully written and fsync'd to its final path within the bundle root, allowing the bundle to become incrementally usable;
- never block the Tauri command thread or the Svelte main thread for the duration of the download — the initiating command SHALL return a `download_id` immediately and progress / readiness / phase events SHALL be observable while the download proceeds in the background.

#### Scenario: Download in progress

- **WHEN** a bundle download is active
- **THEN** the UI receives periodic `download-progress` events suitable for rendering a progress indicator

#### Scenario: Extraction phase

- **WHEN** the downloaded bundle is being extracted
- **THEN** the UI receives `bundle-progress` events whose `phase` field reflects the current extraction step

#### Scenario: Multi-file project download emits per-file progress

- **WHEN** the user downloads a LizaAlert project containing five remote files (`00-manifest.json`, `10-Tracks/a.ozf2`, `10-Tracks/b.ozf2`, `20-overlay.zip`, `99-refs.pdf`)
- **THEN** the system emits at least one `download-progress` event per file with `package_name` set to the file path relative to the project root, `downloaded_bytes` monotonically increasing per file, and `file_index`/`file_count` indicating position in the bundle; the UI displays the name of the currently downloading file together with its index

#### Scenario: Prefix-ordered scheduling

- **WHEN** the bundle contains files whose names begin with numeric prefixes (`00-`, `10-`, `99-`)
- **THEN** the first file the system begins downloading is the lowest-prefix file regardless of remote server response order; files lacking a numeric prefix are scheduled after all prefixed files in lexicographic order

#### Scenario: Incremental bundle availability via `bundle-file-ready`

- **WHEN** a small map file (e.g. `10-Tracks/a.ozf2`) finishes downloading while a large reference file (e.g. `99-refs.pdf`) is still in flight
- **THEN** the system emits a `bundle-file-ready` event for the small file before the large file completes; the user can open the bundle and use the already-downloaded map without waiting for the full bundle to finish; missing files are surfaced as "still downloading" rather than as errors

#### Scenario: UI remains responsive during long downloads

- **WHEN** the user triggers a multi-file bundle download and immediately switches panels, opens Settings, or interacts with already-loaded bundles
- **THEN** all UI interactions remain responsive without measurable jank attributable to the download; the `loadProject` invocation returns a `download_id` synchronously and the Svelte main thread does not await the long-running download promise

#### Scenario: Download cancellation preserves partial state

- **WHEN** the user cancels an in-flight bundle download via `cancel_download(download_id)`
- **THEN** in-flight HTTP requests are aborted within 250 ms; files that already finished downloading remain on disk in the bundle root; a subsequent `load_project` for the same bundle resumes by fetching only the files that are still missing

### Requirement: Failed downloads degrade gracefully

The system SHALL surface download or extraction failures as user-facing errors, SHALL NOT panic, and SHALL NOT leave the bundles root in a partially-extracted unusable state.

#### Scenario: Network failure mid-download

- **WHEN** a download is interrupted by a network error
- **THEN** the system reports the failure to the user, leaves the bundle in either a fully-extracted or fully-removed state, and remains usable for retry

