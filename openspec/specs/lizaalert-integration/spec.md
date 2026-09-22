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

The frontend SHALL merge each incoming chunk into its working project list with upsert-by-slug semantics: an entry whose `slug` is already present SHALL replace the existing entry in place; an entry whose `slug` is new SHALL be appended at the end of the list.

An entry absent from a refresh SHALL be removed when, and only when, that refresh walked the whole listing. A refresh that was stopped read only a prefix of the catalogue and SHALL remove nothing. Rows sent from the cache before the walk began SHALL NOT count as evidence that a project still exists.

This replaces the previous rule, under which an entry absent from a refresh was never removed and stayed reachable until the operator cleared the cache by hand. That rule was written when there was no way to tell a complete refresh from an interrupted one; there is now, so a search taken down upstream no longer stays in the list and in the cache for good, offered to a crew and failing when opened.

#### Scenario: Streaming project list

- **WHEN** the frontend invokes the project list refresh
- **THEN** the backend emits one or more `projects-chunk` events carrying partial project summaries, and the UI appends each chunk to the displayed list

#### Scenario: Refresh updates an existing entry in place

- **WHEN** a `projects-chunk` payload contains an entry whose `slug` already exists in the working project list AND whose `name` differs from the cached value
- **THEN** the existing entry's `name` is updated in place and its position in the list is preserved

#### Scenario: A search is taken down between two refreshes

- **WHEN** a complete catalogue walk does not list a project the previous walk listed
- **THEN** that project is no longer in the list or in the cache

#### Scenario: The operator stops the refresh

- **WHEN** a catalogue walk is stopped partway
- **THEN** no project is removed from the list

#### Scenario: The cached rows sent before the walk

- **WHEN** a refresh emits the cached catalogue before walking
- **THEN** those rows alone do not count as evidence that a project still exists

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

The system SHALL also show that timestamp to the operator beside the project list, and within the notice shown when a refresh has failed, so that a saved list can be told apart from a current one. It SHALL be shown as a calendar date and a clock time rather than an elapsed interval. A timestamp that cannot be read as a date SHALL be shown as nothing at all.

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

#### Scenario: Reading a saved list offline

- **WHEN** the refresh has failed and the operator is reading the cached list
- **THEN** the notice names the date and time that list was written, so a search published since then can be recognised as missing rather than as non-existent

#### Scenario: An unreadable timestamp

- **WHEN** the stored timestamp cannot be parsed as a date
- **THEN** nothing is shown in its place, rather than a placeholder that looks like a date

### Requirement: Catalog cache merges refresh deltas without dropping known entries

The local catalog cache SHALL hold what the last complete refresh listed, together with entries seen since that no refresh has contradicted. A refresh that was stopped SHALL NOT remove anything from the cache, because it read only a prefix of the listing. A refresh that walked the whole listing SHALL remove the entries it did not reference.

This replaces the previous rule, under which the cache accumulated the union of everything ever observed and was therefore a superset of any single refresh. That rule kept a crew's list intact across an interrupted refresh, which was the right trade when a complete refresh could not be distinguished from an interrupted one. It also meant that a search taken down upstream was never removed: it stayed listed, stayed cached, and failed when opened, with no way to clear it short of deleting the cache by hand.

#### Scenario: A stopped refresh keeps the cached entries it did not reach

- **WHEN** the operator has a cached catalogue of 200 entries AND stops a refresh after it has listed 195 of them
- **THEN** the cache still contains 200 entries and all 200 stay listed

#### Scenario: A complete refresh drops what it did not list

- **WHEN** the operator has a cached catalogue of 200 entries AND a refresh walks the whole listing and returns 195 of them
- **THEN** the five entries the listing no longer carries are removed from the list and from the cache

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

### Requirement: The catalogue refresh can be stopped

The catalogue refresh SHALL be stoppable by the operator while it runs, and the
interface SHALL offer that control for as long as a refresh is in flight and
not otherwise. The projects already read SHALL remain listed after a stop.

Stopping SHALL release the busy flag exactly as completion does, so that a
bundle download can start immediately afterwards.

#### Scenario: A crew needs a bundle before the listing finishes

- **WHEN** the operator stops the refresh while it is walking the listing
- **THEN** the walk stops without requesting further pages, the projects already read stay in the list, and a bundle download can be started

#### Scenario: No refresh is running

- **WHEN** no catalogue refresh is in flight
- **THEN** no stop control is offered

### Requirement: A stopped refresh is not passed off as the whole catalogue

A refresh that was stopped SHALL NOT replace the cached catalogue, because it
read only its first pages. The system SHALL report a stopped refresh
distinguishably from a completed one rather than reporting a count as if it
were the total.

