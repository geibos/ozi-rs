
## 2026-04-25 Task 1
- Existing ADR style is concise: title, status/date metadata, Context, Decision, Consequences, with optional scoped sections such as non-goals or non-undoable mutations.
- Documentation skeletons should use "Planned in this reconciliation" until the later implementation tasks land and verify behavior.

## 2026-04-25 Task 3
- TrackPointsPanel can render `PointDetail.timestamp` directly; `src/lib/types.ts` already exposes it as an optional string, so no backend/DTO change was needed.
- The current Vitest setup has no Svelte component DOM harness, so the targeted test uses a lightweight source-level guard for conditional timestamp rendering without new dependencies.

## 2026-04-25 Task 4
- `get_tracks_geojson` already emits `line_width`, so TracksPanel style controls only needed frontend wiring; Rust commands were left untouched.
- TracksPanel receives backend colors as CSS `rgba(...)`, so the color input needs a hex conversion for display and converts selected hex back to `[r, g, b, 255]` for `setTrackColor()`.
- The repo still lacks a Svelte DOM harness, so `src/test/tracks-panel-style.test.ts` follows the existing source-level Vitest guard pattern.

## 2026-04-25 Task 5
- Waypoint symbol undo is safest at the AppState layer: read the current symbol immutably first, return `ProjectLayerError` on missing layer/waypoint, then construct `ProjectCommand::SetWaypointSymbol` so the command stack captures undo/redo deltas.

## 2026-04-25 Task 6
- OK-standard track-name validation is frontend-only and warning-only; the shared helper accepts exactly 8 digits, an underscore, and at least one non-whitespace callsign character, so Cyrillic callsigns work and calendar validity is not checked.
- TracksPanel warning coverage can stay source-level like recent UI tasks: verify helper usage and preserve `renameTrack` calls rather than adding a Svelte DOM harness.

## 2026-04-26 Task 7
- Track export dialog defaults now come from a read-only backend helper: `AppState::export_default_tracks_dir_path(track_name, extension)` builds `active_bundle/10-Tracks/<track-name>.<ext>` with `PathBuf` components and returns `None` when no active bundle can be inferred.
- `TracksPanel.svelte` keeps filename-only fallback defaults exactly as `<track-name>.gpx` and `<track-name>.plt`; only the dialog `defaultPath` suggestion changes, while the chosen path and export formats still flow through existing `exportGpx` / `exportTrackPlt` wrappers.
- `AppState::active_bundle_dir()` now recovers a bundle directory from a restored active map path when selected project state is absent, covering paths under the configured bundles root and local `8-Android&iOS` map folders.

## 2026-04-26 Task 8
- Active-layer selection can stay UI-only: `AppStateDto` now exposes minimal `track_layers` / `waypoint_layers` summaries, and `stores.ts` preserves the selected layer when still valid or falls back to the first available layer.
- Track drawing must capture the active track layer at creation time in `drawingTrackLayerId`; later map clicks use that captured layer so `createEmptyTrack`, `getTrackDetail`, and `insertTrackPoint` cannot drift across different layers.
- The repo still lacks a Svelte DOM harness, so `src/test/active-layer-ui.test.ts` follows the source-level guard style and checks DTO/store wiring, selector presence, no hardcoded layer `1n` component workflows, and no direct `invoke()` calls in the touched components.

## 2026-04-26 Task 2
- Session persistence is isolated from `.ozp` project serialization: `src-tauri/src/infrastructure/persistence.rs` stores a separate JSON record with only `last_project_path` and active-map metadata/path.
- `AppState::new()` remains fresh for ordinary unit tests; Tauri startup uses `AppState::new_with_session_path(default_app_session_path())` so restore happens before frontend command handlers can observe state.
- Successful project save/load and active map opens persist the bounded session snapshot; missing project paths abort restore to fresh state, while missing map paths leave a restored project loaded and emit a diagnostic.

## 2026-04-26 Task 9
- Bundle-loader window docs should describe `src/lib/windows.ts` as the single frontend helper module for the Tauri `bundles` webview: `precreateBundleLoader()` creates the hidden `/?view=bundles` window after main app mount, while `openBundleLoader()` reuses or finds that label, shows/focuses it, and only creates a fallback if needed.
- `src/main.ts` is the routing boundary for the secondary window: `view=bundles` mounts `BundleLoaderView.svelte`; the normal main-window path mounts `App.svelte` and triggers hidden pre-creation.
- Session restore remains intentionally bounded to project/map state; bundle-loader visibility/open state is not restored and should stay documented as a non-goal.

## 2026-04-26 Task 10
- `docs/feature-status.md` is the clearest home for backend/UI/status split language: each audited behavior should say what the backend supports, what the UI surfaces, and what remains planned/not implemented.
- Public docs should describe Task 8 as active-layer selection only; full layer management means create/rename/delete/reorder UI and remains not implemented.
- Export docs should say GPX/PLT dialogs suggest active-bundle `10-Tracks/` defaults when available; exports are not forced there and filename-only fallback remains valid.


## 2026-04-26 Task 11
- Targeted verification passed for session restore valid/missing, waypoint symbol, export default path, and the combined UI Vitest guard suite (`track-points-panel`, `tracks-panel-style`, `track-name-validation`, `export-default-path`, `active-layer-ui`). Transcript: `.sisyphus/evidence/task-11-targeted-verification.txt`.
- Full verification passed: `just test-rust`, `just test-ui`, `just clippy`, and `just test` all exited 0. Transcript: `.sisyphus/evidence/task-11-full-verification.txt`.
- Static audit passed: no direct `invoke()` calls in Svelte components, no hardcoded layer-1 workflow calls in app source, `docs/feature-status.md` covers the 10 audit terms, and public docs qualify bounded restore/full-layer-management claims. Transcript: `.sisyphus/evidence/task-11-static-audit.txt`.
- LSP diagnostics reported no diagnostics for checked affected files: `src-tauri/src/application/mod.rs`, `src-tauri/src/infrastructure/persistence.rs`, `src-tauri/src/commands/mod.rs`, `src/lib/api.ts`, `src/lib/stores.ts`, `src/lib/types.ts`, `src/components/TrackPointsPanel.svelte`, `src/components/TracksPanel.svelte`, `src/components/Sidebar.svelte`, and `src/components/WaypointsPanel.svelte`.

## 2026-04-26 Final-wave F2/F4 fix
- `TracksPanel.svelte` needs composite `(layerId, trackId)` identity not only for selected-state checks but also for Svelte each keys and local rename edit state, because track IDs can collide across layers while `selectedTrack` deliberately keeps the existing `{ layerId, trackId }` store shape.
- README/AGENTS should describe active-bundle `10-Tracks/` as a suggested/default GPX/PLT dialog location; export paths are never forced and users can choose another destination.
