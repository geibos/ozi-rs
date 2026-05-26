## Why

Change A (`bundle-loader-non-blocking`) just shipped: the Maps column now shows a per-row badge (`cached` green / `↓` orange / `% blue`) and disables only the specific row whose file is currently in flight. But the `cached` flip is wired to `AppStateDto.current_project.maps[i].downloaded`, which today is only updated when the whole bundle finishes and the backend finally emits `state-changed` from `commands/mod.rs:516`. Mid-download, `note_bundle_file_ready` (`application/mod.rs:388`) silently sets the map's `local_path` in memory — but no event tells the frontend, so the row stays at "downloading %" until the entire bundle is done.

The user has watched four maps finish back-to-back during one download and seen all four flip from `%` → `cached` at the same moment, well after each was actually openable. Their words:

> Я не хочу кнопку «open bundle now», я хочу чтобы они справа появлялись по мере загрузки.

The "Open bundle now" button and the ready-files list in the status bar exist precisely to compensate for this lag — they let the user manually re-trigger an `appState.refresh()` and then click the partially-arrived map. Once the Maps column itself reflects per-file readiness live, both affordances become redundant.

## What Changes

- The backend SHALL cause `AppStateDto.current_project.maps[i].downloaded` to flip to `true` for a given map within the same event tick that emits `bundle-file-ready` for that map's file — no waiting for the whole bundle to finish. Strategy: keep `bundle-file-ready` as today AND additionally emit `state-changed` from the same forwarder branch (see design.md, Decision 1).
- The Maps column in `src/routes/+page.svelte` SHALL transition a row's badge from `blue %` to `green cached` and clear its `disabled={isDownloading}` the moment that map's file finishes, while the rest of the bundle is still being fetched.
- The "Open bundle now" button SHALL be removed from the bundle-loader status bar. Its slot (`status-actions` row in the grid) collapses to "Cancel only" while a download is in flight.
- The ready-files list SHALL be removed from the bundle-loader status bar. Its reserved grid row (`ready-list`) SHALL be removed from the layout; the bar's overall height SHALL shrink accordingly, but the remaining slots SHALL keep their stable-layout guarantees from change A.
- The frontend store `readyBundleFiles` and the helper `noteBundleFileReady` SHALL be removed; `resetBundleDownloadState` SHALL no longer reset that store.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `map-bundles`: per-map availability inside the active bundle SHALL stream into the Maps column live, so a cached map becomes clickable as soon as its file is on disk, not when the whole bundle finishes.
- `ui-shell`: the status-bar slot contents for the bundle loader change — the "Open bundle now" affordance and the ready-files list go away; the remaining slots (status line, current file, progress bar, byte counters, cancel action) keep their stable-layout guarantee from change A.

## Impact

- **Backend** (`src-tauri/src/commands/mod.rs`): inside the `FileReady` branch of the download forwarder (lines ~473-492), emit `state-changed` after the existing `note_bundle_file_ready` + `bundle-file-ready` emission so the frontend pulls a fresh `AppStateDto`.
- **Frontend** (`src/routes/+page.svelte`): remove the `bundle-file-ready` page-level listener (it was driving `noteBundleFileReady`); remove `canOpenPartial`, `handleOpenPartial`, the `ready-list-slot` grid area and its DOM, and the `Open bundle now` button; trim `.status-bar` `grid-template-areas` / `grid-template-rows` / `--bundle-status-bar-height` accordingly.
- **Frontend stores** (`src/lib/stores.ts`): remove `readyBundleFiles` and `noteBundleFileReady`; drop the `readyBundleFiles.set([])` line from `resetBundleDownloadState`.
- **Round-trip with existing changes**:
  - Change A's stable-layout guarantee still holds: the bar still uses CSS-grid reserved slots; only the `ready-list` row is removed. The bar's outer height changes once (idle-vs-busy is still the same), so the cross-state invariant from A's spec is preserved.
  - Change C (`consolidate-state-event-flow`) will later move the `bundle-file-ready` listener to `+layout.svelte`. This change removes the page-local listener entirely (the only consumer was `noteBundleFileReady`), which puts no new listener in the page and does not contradict C. See design.md "Overlap with other changes".
- **Spec evidence**: manual smoke — start a multi-file bundle download, watch a map row finish first, click it before the rest of the bundle finishes; the workspace opens with that map and the original download continues to completion.
- **Risk**: low. `state-changed` is already emitted on dozens of mutations across `commands/mod.rs`; adding one more per file-ready event is in the same order of magnitude as the existing event rate. The frontend already de-duplicates `getAppState()` work via Svelte store equality. The `Open bundle now` button has no functional contract beyond "refresh app state then let the user click a cached row" — removing it removes only a manual workaround.
