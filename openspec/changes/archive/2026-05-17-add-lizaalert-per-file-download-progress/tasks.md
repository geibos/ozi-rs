## 1. Validate delta

- [x] 1.1 Run `openspec validate add-lizaalert-per-file-download-progress --strict` and confirm the `lizaalert-integration` MODIFIED requirement parses with all original scenarios plus the new multi-file, prefix-order, incremental-availability, and non-blocking-UI scenarios.
- [x] 1.2 Confirm the full-text copy of the modified requirement matches the current body of `openspec/specs/lizaalert-integration/spec.md` before the additions (no silent edits to existing scenarios).

## 2. Wire per-file `download-progress` in `mirror_remote_directory`

- [x] 2.1 In `src-tauri/src/infrastructure/lizaalert.rs`, extend `mirror_remote_directory` to accept a progress callback `Fn(&str /*package_name*/, u64 /*downloaded*/, Option<u64> /*total*/, usize /*idx*/, usize /*count*/) + Send + Sync`. *(Replaced by `DownloadNotification::FileProgress` events from `download_bundle_concurrent`; same payload shape, channel-based delivery — `src-tauri/src/infrastructure/lizaalert.rs:101`.)*
- [x] 2.2 For each remote file fetch, compute `package_name` as the URL path relative to the project root (preserve subdirs like `10-Tracks/`). *(`collect_remote_files_rel`.)*
- [x] 2.3 Stream the HTTP body chunk-by-chunk and invoke the callback at least once per file plus every ~64 KiB; match the cadence already used by single-map `download_map`. *(Async streaming via `reqwest::Client::bytes_stream`; one progress event per chunk plus a leading 0-byte event so the UI label appears immediately.)*
- [x] 2.4 Pass `Content-Length` through as `total_bytes`; emit `None` if absent — frontend already handles indeterminate progress. *(Tested by `missing_content_length_still_emits_progress` at `lizaalert.rs:1661`.)*

## 3. Prefix-ordered scheduling with bounded concurrency

- [x] 3.1 After listing the remote directory, sort the file enumeration by leading numeric prefix (`00-`, `10-`, `20-`, ...). Files without a numeric prefix sort lexicographically after the prefixed ones. *(`sort_remote_files_by_prefix` + `leading_prefix` at `lizaalert.rs:145`; covered by `prefix_ordered_scheduling_starts_smallest_first` at `lizaalert.rs:1606`.)*
- [x] 3.2 Drive the download with `tokio::task::JoinSet` plus an `Arc<tokio::sync::Semaphore>` (initial permits = `download_concurrency`, default `3`, configurable later if needed). Spawn tasks in sorted order so smaller-prefix files start first while still benefiting from parallelism. *(`lizaalert.rs:511`–`512`; covered by `concurrency_cap_is_respected` at `lizaalert.rs:1894`.)*
- [x] 3.3 Funnel progress and completion notifications through a single `tokio::sync::mpsc::UnboundedSender` consumed by a dedicated forwarding task that calls the Tauri `Emitter`. Workers never touch the emitter directly. *(See `commands/mod.rs:457` forwarder; sole writer to the Tauri Emitter.)*

## 4. Incremental bundle availability (`bundle-file-ready`)

- [x] 4.1 After each file finishes downloading and is fsync'd to its final path inside the bundle root, the worker sends a `BundleFileReady { package_name, local_path }` notification through the same channel. *(`DownloadNotification::FileReady` at `lizaalert.rs:109`; `download_to_path_async` fsyncs the `.part` file and atomically renames to the canonical path.)*
- [x] 4.2 In `src-tauri/src/commands/mod.rs`, forward those notifications as a new `bundle-file-ready` Tauri event with the same payload. *(`commands/mod.rs:476` — `DownloadNotification::FileReady` handler emits `bundle-file-ready` via the forwarder.)*
- [x] 4.3 In `src-tauri/src/application/mod.rs`, the bundle / active-map state accepts and reflects partially-downloaded bundles: list & open whatever is on disk now, ignore files still in flight without erroring. *(`AppState::note_bundle_file_ready` at `application/mod.rs:382`; non-map files like PDF/TXT are tolerated.)*
- [ ] 4.4 If the user opens a partial bundle and asks for a file that hasn't arrived yet, return a clear `not_yet_downloaded` error (don't crash, don't block). *(Deferred out of scope: `apply_map_downloaded` path already routes missing files through the diagnostic stream and triggers a re-download; no dedicated `not_yet_downloaded` enum variant added. Tracked as a follow-up change.)*

## 5. Non-blocking UI (download runs detached)

- [x] 5.1 Refactor `load_project` to return immediately with a `download_id: String` (uuid v4). Move the existing body into a `tokio::task::spawn`'d future keyed by that id. *(`commands/mod.rs:421`; `DownloadRegistry` tracks the spawned task.)*
- [x] 5.2 All Tauri events (`download-progress`, `bundle-file-ready`, `bundle-progress`) carry the `download_id` so the frontend can disambiguate concurrent downloads (future-proofing — even though MVP only triggers one at a time, the contract should not assume singleton). *(See `BundleFileReadyPayload` at `commands/mod.rs:204` and progress payload struct.)*
- [x] 5.3 Provide a `cancel_download(download_id)` Tauri command that aborts the spawned task; in-flight HTTP requests drop; already-downloaded files stay on disk (resumable later). *(`commands/mod.rs:526`; `CancelToken` + async `select!` inside `download_to_path_async`; aborts in <250 ms — covered by `cancel_aborts_within_deadline_and_resume_only_fetches_missing` at `lizaalert.rs:1767`.)*
- [x] 5.4 The frontend `loadProject` API wrapper returns the `download_id` and exposes an `onDownloadProgress(id, cb)` / `onBundleFileReady(id, cb)` / `cancelDownload(id)` surface. *(API exposes `loadProject -> Promise<string>`, `openLocalBundle -> Promise<string>`, `cancelDownload(id)`. On-event subscription stays on the existing `listen()` channel keyed by `download_id` inside the payload — simpler than a per-id callback registry and survives concurrent downloads.)*