#### Scenario: Stopping a refresh and reopening the application offline

- **WHEN** a refresh is stopped partway and the application is later started without a network
- **THEN** the catalogue shown is the last complete one, not the fragment the stopped refresh read

#### Scenario: Reporting the outcome

- **WHEN** a refresh is stopped partway
- **THEN** the status says the refresh was stopped at that many projects, not that that many were loaded

### Requirement: Bundle progress is reported in the interface's language

Progress reported while a bundle is being opened SHALL reach the interface as a
translation key with its arguments, not as a finished sentence, so that it can
be shown in the language the interface is set to. The phase word shown beside
it SHALL be translated on the same terms.

The backend SHALL also send the English wording, and the interface SHALL show
that wording when it has no translation for the key, so that an untranslated
message degrades to English rather than to a key.

#### Scenario: A download watched in a Russian window

- **WHEN** a bundle download reports its progress and the interface is Russian
- **THEN** the message and the phase beside it are in Russian

#### Scenario: A message the interface does not know

- **WHEN** progress arrives with a key the dictionaries do not define
- **THEN** the backend's own wording is shown, and the key is not

#### Scenario: A bundle name that looks like a placeholder

- **WHEN** a message argument itself contains placeholder-shaped text
- **THEN** it appears in the message unchanged

### Requirement: The catalogue refresh says how far it has got

While a catalogue refresh runs, the interface SHALL state how many projects are
already listed, so that the decision to stop the refresh can be made on what is
there rather than on elapsed time alone.

#### Scenario: A crew watching the refresh

- **WHEN** a catalogue refresh is running and projects have arrived
- **THEN** the refreshing hint states how many are listed so far

### Requirement: The project catalogue can be walked from the keyboard

The project list SHALL be operable from the keyboard without a pointer: it
SHALL expose itself as a listbox that takes focus and names the row the
keyboard is on, SHALL move that position with the arrow keys, by a screenful
with Page Up and Page Down and to either end with Home and End, and SHALL open
the row it is on when Enter is pressed.

The position SHALL be held against the full filtered list rather than against
the rows currently rendered, because the list is virtualized and most rows do
not exist in the document. It SHALL NOT move past either end, the list SHALL
scroll to keep it visible, and it SHALL be marked distinctly from the selected
row.

Narrowing the list SHALL clear the position.

#### Scenario: Finding a search without the pointer

- **WHEN** the operator types part of a name, presses Down and presses Enter
- **THEN** the first matching project is opened

#### Scenario: Reaching the end of the catalogue

- **WHEN** the operator presses End and then Down
- **THEN** the position is on the last project and stays there

#### Scenario: Narrowing the list after moving

- **WHEN** the operator has moved the position and then changes the filter
- **THEN** no row is pointed at until the keyboard is used again

### Requirement: The application state snapshot does not carry the catalogue

The application state snapshot SHALL NOT include the project catalogue. It is
fetched on every state change — once per file during a bundle download — and
the catalogue is thousands of rows, so carrying it there costs that payload on
every such change.

Building the snapshot SHALL NOT read the bundles directory, which was needed
only to mark catalogue rows.

The catalogue SHALL reach the interface as its own stream, and the interface
SHALL seed itself from its persisted cache, so that a cold start renders the
previous catalogue without waiting for either.

#### Scenario: A bundle download in progress

- **WHEN** a bundle download emits a state change for each file it finishes
- **THEN** no part of the catalogue is transferred with those state changes

#### Scenario: A cold start with a cached catalogue

- **WHEN** the application starts with a previously cached catalogue
- **THEN** the project list renders from the cache before any catalogue request completes

### Requirement: A catalogue link opens the search it names

A link to a project in the online catalogue SHALL open that project when the
operator pastes it, without their having to find the search by name. A link is
how a search reaches a crew, and its name is a transliteration they would
otherwise have to retype exactly.

Parsing SHALL tolerate what passing through a messenger does to a link — an
absent scheme, a missing or extra trailing slash, appended query parameters, a
percent-encoded name — and SHALL require the catalogue's own host, so that a
lookalike address is not treated as one.

Text that is not a catalogue link SHALL be treated as ordinary input rather
than guessed at.

A link naming a project the catalogue has not listed SHALL be reported as such,
naming it, rather than silently doing nothing.

#### Scenario: A link sent over a messenger

- **WHEN** the operator pastes a catalogue link for a listed project
- **THEN** that project is opened and identified by its name

#### Scenario: A link for a search not in the list yet

- **WHEN** the operator pastes a catalogue link naming a project the catalogue has not listed
- **THEN** they are told, and the name in the link is shown

