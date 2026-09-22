# map-bundles Specification

## Purpose
Covers the map bundle as a unit of storage separate from the project: what a bundle directory contains, where bundles live (the bundles root and its per-bundle subdirectories), how a local bundle is opened, how the active map inside a bundle is tracked and switched, and how the bundle catalog and per-map availability behave while a download is in flight. Fetching bundles from `maps.lizaalert.ru` is specified in `lizaalert-integration`; project files are specified in `project-persistence`.

### Decision history

- ADR-0002 (2026-03-29, accepted): map bundle (a directory of georeferenced rasters, downloaded or opened locally, shared by many projects) and project are distinct concepts, and the app keeps a configurable bundles root with one subdirectory per bundle; rationale: earlier versions lost track data when the map changed and had no home for several operations over the same area. Codified as: Map bundle is a directory containing one or more georeferenced raster maps; User can open a local bundle from a chosen directory; Bundles root directory is user-configurable; Active map is tracked per project and is switchable without unloading overlays; Bundles root defaults to Documents and holds one directory per bundle. Reality note: the root chosen via `set_bundles_root` is held in memory only and re-derived from the Documents folder at every start (`src-tauri/src/application/mod.rs:565-567`, `src-tauri/src/lib.rs:28-34`; `PersistedAppSession` has no such field), so the "persists across app restarts" clause of the user-configurable requirement is not met today.
- Legacy plan `docs/superpowers/plans/2026-04-12-production-bugs-fix.md` (executed): parallel per-file downloads and incremental map availability replaced the monolithic bundle load; rationale: users waited for whole multi-gigabyte bundles before any map could be opened. Codified as: Bundle catalog and cached maps remain interactive during an in-flight download; Per-map availability inside the active bundle streams into the UI live; Bundle download progress is observable from every surface that lists the downloading project's maps. Its "pre-created hidden bundle-loader window" (Task 5) is superseded: the loader is a component mounted on the `/` route (`src/routes/+page.svelte`), and `src/lib/windows.ts` no longer exists.
- Owner decision (2026-09-19): code is primary. The persistence promise in "Bundles root directory is user-configurable" stands and the in-memory-only implementation is fixed in `revive-ui-cycle` slice 0.3 (bundles root stored in the session file and restored at startup).
## Requirements
### Requirement: Map bundle is a directory containing one or more georeferenced raster maps

The system SHALL treat a map bundle as a filesystem directory that may contain SQLite MBTiles (`*.sqlitedb`) and/or OziExplorer raster maps (`*.map` paired with `*.ozf2`). A bundle MAY include a `10-Tracks/` subfolder for exported track files.

#### Scenario: Bundle directory layout

- **WHEN** the user opens a directory that contains a `*.sqlitedb` file
- **THEN** the system recognizes the directory as a map bundle and exposes the contained map(s) as candidate map layers

#### Scenario: Mixed-format bundle

- **WHEN** a bundle directory contains both an MBTiles file and an OZF2 + `.map` pair
- **THEN** the system exposes both maps as independent candidates and the user may select either as the active map

### Requirement: User can open a local bundle from a chosen directory

The system SHALL provide a UI affordance ("Maps…" / directory picker) that lets the user select a directory and load it as the active bundle without requiring network access.

#### Scenario: Local bundle open

- **WHEN** the user selects "Maps…" and picks a directory containing a recognized bundle layout
- **THEN** the available maps in that bundle become selectable, and any subsequently chosen active map renders in the map view

### Requirement: Bundles root directory is user-configurable

The system SHALL persist a user-configurable bundles root directory that LizaAlert downloads target by default and that the bundle browser uses as its initial path.

#### Scenario: Setting bundles root

- **WHEN** the user changes the bundles root directory in app settings
- **THEN** subsequent LizaAlert downloads place bundle data under the new root, and the new value persists across app restarts

### Requirement: Active map is tracked per project and is switchable without unloading overlays

The system SHALL remember which map within the active bundle is currently selected, and SHALL allow switching the active map without unloading project tracks or waypoints.

#### Scenario: Switching active map preserves overlays

- **WHEN** the user switches the active map from a Topo MBTiles map to an OZF2 Satellite map
- **THEN** all loaded tracks and waypoints remain present and rendered over the new base map

