## 1. Remove blanket UI lock

- [x] 1.1 Drop `disabled={$busy}` from the project list-item button in `src/routes/+page.svelte`
- [x] 1.2 Keep `disabled={$busy}` on the refresh button (re-running `loadProjects` while one is in flight is the duplicate we DO want to block)
- [x] 1.3 Drop the blanket disable on map rows; keep the per-row `disabled={isDownloading}` on a row whose specific file is currently being fetched

## 2. Cross-bundle switch = cancel-and-restart

- [x] 2.1 In `handleSelectProject(slug)`: if `$activeDownloadId` is non-null and `selectedSlug !== slug`, call `cancelDownload($activeDownloadId)` and wait for it before calling `loadProject(slug)`
- [x] 2.2 Set transient status text ("Switching to <project>…") while the cancel + restart sequence is in flight; clear it once the new `loadProject` resolves
- [x] 2.3 Reset `readyBundleFiles` and `downloadProgress` on the switch (via existing `resetBundleDownloadState`) so the progress UI does not show stale entries from the cancelled bundle

## 3. Open cached maps mid-download

- [x] 3.1 Verify the existing `handleOpenMap(mapName)` path works when another download is in flight; if `openSelectedMap` returns an error in that scenario, capture the failure as a non-blocking toast (do not break the in-flight download)
- [x] 3.2 Ensure `goto('/project')` after `handleOpenMap` does not unmount the loader prematurely — the active download must continue once the workspace becomes the active route

## 4. Stable status-bar layout

- [x] 4.1 Convert `.status-bar` in `src/routes/+page.svelte` to a CSS grid with named slots: `status-line`, `progress`, `ready-list`, `actions`
- [x] 4.2 Reserve fixed `min-height` for each slot so `{#if}` toggles inside slots cannot change the grid's outer height
- [x] 4.3 Replace the ad-hoc `min-height: 24px` with a single CSS custom property (e.g. `--bundle-status-bar-height`) reused as the bar height AND as the bottom inset of the bundle-loader grid
- [x] 4.4 Reserve the 80px ready-files slot height even when the list is empty
- [x] 4.5 Reserve the progress-bar slot height even when no progress is being reported (placeholder track visible at 0%)

## 5. Verification

- [ ] 5.1 Manual: start a download, click another project, confirm the cancel + restart flow runs once and the new project starts downloading
- [ ] 5.2 Manual: start a download, click a cached map row in the same or a different bundle, confirm the workspace opens with that map AND the original download continues to completion (check progress events keep arriving)
- [ ] 5.3 Manual: during a download, record a short screen capture; confirm the bottom edge of the status bar does not move and the project list above it does not shift
- [x] 5.4 `just ci` passes (clippy, check, lint, test)
- [x] 5.5 `openspec validate bundle-loader-non-blocking --strict` passes
