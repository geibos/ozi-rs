## ADDED Requirements

### Requirement: A bundle download reports its weight, not only its file count

When a bundle download scans the project it SHALL report the total size of the
files it is about to fetch, excluding files already on disk, and SHALL report
the bytes fetched so far as the download proceeds. When the listing states no
sizes the total SHALL be reported as unknown rather than as zero.

#### Scenario: Starting a bundle download

- **WHEN** the scan finds two files whose listing states 2 KiB and 1.5 MiB
- **THEN** the progress reports a total of those sizes together, and the panel shows fetched-of-total

#### Scenario: Resuming a download

- **WHEN** part of the bundle is already on disk
- **THEN** the announced total covers only what still has to be fetched

### Requirement: The progress panel stays visible over the bundle loader

The download progress panel SHALL remain visible when the bundle loader is
opened over the workspace, so that queueing another map does not hide the
download already running.

#### Scenario: Opening the loader during a download

- **WHEN** a bundle download is running and the operator opens the bundle loader
- **THEN** the progress panel is still visible above it