## 6. Frontend — `BundleLoaderView.svelte`

- [x] 6.1 `updateDownloadProgress` reads `payload.package_name`, `payload.file_index`, `payload.file_count` and renders a secondary label "Downloading 3 / 12 — `10-Tracks/track1.ozf2`". *(`currentFileLabel` derived store at `BundleLoaderView.svelte:183`; testid `current-file-label`.)*
- [x] 6.2 If `total_bytes` is absent, render an indeterminate animation rather than dividing by zero. *(testid `indeterminate-bar` at `BundleLoaderView.svelte:319`.)*
- [x] 6.3 Subscribe to `bundle-file-ready`: maintain a reactive list of ready files; render each as a row with a tick. Once at least one openable map file is ready, enable an "Open bundle now" button that opens the partial bundle without waiting for the rest. *(testid `open-bundle-now` at `BundleLoaderView.svelte:358`.)*
- [x] 6.4 Loader view does not `await` the long-running `loadProject` promise; it kicks off the download, stores the `download_id`, and rerenders purely from events. Switching to other panels mid-download remains responsive. *(`handleSelectProject` is a synchronous function and uses `.then()`; covered by `bundle-loader-non-blocking.test.ts`.)*
- [x] 6.5 Cancel button calls `cancelDownload(id)` and returns the loader to the bundle list. Already-downloaded files remain available; resuming triggers a new download of only the missing files. *(testid `cancel-download`; resume behavior covered by `cancel_aborts_within_deadline_and_resume_only_fetches_missing`.)*

## 7. Tests

- [x] 7.1 Rust integration test (mock HTTP server such as `wiremock` or `httptest`): synthetic project with five files (`00-manifest.json`, `10-Tracks/a.ozf2`, `10-Tracks/b.ozf2`, `20-overlay.zip`, `99-refs.pdf`) of distinct sizes. Assert per-file `download-progress` events arrive, per-file monotonic `downloaded_bytes`, final `downloaded_bytes == total_bytes` for each. *(`per_file_progress_is_monotonic_and_complete` at `lizaalert.rs:1542`.)*
- [x] 7.2 Rust test: files lacking `Content-Length` still produce `download-progress` events with `total_bytes = None` without panic. *(`missing_content_length_still_emits_progress` at `lizaalert.rs:1661`.)*
- [x] 7.3 Rust test: prefix-ordered scheduling — the first event arrives for `00-manifest.json` even when the server adds artificial latency to the `99-` file. *(`prefix_ordered_scheduling_starts_smallest_first` at `lizaalert.rs:1606`.)*
- [x] 7.4 Rust test: `bundle-file-ready` fires for `10-Tracks/a.ozf2` strictly before the `99-refs.pdf` download completes, when `a.ozf2` is small and `refs.pdf` is large with a delayed server. *(`small_file_ready_before_large_file_completes` at `lizaalert.rs:1713`.)*
- [x] 7.5 Rust test: `cancel_download` mid-stream aborts in-flight tasks within 250 ms; already-downloaded files remain on disk; a follow-up `load_project` resumes by downloading only missing files. *(`cancel_aborts_within_deadline_and_resume_only_fetches_missing` at `lizaalert.rs:1767`.)*
- [x] 7.6 Rust test (parallelism): with `download_concurrency = 3` and 6 files, observe at most 3 concurrent in-flight workers (track via mock server hit log). *(`concurrency_cap_is_respected` at `lizaalert.rs:1894`.)*
- [x] 7.7 Frontend (vitest): `BundleLoaderView` given a scripted event stream renders the current-file label, indeterminate bar where applicable, ready-file ticks, and surfaces the "Open bundle now" button after the first ready event. Cancel button dispatches `cancelDownload`. *(`src/test/bundle-loader-progress.test.ts`.)*
- [x] 7.8 Frontend (vitest): main-thread responsiveness — assert that a synthetic `loadProject` returning a long-pending event stream does not block a separate sibling component from rendering (no `await` on the download promise in the loader). *(`src/test/bundle-loader-non-blocking.test.ts`.)*

## 8. QA via `just test` / `just clippy`

- [x] 8.1 `just clippy` is clean.
- [x] 8.2 `just test` passes (Rust + frontend). *(203 Rust + 66 vitest after merge.)*
- [ ] 8.3 Manual smoke per `docs/agent-verification.md`: throttle network (or pick a known multi-file LizaAlert project), trigger download, observe per-file label cycling through `00-`, `10-`, `20-`, ... in order; the "Open bundle now" button activates partway through the run and successfully opens the partial bundle; cancel works mid-stream; resume restarts only the missing files; the UI stays interactive (open Settings, switch panels) throughout.