### Requirement: User can reveal active bundle in the OS file manager

The system SHALL expose a "Reveal bundle" action that opens the active bundle's directory in the host OS file manager (Finder on macOS, Explorer on Windows).

#### Scenario: Reveal active bundle

- **WHEN** the user invokes "Reveal bundle" with an active bundle loaded
- **THEN** the OS file manager opens at the bundle directory

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

### Requirement: Per-map availability inside the active bundle streams into the UI live

The system SHALL surface a map within the active bundle as "available" (clickable, badged as cached) within the same event tick the backend completes the download of that map's file. The frontend SHALL NOT wait for the whole bundle to finish before flipping a freshly-downloaded map row from "downloading" to "cached".

The backend SHALL emit `state-changed` at the same point it emits `bundle-file-ready` so that the next `getAppState()` snapshot exposes the updated `downloaded` flag on `LizaMapPackageDto` for the just-completed map. The flag's underlying source (the per-`LizaMapPackage` `local_path`) MUST already be set before that emission.

#### Scenario: Single map finishes mid-download

- **WHEN** a multi-file bundle download is in progress AND the file backing one map (e.g. `10-Tracks/topo-A.sqlitedb`) finishes downloading while another file is still in flight
- **THEN** within the same tick the backend emits `bundle-file-ready` for that file it also emits `state-changed`; the next `AppStateDto.current_project.maps[i].downloaded` for that map is `true`; the corresponding map row in the bundle loader transitions from the blue `%` progress badge to the green `cached` badge and becomes clickable

#### Scenario: Clicking a just-finished map mid-download opens it

- **WHEN** a multi-file bundle download is in progress AND a map row whose file completed earlier in the same download now shows the `cached` badge AND the user clicks that row
- **THEN** the map opens in the workspace via the existing `openSelectedMap` path; the ongoing download of the remaining files continues uninterrupted; subsequent files completing in that bundle continue to flip their rows live

#### Scenario: All maps inside one bundle finish before non-map files

- **WHEN** a bundle contains two map files and one reference PDF AND both map files finish downloading before the PDF
- **THEN** both map rows flip to `cached` and become clickable as each finishes; the user can open either map without waiting for the PDF; once the PDF completes the bundle download is fully done with no further row state changes

### Requirement: Bundle download progress is observable from every surface that lists the downloading project's maps

While a bundle download is in flight, every UI surface that lists the maps belonging to the downloading project SHALL render an in-progress indicator on the affected rows, sourced from the `downloadingMaps` and `downloadProgress` stores in `src/lib/stores.ts`. This includes at minimum: the `BundleLoader` component's maps column (existing behaviour, unchanged) AND the Library Rail Maps tab (`src/components/library/MapsTab.svelte`, newly wired by this change).

The progress data SHALL be sourced from the existing layout-level listeners on `download-progress` and `bundle-progress` events (`src/routes/+layout.svelte`). No surface SHALL register its own `listen()` for either event type.

#### Scenario: Library Maps tab mirrors BundleLoader progress

- **WHEN** a bundle download is in progress for the current project AND the user has the Library Maps tab visible AND opens the bundle loader Sheet side-by-side
- **THEN** each map row in the Library Maps tab shows the same in-progress indicator as the corresponding row inside the bundle loader's maps column, both reflecting the same `downloadProgress` payload within one animation frame of each event arrival

#### Scenario: Status bar mirrors bundle-loader progress text

- **WHEN** a bundle download is in progress
- **THEN** the workspace status bar shows the same `bundleProgress` text that the bundle loader's status region shows, both sourced from the same `bundleProgress` store

#### Scenario: Per-map progress disappears when the map is cached

- **WHEN** a single file inside the bundle completes AND the backend emits `bundle-file-ready` AND `state-changed` AND `appState.refresh()` flips `currentProject.maps[i].downloaded` to true for that map
- **THEN** the in-progress indicator on the affected row is replaced by the `cached` badge AND the row no longer reads from `downloadProgress`

#### Scenario: Progress wiring does not duplicate event subscriptions

- **WHEN** the static count of `listen("download-progress", ...)`, `listen("bundle-progress", ...)`, and `listen("bundle-file-ready", ...)` registrations in the `src/` tree is taken
- **THEN** each event name appears exactly once across the entire frontend, in `src/routes/+layout.svelte`

