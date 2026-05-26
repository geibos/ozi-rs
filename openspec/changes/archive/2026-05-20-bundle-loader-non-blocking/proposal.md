## Why

Today the bundle loader blocks every project row and every map row in the list as soon as `appState.busy` flips true. A user who starts downloading bundle A cannot switch to bundle B and cannot open maps that are already cached locally — including maps inside the bundle currently being fetched. The intent (don't let the user double-launch the same download) collapsed into a blanket lock that contradicts the offline-first stance of the tool: cached data is always available, even mid-download.

In parallel, the progress status bar is a `position: fixed` block whose contents (current-file label, indeterminate bar, byte counters, per-file tick list, "Open bundle now" / "Cancel" buttons) appear and disappear on every progress event. Its height changes 5-10× per second during a download, which is what the user perceives as "everything is jumping". This is not a perf problem; it is a layout problem.

## What Changes

- The bundle loader project list SHALL remain interactive while a download is in progress. Selecting a different project SHALL be allowed; the runtime decides whether to cancel-and-restart, queue, or attach progress to the new selection (see design.md).
- Map rows for already-downloaded maps inside any bundle SHALL be openable regardless of whether some other map (or the same bundle's pending files) is still downloading. Only the specific map row whose file is being fetched SHALL remain disabled, with its progress badge as the affordance.
- The status bar SHALL have a stable layout that does not reflow as progress events arrive. Progress contents SHALL occupy a reserved area of fixed height; appearing/disappearing pieces SHALL fade in or fill placeholder rows, not push neighbours.
- `disabled={$busy}` SHALL be removed as a blanket gate on project-list and map-list rows; per-row disable SHALL be driven only by the specific in-flight operation that touches that row.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `map-bundles`: bundle/map selection is not globally blocked during an in-flight download; cached maps remain openable mid-download.
- `ui-shell`: the bundle-download progress region has a stable layout (reserved space, no reflow on progress events).

## Impact

- **Frontend**: `src/routes/+page.svelte` (loader markup and `disabled` predicates), `src/lib/stores.ts` (decoupling "an op is in flight" from "user input is locked"), CSS for `.status-bar` (reserved layout slots).
- **Backend**: no change required if cancel-and-restart is the chosen strategy for cross-bundle switching. If concurrent downloads are chosen instead, `application/` state for `activeDownloadId` becomes a set (out of scope for this change — see design.md).
- **Spec evidence**: smoke check that selecting another project mid-download works; visual smoke that the status-bar bottom edge does not jitter during a download.
- **Risk**: low. Removing the blanket lock can only widen what the user is allowed to do; the affected backend commands (`load_project`, `open_selected_map`) already validate their inputs and can reject invalid concurrency.
