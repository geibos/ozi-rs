## Context

`src/routes/+page.svelte` is the bundle loader. Today every project row in the list and every map row in the right column carries `disabled={$busy}` (or `disabled={isDownloading}`), where `$busy = appState.busy`. The backend flips `appState.busy = true` for the lifetime of any LizaAlert download. The result: while bundle A is being fetched, the user can do nothing in the bundle loader except cancel.

The status bar (`src/routes/+page.svelte:296`) is `position: fixed; bottom: 0`. Its inner content is composed of multiple `{#if}` blocks (message line, current-file label, indeterminate bar, byte counters, progress bar, ready-files list, action buttons). Each download / progress / file-ready event toggles which subset of these is rendered, and the bar's height changes accordingly.

Two upstream constraints matter:
- The backend exposes exactly one `activeDownloadId` at a time (`stores.ts:86`); the IPC contract is single-download.
- Files that are already on disk (cached) belong to bundles separately from the in-flight one; opening them via `openSelectedMap` does not touch the download pipeline.

## Goals / Non-Goals

**Goals:**
- Stop blocking the user from interacting with cached data during a download.
- Stop the status bar from reflowing while a download progresses.
- Keep the change scoped to UI behavior — no new IPC commands, no new backend state.

**Non-Goals:**
- True parallel downloads of multiple bundles. The IPC contract stays single-bundle.
- Resumable downloads. Already covered by `cancelDownload` semantics in `api.ts:43`.
- Redesigning the bundle loader visual language (that belongs to a later vision-pass change).
- Fixing track / waypoint rendering or any non-bundle-loader fricton (those are separate changes B and C).

## Decisions

### Decision 1: Cross-bundle switch = cancel-and-restart, not queue

When a download for bundle A is in flight and the user clicks bundle B in the project list:

- Frontend SHALL call `cancelDownload(activeDownloadId)` and then `loadProject(B.slug)`.
- Already-downloaded files for A SHALL remain on disk; the user can come back to A later and download will skip them (existing backend behavior per `api.ts:38-41`).
- The status text SHALL briefly indicate the switch ("Switching to <B>…"), then become the normal progress text for B.

**Rationale**: Parallel downloads require backend state changes (`activeDownloadId` → set), event routing changes, and a new mental model for the user. Cancel-and-restart costs ~1 IPC call and reuses partial state. It is the smallest possible change that removes the user-visible block.

**Alternatives considered**:
- _Queue switches_: Implies a second download has a "pending" status, which the UI doesn't currently model. Adds state without removing user-perceived friction (the user wanted B, queuing it behind A is not what they asked for).
- _Concurrent downloads_: Out of scope (see Non-Goals).

### Decision 2: Per-row disable, not blanket disable

`disabled={$busy}` on project rows is removed. Replaced by per-row checks driven by what the row actually does:

- **Project rows**: never disabled. Clicking a different project triggers cancel-and-restart (Decision 1).
- **Map rows (cached, `m.downloaded === true`)**: never disabled. Click opens the map via `openSelectedMap`.
- **Map rows (downloading right now, the active file in the active bundle)**: remain disabled, with the existing progress badge as the affordance.
- **Refresh button (`↻`)**: stays `disabled={$busy}` because re-running `loadProjects()` while one is in flight is the duplicate-call we actually want to prevent.

### Decision 3: Status bar uses a reserved-slot layout

The status bar SHALL be a CSS grid with named slots: `status-line`, `progress`, `ready-list`, `actions`. Slot heights SHALL be reserved (`min-height` on each row, fixed total height for the bar) so that `{#if}` toggles inside slots only change content visibility, not the bar's outer dimensions.

- The `ready-files` list keeps its existing `overflow-y: auto` with `max-height: 80px`. Its slot in the grid reserves that 80px.
- The progress bar slot stays at a fixed height even when no progress is being reported.
- Total status-bar height SHALL be a single CSS value used both for the bar itself and for the bottom padding the main grid leaves for it.

**Rationale**: This is purely a CSS change. No JS, no reactivity tweak. The "jumping" is a layout reflow problem, not a render-frequency problem.

**Alternatives considered**:
- _Hide the bar when not busy and absolute-fade in_: still reflows on every toggle inside, doesn't solve the root cause.
- _Throttle progress events_: orthogonal — addresses event rate, not layout. Belongs to change C, not here.

## Risks / Trade-offs

- **Risk**: User clicks project B by accident while A is half-downloaded, losing time. **Mitigation**: A's partial files survive on disk, so resuming costs only the missing pieces. No data is lost. Cancel-and-restart is the expected SAR-volunteer behavior — they switch missions, not multitask downloads.
- **Risk**: Opening a cached map mid-download stresses the tile loader (same disk, two readers). **Mitigation**: Tile loader already uses an LRU cache (ADR-0010, ADR-0012); the disk contention is bounded by the OS page cache. Acceptable.
- **Trade-off**: Reserved status-bar height costs a few extra pixels of vertical space when idle. Cheap, and idle state is the rare case in a workflow whose whole point is downloading bundles.

## Migration Plan

No migration. This is a pure UI behavior change. After release, no user data or session file is affected.