### Requirement: A refused bundle download says why

When the system cannot start opening a bundle it SHALL report the reason rather
than returning as if it had started. A refusal because another bundle operation
is still running SHALL be distinguishable from a refusal because the project is
unknown, and the loader SHALL tell the operator which it was. While the
application is busy, the control that starts a download SHALL be disabled and
SHALL state that it is waiting.

#### Scenario: Pressing download during the catalogue refresh

- **WHEN** the operator asks to open a bundle while the project list is still loading
- **THEN** the request is refused with a "still loading" reason and the operator is told to wait

#### Scenario: Asking for a project that is not in the list

- **WHEN** the requested project slug is absent from the catalogue
- **THEN** the request is refused as an unknown project rather than silently doing nothing

### Requirement: Loader failures are visible

The bundle loader SHALL surface a failure to preview a project, to start a
bundle download, or to open a local bundle, instead of discarding it.

#### Scenario: The project listing cannot be read

- **WHEN** previewing a project fails
- **THEN** the operator sees an error naming the failure, not an empty map list

### Requirement: The progress panel belongs to the running download

The bundle progress panel SHALL be shown for the download that is running and
SHALL be released when it ends, so that a later unrelated operation does not
bring back a finished download's progress.

#### Scenario: Refreshing the project list after a download finished

- **WHEN** a bundle download has completed and the operator later refreshes the project list
- **THEN** no progress panel for the completed download is shown

### Requirement: Opening a map closes the loader

Opening a map SHALL clear the request to show the bundle loader, so the loader
does not reopen over the map that was just opened.

#### Scenario: Switching maps from the Maps tab

- **WHEN** the operator opens the loader from the Maps tab and then opens a map
- **THEN** the workspace shows that map with the loader closed

### Requirement: A downloaded bundle is recognised wherever its files sit

The system SHALL find a bundle's cached `.sqlitedb` maps anywhere inside that
bundle's directory, and SHALL recognise the bundle's coordinates file by the
same pattern the online listing is matched by.

#### Scenario: Maps stored outside the conventional folder

- **WHEN** a downloaded bundle keeps its `.sqlitedb` files in a subdirectory other than the conventional one
- **THEN** those maps are reported as downloaded and are opened from disk rather than fetched again

#### Scenario: A differently named coordinates file

- **WHEN** a cached bundle's coordinates file is not named exactly `2-Coordinates.txt`
- **THEN** the bundle still counts as cached and opens offline

### Requirement: The project list says which bundles are on disk

Each project summary SHALL carry whether that bundle is already downloaded and
openable without a network, and the project list SHALL mark those projects. The
list SHALL offer a way to show only them. A summary restored from a stale local
cache SHALL default to "not downloaded" rather than claiming availability.

#### Scenario: Choosing a bundle with no signal

- **WHEN** the operator opens the project list offline and asks for downloaded bundles only
- **THEN** only the bundles present on disk are listed, each marked as downloaded

#### Scenario: A cache written before the flag existed

- **WHEN** the catalogue is restored from a local cache whose entries carry no availability flag
- **THEN** those projects are treated as not downloaded until the backend reports otherwise

### Requirement: The project filter matches what the operator types

The project filter SHALL match the query against both the project name and its
slug, and the list SHALL report how many projects are shown out of the total.

#### Scenario: Searching by date

- **WHEN** the operator types a date as it appears in the slug, such as `2026-09-21`
- **THEN** the projects of that date are listed, although their display names spell the date differently

#### Scenario: Reading how much the filter narrowed

- **WHEN** a query is active
- **THEN** the list reports the number shown out of the catalogue total

### Requirement: A single-map download is visible and can be stopped

Downloading one map package SHALL report a download identifier to the caller,
SHALL show the same progress panel a whole-bundle download shows, and SHALL be
cancellable. Both download paths SHALL announce when they stop, and a failure
SHALL be reported to the operator rather than only ending the progress display.

#### Scenario: Opening a map that is not on disk

- **WHEN** the operator opens a map package that has not been downloaded
- **THEN** the progress panel shows that download and offers to cancel it

#### Scenario: Cancelling a map download

- **WHEN** the operator cancels a running single-map download
- **THEN** the download stops, the partial file is removed, and the panel closes

