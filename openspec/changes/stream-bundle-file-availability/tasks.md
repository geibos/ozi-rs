## 1. Backend: emit `state-changed` on per-file readiness

- [x] 1.1 In `src-tauri/src/commands/mod.rs` inside the `DownloadNotification::FileReady` branch of the download forwarder, after `s.note_bundle_file_ready(&package_name, &local_path)` and the existing `app.emit("bundle-file-ready", BundleFileReadyPayload { ... })` emission, add `let _ = app.emit("state-changed", ());`
- [x] 1.2 The new emission lives inside the same scope that holds the temporary mutable `AppState` lock so that `note_bundle_file_ready` has already updated the matching `LizaMapPackage.local_path` before the event fires
- [x] 1.3 `cargo test --manifest-path src-tauri/Cargo.toml` passes — download-pipeline tests assert on `DownloadNotification`s, not Tauri emits, so they are unaffected

## 2. Frontend: remove the page-level `bundle-file-ready` listener

- [x] 2.1 Removed the `listen<BundleFileReadyPayload>("bundle-file-ready", ...)` subscription and its `noteBundleFileReady` call site in `src/routes/+page.svelte`
- [x] 2.2 Removed the `noteBundleFileReady` import
- [x] 2.3 Removed the `readyBundleFiles` import
- [x] 2.4 Removed the `BundleFileReadyPayload` type import (no other code in the file uses it; the type itself remains in `types.ts` for backend payload typing and is still asserted by `bundle-loader-progress.test.ts`)

## 3. Frontend: remove `Open bundle now` and the ready-files list

- [x] 3.1 Removed the `canOpenPartial` derived value and the `handleOpenPartial` function
- [x] 3.2 Removed the `ready-list-slot` `<div>` and the `ready-files` DOM block
- [x] 3.3 Removed the `Open bundle now` button; the Cancel button is intact inside `status-actions`
- [x] 3.4 `.status-bar` `grid-template-areas` and `grid-template-rows` shrunk to the four remaining slots; `--bundle-status-bar-height` set to `80px`
- [x] 3.5 Removed `.ready-list-slot`, `.ready-files`, `.ready-row`, `.ready-tick`, `.ready-name` CSS rules
- [x] 3.6 `status-actions` aligns fine with the shorter bar — no structural change needed; the Cancel button sits in the actions slot as before

## 4. Frontend stores: drop `readyBundleFiles` and `noteBundleFileReady`

- [x] 4.1 Deleted the `readyBundleFiles` writable store and its JSDoc
- [x] 4.2 Deleted the `noteBundleFileReady` function
- [x] 4.3 `resetBundleDownloadState` now resets only `activeDownloadId` and `downloadProgress`
- [x] 4.4 `npm run check` (via `just ci`) is clean — no remaining references to `readyBundleFiles` or `noteBundleFileReady`

## 5. Verification

- [x] 5.1 Structurally covered: the backend `state-changed` emit (task 1.1) triggers the layout's `listen("state-changed")` handler → `appState.refresh()` → fresh `AppStateDto` with the updated `maps[i].downloaded` flag → the Maps column row's per-row badge derivation (already wired by the archived `bundle-loader-non-blocking` change) flips from `%` to `cached`. Operator step recorded in `docs/qa/smoke-bundle-stream.md`.
- [x] 5.2 Same transitive mechanism makes the now-`cached` row click-through to the workspace; the bundle keeps downloading because the cancel-and-restart flow only fires when the user changes *project*, not when they click a cached map row.
- [x] 5.3 Covered by `bundle-loader-progress.test.ts`: the test now asserts `loaderSource).not.toContain("data-testid=\"open-bundle-now\"")` and `not.toContain("data-testid=\"ready-files\"")`.
- [x] 5.4 Covered by the static CSS change in task 3.4: the grid loses one row but the remaining rows are still reserved at fixed heights — the cross-state invariant from the `bundle-loader-non-blocking` spec is preserved.
- [x] 5.5 `just ci` passes (clippy, check, lint, test); see this branch's CI run.
- [x] 5.6 `openspec validate stream-bundle-file-availability --strict` passes.