#### Scenario: Ordinary text

- **WHEN** the operator types something that is not a catalogue link
- **THEN** no project is opened by it

### Requirement: The catalogue filter accepts the crew's own language

Every surface that searches the catalogue SHALL match a query written in
Russian against project names written in latin transliteration. Because no single transliteration is in
use, a Cyrillic letter SHALL match any of the latin spellings in common use for
it, and SHALL also match itself, so an entry written in Cyrillic is still
found. A query containing no Cyrillic SHALL behave exactly as before, matching
the name or the slug as a literal substring. A space in the query SHALL match
whichever separator the catalogue uses between words.

#### Scenario: Typing the Russian name of the search

- **WHEN** the catalogue holds a project named `2026 09 20 Schuvalovo` AND the operator types `Шувалово`
- **THEN** that project is listed, and the match count reflects it

#### Scenario: The same letter spelled two ways

- **WHEN** the catalogue holds both `Shuvalovo` and `Schuvalovo` AND the operator types `Шувалово`
- **THEN** both are listed

#### Scenario: A query that matches nothing

- **WHEN** the operator types a Russian name no project carries
- **THEN** the list is empty and the count says so, rather than falling back to everything

#### Scenario: A latin query is unchanged

- **WHEN** the operator types `Sagra`
- **THEN** the result is the same list the literal substring match produced before this change

#### Scenario: The command palette answers the same as the loader

- **WHEN** the same Russian query is typed into the command palette's project search and into the loader's filter
- **THEN** both list the same projects, subject to the palette's own screenful limit

### Requirement: A catalogue refresh does not block a bundle download

A running catalogue refresh SHALL NOT prevent a bundle from being downloaded or
opened from disk, and a running bundle operation SHALL NOT prevent the
catalogue from being refreshed. Each SHALL still refuse a second instance of
itself, and SHALL say which of the two is in the way. Completing one SHALL NOT
clear the other's in-progress state.

#### Scenario: Downloading during the launch-time refresh

- **WHEN** the catalogue walk started at launch is still running AND the operator asks to open a bundle
- **THEN** the download starts, rather than being refused until the walk ends or is stopped

#### Scenario: Refreshing during a download

- **WHEN** a bundle download is in flight AND the operator asks to refresh the catalogue
- **THEN** the refresh starts

#### Scenario: A second bundle

- **WHEN** a bundle is already being downloaded or opened AND another is asked for
- **THEN** it is refused, and the refusal names a bundle operation rather than the project list

#### Scenario: The walk ends while a download runs

- **WHEN** the catalogue walk finishes or is stopped while a download is in flight
- **THEN** the download keeps its progress and its own in-progress state

### Requirement: Two files of one bundle never share a partial path

A partial download SHALL be written to a path formed by appending a suffix to
the whole file name, so that two files of one bundle differing only in
extension never write to the same partial file.

#### Scenario: A bundle holding sheet.map and sheet.ozf2

- **WHEN** both files of that bundle are downloading at once
- **THEN** each writes to its own partial file and both land intact

#### Scenario: A partial path keeps what the file is

- **WHEN** a partial path is formed for `sheet.ozf2`
- **THEN** the original extension is still part of the name

### Requirement: A ready bundle file belongs to the map whose name it is

A file reported ready inside a bundle SHALL be matched to a map package by the
last component of its path, compared whole, rather than by whether the path
ends with the package's file name.

#### Scenario: A bundle holding map.ozf2 and bigmap.ozf2

- **WHEN** `bigmap.ozf2` finishes downloading
- **THEN** only `bigmap.ozf2` is marked available, and `map.ozf2` still needs downloading

### Requirement: A finished catalogue walk does not take the status line from a download

While a download is running, a catalogue walk that finishes SHALL report its
result to the diagnostics log without replacing the status line.

#### Scenario: The launch-time walk finishes mid-download

- **WHEN** the catalogue walk completes while a bundle is downloading
- **THEN** the status line still reports the download, and the walk's result is in the diagnostics log

#### Scenario: Nothing is downloading

- **WHEN** the catalogue walk completes with no download running
- **THEN** the status line reports how many projects were loaded

### Requirement: A cold launch with no link does not walk the catalogue

When the machine reports that it has no network at all, the application SHALL
NOT start the launch-time catalogue walk, and SHALL present the saved list as
saved rather than as the result of a refresh that failed.

The operator's own request to refresh SHALL run regardless: they can see the
state of the link better than the machine reports it.

#### Scenario: Launching in a field camp

- **WHEN** the application starts and the machine reports no network
- **THEN** no catalogue request is made and the catalogue is shown as the saved list with its age

