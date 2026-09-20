# lizaalert-integration Specification

## Purpose
Covers the integration with `maps.lizaalert.ru`: streaming the project catalog and caching it locally, downloading a project's bundle files concurrently with resume and cancellation, the progress and readiness events the frontend consumes, failure behavior, and the HTTP/TLS stack the client is built on. Where downloaded bundles live and how they are opened is specified in `map-bundles`.

### Decision history

- ADR-0008 (2026-03-28, accepted): use reqwest 0.13 with `default-features = false` and the `rustls` backend; rationale: the endpoint is HTTPS-only and the binary must build without OpenSSL or system TLS on every platform. Codified as: LizaAlert HTTP client uses reqwest with rustls, no native TLS. Partly superseded: the ADR's "blocking API only, no async runtime" no longer holds — tokio is a dependency (`src-tauri/Cargo.toml:33`) and bundle downloads run on the async client with the `stream` feature (`src-tauri/Cargo.toml:26`, `src-tauri/src/infrastructure/lizaalert.rs:543`), while the blocking client remains for listings (`lizaalert.rs:473`). The ADR's remark about the webpki trust store also no longer describes the build: the graph shows reqwest pulling `rustls-platform-verifier` (OS trust store) and no `webpki-roots`.
- Legacy plan `docs/superpowers/plans/2026-04-12-production-bugs-fix.md` (executed): bounded-concurrency parallel downloads, prefix-ordered scheduling, `completed`/`total` counts on `bundle-progress`; rationale: sequential downloads and a progress bar without data made large bundles unusable. Codified as: Download progress is observable (counts also covered by the `ui-shell` status-bar requirement). Retries were never implemented (no retry or backoff logic in `lizaalert.rs`) and are not codified.
- Code, no ADR (2026-05 to 2026-07): per-file `.part` write + rename, staged archive extraction, resume of missing files, and offline open of cached bundles; rationale: a cancelled or failed download must never leave a truncated file that the cached-map listing would treat as complete. Codified as: Failed downloads degrade gracefully (modified in this change).
## Requirements
### Requirement: System fetches the LizaAlert project list as a stream

The system SHALL fetch the list of available projects from `maps.lizaalert.ru` and SHALL deliver results to the frontend in chunks via a `projects-chunk` event so the UI can render progressively.

The frontend SHALL merge each incoming chunk into its working project list with upsert-by-slug semantics: an entry whose `slug` is already present SHALL replace the existing entry in place; an entry whose `slug` is new SHALL be appended at the end of the list. Entries already present in the working list but absent from a refresh SHALL NOT be removed by that refresh; they SHALL remain reachable from the bundle loader until the user explicitly clears the local catalog cache.

#### Scenario: Streaming project list

- **WHEN** the frontend invokes the project list refresh
- **THEN** the backend emits one or more `projects-chunk` events carrying partial project summaries, and the UI appends each chunk to the displayed list

#### Scenario: Refresh updates an existing entry in place

- **WHEN** a `projects-chunk` payload contains an entry whose `slug` already exists in the working project list AND whose `name` differs from the cached value
- **THEN** the existing entry's `name` is updated in place and its position in the list is preserved

#### Scenario: Refresh that omits a known slug keeps the entry

- **WHEN** a complete refresh finishes (every `projects-chunk` event has been processed and no further chunk has arrived for the debounce window described in the Cache write Requirement) AND the working list contains a `slug` that was not referenced by any chunk in that refresh
- **THEN** that `slug` remains in the working list and stays selectable in the bundle loader

### Requirement: User can download a LizaAlert project bundle

The system SHALL allow the user to select a LizaAlert project from the streamed list and download the associated bundle into the configured bundles root.

#### Scenario: Download a project bundle

- **WHEN** the user selects a LizaAlert project and triggers download
- **THEN** the backend downloads the bundle archive, extracts it under the configured bundles root, and makes the bundle available as if opened locally

### Requirement: Download progress is observable

The system SHALL emit `download-progress` events carrying `package_name`, `downloaded_bytes`, optional `total_bytes`, `file_index`, `file_count`, and the originating `download_id` during bundle downloads, and SHALL emit `bundle-progress` events carrying a `phase` identifier during bundle extraction. For multi-file bundle downloads the system SHALL additionally:

- schedule files by their leading numeric prefix (`00-`, `10-`, `20-`, …) so smaller-prefix files start first;
- run downloads concurrently with a bounded worker pool (default ≥6) while still respecting the prefix order at scheduling time;
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

### Requirement: LizaAlert project catalog is cached locally with a write timestamp

