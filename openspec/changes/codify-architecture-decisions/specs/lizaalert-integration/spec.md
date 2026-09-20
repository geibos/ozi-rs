## ADDED Requirements

### Requirement: LizaAlert HTTP client uses reqwest with rustls, no native TLS

All HTTP traffic to `maps.lizaalert.ru` SHALL go through `reqwest` built with `default-features = false` and the `rustls` TLS backend, so the binary carries no dependency on OpenSSL or the platform's native TLS library. The catalog root SHALL be addressed over HTTPS (`https://maps.lizaalert.ru/maps/`). Bundle file bodies SHALL be streamed to disk chunk by chunk with a progress callback per chunk; the system SHALL NOT buffer a whole file body in memory before writing it.

#### Scenario: Manifest declares the rustls backend

- **WHEN** the `reqwest` entry in `src-tauri/Cargo.toml` is inspected
- **THEN** it sets `default-features = false` and its feature list includes `rustls`

#### Scenario: No native TLS in the dependency graph

- **WHEN** `cargo tree -i native-tls` and `cargo tree -i openssl-sys` are run in `src-tauri`
- **THEN** both report that the package ID matches no package

#### Scenario: Catalog is fetched over HTTPS

- **WHEN** the project list refresh starts
- **THEN** the first request goes to `https://maps.lizaalert.ru/maps/` and every derived file URL keeps the `https://` scheme

#### Scenario: Large file is streamed, not buffered

- **WHEN** a map package of several hundred megabytes is downloaded
- **THEN** `download-progress` events with increasing `downloaded_bytes` arrive while the transfer is still running, and the bytes are appended to the on-disk temp file as they arrive

## MODIFIED Requirements

### Requirement: Failed downloads degrade gracefully

The system SHALL surface download or extraction failures as user-facing errors and SHALL NOT panic. Failure handling SHALL be atomic per file and per archive, not per bundle:

- a file being downloaded SHALL be written to a sibling temporary file whose extension is replaced by `.part` (`10-Tracks/b.ozf2` → `10-Tracks/b.part`) and renamed to its canonical path only after it is fully written and fsynced; a network error or cancellation mid-stream SHALL remove the `.part` file and leave nothing at the canonical path, so a file present at its canonical path is always complete;
- a cached OZI archive SHALL be extracted into a sibling staging directory and renamed into place only on success; a failed or interrupted extraction SHALL leave no destination directory, and the next open of the bundle SHALL retry the extraction;
- files that finished before the failure SHALL remain on disk, and re-selecting the same project SHALL resume by fetching only the missing files;
- one failing file SHALL NOT abort the other in-flight files of the same bundle; the first error is reported after the remaining workers finish, unless the user cancels.

When the remote listing is unreachable and the bundle is already cached (`<bundles root>/<slug>/2-Coordinates.txt` exists), the system SHALL open the cached bundle instead of reporting an error. When the listing is unreachable and the bundle is not cached, the error SHALL be reported and the application SHALL remain usable.

#### Scenario: Network failure mid-download

- **WHEN** a download is interrupted by a network error while `10-Tracks/b.ozf2` is in flight
- **THEN** the system reports the failure to the user, `10-Tracks/b.ozf2` does not exist at its canonical path, no `10-Tracks/b.part` remains, files completed earlier remain on disk, and re-selecting the project fetches only the missing files

#### Scenario: Interrupted archive extraction leaves no half-extracted directory

- **WHEN** extraction of a cached OZI archive fails part-way
- **THEN** no destination directory for that archive exists afterwards, the error is reported, and the next open of the same bundle extracts the archive again

#### Scenario: Cached bundle opens while offline

- **WHEN** the LizaAlert listing is unreachable AND the selected project's bundle directory already contains `2-Coordinates.txt`
- **THEN** the project opens from the cached files, the `bundle-progress` phase message reads `Opening cached project bundle: …`, and no error is shown

#### Scenario: Uncached bundle offline reports an error

- **WHEN** the LizaAlert listing is unreachable AND the selected project has no cached bundle directory
- **THEN** the system reports the connection error to the user and remains usable