#### Scenario: A download that fails

- **WHEN** a download ends in failure
- **THEN** the operator is told it failed, rather than only seeing the panel disappear

#### Scenario: Two downloads in flight

- **WHEN** one download finishes while another is still running
- **THEN** the panel keeps showing the running one

### Requirement: A map's download size is known before the download starts

A map package SHALL carry its size in bytes when that is known — read from the
listing for a remote map and from the file for a cached one — and the screens
that offer a map for opening SHALL show it. An unknown size SHALL be shown as
unknown rather than as zero.

#### Scenario: Choosing between a topo and a satellite layer

- **WHEN** a project lists a 16 MiB topo map and a 185 MiB satellite map
- **THEN** each row states its size, so the operator can pick what the link can carry

#### Scenario: A listing without a size column

- **WHEN** the listing states no size for a file
- **THEN** that map's size is reported as unknown and no size is shown for it

### Requirement: Sizes are stated in the application's language

Byte sizes SHALL be formatted with the unit names of the active language, using
binary units as the source listing does, and the same rule SHALL apply wherever
a size is shown.

#### Scenario: Reading a size in Russian

- **WHEN** the application is running in Russian and a file is 185.1 mebibytes
- **THEN** it reads `185.1 МиБ`, in the map list and in the download panel alike

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

### Requirement: The operator chooses what a bundle download fetches

A previewed bundle SHALL report the entries at the top level of its directory,
with the size the listing states where it states one. The loader SHALL present
them for selection with everything selected, and the download SHALL omit the
entries the operator cleared. With nothing cleared the download SHALL fetch the
whole bundle, as before.

#### Scenario: Leaving the print sheets on the server

- **WHEN** the operator clears the print-maps folder and starts the download
- **THEN** every other entry is fetched and nothing under that folder is

#### Scenario: Changing which project is previewed

- **WHEN** the operator clears an entry and then previews a different project
- **THEN** the choice resets, because a different bundle has different contents

#### Scenario: An offline bundle

- **WHEN** a cached bundle is opened without a network
- **THEN** its contents are reported from disk rather than from a listing

### Requirement: Only the most recently requested project preview may land

The system SHALL apply only the preview result belonging to the project
currently selected, and SHALL discard any earlier preview's result — whether it
succeeded or failed — rather than letting it replace the shown map list or
report its outcome. Previewing fetches the map list asynchronously and the
operator may start another preview before the first answers.

A preview SHALL NOT acquire the busy flag, so that a click during the catalogue
walk is not ignored, and SHALL NOT release it, so that a preview finishing
during a download does not let a second download start.

#### Scenario: Clicking through several projects in a row

- **WHEN** the operator previews one project and then previews another before the first answers
- **AND** the first project's map list arrives last
- **THEN** the shown map list is the second project's, and the first result is discarded

#### Scenario: An abandoned preview fails

- **WHEN** a preview the operator has moved on from fails
- **THEN** no failure is reported for it

#### Scenario: A preview finishes while a bundle download is running

- **WHEN** a preview started before a download completes during that download
- **THEN** the application is still busy and no second download may start

### Requirement: The loader identifies an arriving preview by slug

The loader SHALL decide that a requested preview has arrived by matching the
project's slug and not its display name, because display names are not unique
in the catalogue.

While a preview is outstanding the loader SHALL keep showing that it is
waiting. A timer SHALL NOT end the wait; after a long wait the loader SHALL say
the wait is running long, and the wait SHALL end only when the map list arrives
or the request fails.

#### Scenario: Two catalogue entries share a display name

- **WHEN** the operator previews a project whose display name matches another entry's
- **THEN** the wait ends only when that project's own slug arrives

#### Scenario: The server is slow to answer

- **WHEN** a preview has been outstanding for longer than the loader's patience
- **THEN** the loader says the map list is still loading rather than showing it as arrived

### Requirement: The bundle loader keeps the operator's place while it is closed

The loader SHALL restore the operator's browsing position when it is reopened
within the same session, covering the catalogue filter, the "only downloaded"
restriction, the selected project, the bundle contents the operator has cleared
and the scroll offset in the project list. The position SHALL NOT outlive the
session, so that a later run starts on the current search rather than an old
one.

#### Scenario: Closing the loader to look at the map

