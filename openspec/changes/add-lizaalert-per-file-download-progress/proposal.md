## Why

LizaAlert project bundles consist of multiple numbered-prefix files (`00-...`, `10-Tracks/...`, `20-...` etc.) of very different sizes — small JSON manifests, medium-size OZF2 raster maps, large reference PDFs. Today the user gets only coarse phase events (Scanning → Downloading → Extracting) and must wait for **every** file to finish before the project is usable. Three concrete problems on slow field channels (the LTE/3G context where this app runs):

1. **No per-file visibility.** The user cannot tell which file is being fetched or how much remains, even though `download-progress` with `package_name` / `downloaded_bytes` / `total_bytes` is already defined for the single-map path (`download_map` in `src-tauri/src/infrastructure/lizaalert.rs:267-306`). The parallel project mirror path (`mirror_remote_directory`) never emits it.
2. **No incremental usability.** A SAR operator wants the OZF2 map (small, arrives fast) the moment it lands on disk — the operator does not need the 200 MB regional PDF to finish before opening a map and starting work. Today the bundle is gated behind "everything done".
3. **UI freezes during downloads.** Long synchronous awaits in the Tauri command path block the Svelte main thread; the loader view stops repainting.

This change broadens the requirement to cover (a) per-file progress, (b) deterministic prefix-ordered scheduling so small/critical files arrive first, (c) incremental bundle availability, and (d) non-blocking UI throughout.

## What Changes

- **Per-file progress events** — `mirror_remote_directory` (and its `open_project` wrapper) emit `download-progress` events for every remote file:
  - `package_name` = file path relative to the project root (e.g. `10-Tracks/track1.ozf2`)
  - `downloaded_bytes` = bytes received so far for that file (monotonic per file)
  - `total_bytes` = `Content-Length` when available, otherwise omitted
  - `file_index` / `file_count` = position in the bundle's prefix-sorted file list, so the UI can show "3 / 12 — `10-Tracks/track1.ozf2`"
- **Parallel download with bounded concurrency** — files are fetched concurrently (configurable cap, default ≥3) but **scheduled in the order their leading numeric prefix dictates** (`00-` first, then `10-`, `20-`, ...; files without a prefix sort last alphabetically). Smaller-prefix files start first so the user reaches map data before reference material.
- **Incremental bundle availability** — the system emits a new `bundle-file-ready` event with `{package_name, local_path}` as soon as each individual file is fully downloaded and verified. The active bundle's contents update progressively — map files become openable in the existing bundle UI before the rest of the bundle finishes. The user can open the bundle and load a partially-downloaded project; missing files surface as "still downloading" placeholders rather than as errors.
- **Coordinated event stream** — parallel workers funnel events through a shared async channel (or `Mutex<Emitter>`) so the UI sees a coherent ordered stream without dropped or torn updates.
- **Non-blocking UI** — all download / extraction / disk I/O work runs on a Tokio task; the Tauri command returns immediately with a `download_id` and the UI subscribes to events keyed by that id. The Svelte main thread never `await`s the whole download. The user can switch panels, open Settings, browse already-downloaded bundles, or even start work on a partially-arrived file while the rest streams in the background.
- **`BundleLoaderView.svelte`** — shows current-file label, indeterminate bar when `total_bytes` is missing, per-file completion ticks as `bundle-file-ready` events arrive, and a "Open partial bundle" button enabled the moment the first map-bearing file is ready.

## Impact

- Affected capabilities:
  - `lizaalert-integration` — MODIFIED: "Download progress is observable" requirement gains multi-file scenario, prefix-ordered scheduling, incremental availability via `bundle-file-ready`, and non-blocking-UI guarantee.
- Affected code (implementation, follow-up change):
  - `src-tauri/src/infrastructure/lizaalert.rs` — `mirror_remote_directory` gains a progress callback, prefix-ordered file enumeration, bounded-concurrency worker pool (`tokio::task::JoinSet` + semaphore), and per-file ready notification after the file is fsync'd.
  - `src-tauri/src/commands/mod.rs` — `load_project` returns immediately with a `download_id`; spawns the work on a `tokio::task::spawn`; forwards both `download-progress` and `bundle-file-ready` events through the Tauri `Emitter` keyed by id.
  - `src-tauri/src/application/mod.rs` — bundle state machine accepts partial-bundle availability (already-downloaded files exposed even while others are still in flight).
  - `src/views/BundleLoaderView.svelte` — current-file label, per-file completion list, "Open partial bundle" affordance, all reactive on events. No `await` on the download promise.
- Wire format: additive only — same `download-progress` event name and payload shape used by single-map downloads, plus new `file_index`/`file_count` fields and new `bundle-file-ready` event. No breaking change to existing consumers.
- Risk: medium — concurrency and partial-state correctness need careful tests (per-file monotonicity, no torn writes, deterministic prefix ordering, cancel/resume mid-download). Covered by tasks §5.