The system SHALL persist the merged LizaAlert project catalog to local browser storage with a write timestamp. The cache SHALL live under a versioned key (e.g. `liza:projects:v1`) and SHALL hold a JSON object `{ items: LizaProjectSummaryDto[], writtenAt: string }` where `writtenAt` is an ISO-8601 UTC timestamp. The system SHALL write the cache once per completed refresh, after the projectsStore stops changing for a short debounce window (~800 ms), not on every `projects-chunk` event.

The system SHALL hydrate the working catalog from this cache synchronously at frontend module initialization, before any component mounts. When the cache is absent, malformed, or fails shape validation, hydration SHALL silently fall back to an empty list — no toast, no error surface.

The system SHALL swallow storage write failures (e.g. `QuotaExceededError`) silently and log them only to the developer console; the cache is best-effort and SHALL NOT block any user-facing flow.

#### Scenario: Cache survives across app restarts

- **WHEN** the user successfully completes a project list refresh in session 1 AND restarts the application
- **THEN** session 2's bundle loader renders the cached catalog within the first paint, without waiting for any IPC

#### Scenario: Cache is written once per refresh, not per chunk

- **WHEN** a single refresh delivers five `projects-chunk` events
- **THEN** the system writes the cache exactly once — after the refresh has settled (no further chunk arrives for ~800 ms) — not five times

#### Scenario: Corrupt cache is tolerated

- **WHEN** the local storage value under the catalog cache key is missing, empty, or fails JSON parsing or shape validation
- **THEN** the working catalog initializes to an empty list, no error is surfaced to the user, and the next refresh proceeds normally and writes a fresh cache on completion

#### Scenario: Quota exceeded does not break the UI

- **WHEN** a cache write fails with `QuotaExceededError` or any other storage error
- **THEN** the error is logged to the developer console only, the user sees no toast, and the in-memory catalog remains usable for the rest of the session

### Requirement: Catalog cache merges refresh deltas without dropping known entries

The local catalog cache SHALL accumulate the union of entries the frontend has ever observed via `projects-chunk` events. A refresh that does not reference a previously-seen `slug` SHALL NOT remove that entry from the cache. The cache SHALL therefore be a superset of any single refresh's payload.

#### Scenario: Historical entries persist across refreshes

- **WHEN** the user has a cached catalog containing 200 entries AND triggers a refresh that returns only 195 of those 200 entries
- **THEN** after the refresh settles, the cache still contains 200 entries and the bundle loader still lists all 200; the five entries absent from the refresh are not removed

#### Scenario: New entries appear after refresh

- **WHEN** a refresh returns three previously-unseen entries among its chunks
- **THEN** those three entries are appended to the working list AND included in the next cache write

### Requirement: Library Maps tab rows reflect in-flight download progress per map

The Library Rail Maps tab (`src/components/library/MapsTab.svelte`) SHALL render an in-progress indicator on every map row whose package name is present in the `downloadingMaps` store. The indicator SHALL be derived from `downloadProgress.get(mapName)` so the row reflects the latest `download-progress` payload (downloaded_bytes / total_bytes or completed / total — whichever the payload provides).

When `downloadingMaps` no longer contains a map (download cancelled, completed, or never started) AND `m.downloaded` is true (per `currentProject.maps[].downloaded`), the row SHALL display the existing `cached` badge. When neither condition holds the row SHALL display its neutral state.

Switching between the three states (`in-progress` → `cached` → neutral) SHALL be driven by the existing stores fed by the layout-level listeners; the Maps tab SHALL NOT register any new `listen()` subscription.

#### Scenario: Map is currently downloading

- **WHEN** the user views the Library Maps tab AND `downloadingMaps.has(m.name)` is true AND `downloadProgress.get(m.name)` reports 47% progress
- **THEN** the row for `m.name` shows an in-progress indicator (badge, percentage, or progress bar) reflecting 47%

#### Scenario: Map transitions from downloading to cached

- **WHEN** a map's download completes AND the backend emits `bundle-file-ready` followed by `state-changed`, which causes `appState.refresh()` to flip `m.downloaded` to true AND removes the map from `downloadingMaps`
- **THEN** the row's indicator transitions from the in-progress display to the `cached` badge within one animation frame

#### Scenario: Maps tab does not duplicate event listeners

- **WHEN** the Maps tab mounts
- **THEN** the static count of `listen("download-progress", ...)` and `listen("bundle-progress", ...)` calls in the `src/` tree is unchanged from before this change (single owner remains `src/routes/+layout.svelte`)

#### Scenario: Selecting a project does not throw a JSON parse error

- **WHEN** the user clicks a project row in the Library Maps tab (or its parent Projects pane) AND the project's bundle is selected via `load_project`
- **THEN** no Sonner toast with `data-testid="ipc-error"` appears in a dev build (i.e. the IPC payload validates cleanly) AND the project becomes the current project