- **WHEN** the operator has filtered the catalogue and selected a project, closes the loader, and opens it again
- **THEN** the filter, the selection and the place in the list are as they left them

#### Scenario: A fresh session

- **WHEN** the application is started and the loader is opened for the first time
- **THEN** the catalogue is unfiltered and nothing is selected

### Requirement: A dropped transfer is retried before the file is given up

A file within a bundle SHALL be fetched again after a failed transfer, up to a
bounded number of attempts, before the download fails. A bundle is fetched over
the link a crew has in the field, where a dropped connection is ordinary, and
one such file SHALL NOT cost them the whole bundle.

A cancelled transfer SHALL NOT be retried: cancellation is the operator's
instruction, not a transport failure.

A retry SHALL be reported, so that it is not mistaken for a stall.

A retry SHALL continue from what the failed attempt already wrote rather than
fetching the file again, unless the server does not honour the request for the
remainder, in which case the file SHALL be fetched again from the start. What
a failed attempt wrote SHALL be kept for the next one and SHALL be removed once
the file is given up on.

#### Scenario: A connection drops partway through a bundle

- **WHEN** one file's transfer fails and the next attempt succeeds
- **THEN** that file lands complete and the bundle download continues

#### Scenario: The operator cancels

- **WHEN** a transfer ends because the download was cancelled
- **THEN** it is not attempted again

#### Scenario: A drop near the end of a large map

- **WHEN** a transfer fails after most of a file has been written and the next attempt is made
- **THEN** the remainder is requested and the file is completed without fetching what had already arrived

#### Scenario: The file is given up on

- **WHEN** every attempt at a file has failed
- **THEN** nothing partial is left behind for it

#### Scenario: Watching a retry

- **WHEN** a file is being fetched again after a failure
- **THEN** the progress reported says so, naming the file and the attempt

### Requirement: The download progress panel stays readable while a toast is up

Transient notifications SHALL NOT obscure the bundle download progress panel.
While the panel is on screen, the notification viewport SHALL be positioned
clear of it, and SHALL return to its usual position once the panel is gone.
The clearance SHALL follow the panel's measured height, so that a panel listing
many files is cleared as fully as a panel listing one.

#### Scenario: A file fails while the bundle is still downloading

- **WHEN** a bundle download is in flight AND a notification reports that one file failed
- **THEN** the notification is drawn clear of the progress panel, and the panel's title, cancel control and file/byte counters stay visible

#### Scenario: Nothing is downloading

- **WHEN** no download is on screen AND a notification appears
- **THEN** it occupies its usual corner, at the notification library's own edge offset

#### Scenario: The download finishes while a notification is up

- **WHEN** the panel disappears because the download finished
- **THEN** the notification returns to its usual corner rather than staying lifted

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

### Requirement: The status bar is sized to what it has to say

The launch screen's status bar SHALL occupy only the space its content needs.
With no download on screen it SHALL show the status line alone; when a download
begins it SHALL grow to hold the file name, the progress and the byte counts,
and SHALL return to one line when the download ends. It SHALL NOT render an
empty progress track, which reads as a broken control rather than as reserved
space.

#### Scenario: The application has just been launched

- **WHEN** the operator reaches the first screen and nothing is downloading
- **THEN** the status bar is a single line and no empty progress track is drawn

#### Scenario: A download begins

- **WHEN** a bundle download starts
- **THEN** the bar grows to report it, and does not resize again when the first byte counts arrive

#### Scenario: The download ends

- **WHEN** it finishes
- **THEN** the bar returns to a single line

### Requirement: A map can be opened before its bundle finishes

A map whose file has landed during a bundle download SHALL be openable from
disk while the remaining files continue downloading, and the operator SHALL be
told when the first such map becomes available.

#### Scenario: The topo layer lands first

- **WHEN** a bundle download writes a map file and the rest of the bundle is still downloading
- **THEN** that map opens from disk without starting another download, and the operator is told it is ready

#### Scenario: Files that are not maps

- **WHEN** the file that lands is a print sheet, a coordinates file or an archive
- **THEN** nothing is announced, because none of them is something the crew can open

#### Scenario: One announcement per download

- **WHEN** several maps of the same bundle land in turn
- **THEN** the operator is told once, not once per file

