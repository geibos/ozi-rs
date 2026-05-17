## Why

ozi-rs has been documented across `README.md`, `AGENTS.md`, `docs/`, and 24 ADRs, but lacks behavior-level specifications that future change proposals can reference. To adopt OpenSpec as the source of truth for behavioral specs going forward, the currently shipping behavior must first be captured. This change bootstraps the initial OpenSpec capability set so all subsequent work flows through the proposal → spec → archive workflow.

## What Changes

- Introduce 12 new capability specs that describe what the system currently does in production.
- Establish OpenSpec as the contract that future requirement changes MODIFY, ADD, or REMOVE against.
- Leave existing docs (`docs/requirements.md`, `docs/feature-status.md`, ADRs) in place as supporting material; they remain useful but are no longer the authoritative behavioral contract.
- No runtime, dependency, or build changes are introduced by this proposal.

## Capabilities

### New Capabilities

- `map-bundles`: Map bundles as directories of maps, local-folder open, configurable bundles root, reveal in OS file manager, multiple maps per bundle, per-project active map.
- `lizaalert-integration`: Browse remote project list from `maps.lizaalert.ru` as a stream, download and extract project bundles, observable download/extraction progress.
- `tile-rendering`: Custom `sqlite://` (MBTiles) and `ozi://` (OZF2 reprojected to Web Mercator) MapLibre tile protocols; OpenStreetMap online fallback.
- `project-persistence`: `.ozp` JSON save/load; bounded startup restore of last project path and active map; missing-file degradation; explicit list of non-restored state.
- `layers`: Track, waypoint, and map layer model; active-layer selection for editing workflows; backend layer summaries.
- `track-import`: GPX import (single file and ZIP archive); PLT import with Windows-1251 encoding; ZIP entry classification; user-facing import errors.
- `track-export`: GPX export with Garmin color extension; PLT export with OLE-date timestamps and COLORREF BGR color; active-bundle `10-Tracks/` default path suggestion.
- `track-display`: Per-track visibility, color, line width, opacity; statistics (distance, duration, point count); track-point list with timestamps; warning-only OK-standard track-name validation.
- `track-editing`: Move/insert/delete track point; split/join segment; drawing mode; Douglas–Peucker simplification with live preview.
- `waypoints`: Add/move/rename/delete waypoints; optional symbol; per-waypoint visibility; export to GPX and PLT.
- `undo-redo`: Delta-based command stack with depth 100; drag coalescing; redo clearing; explicit list of non-undoable style mutations; undo history not persisted across restarts.
- `ui-shell`: Catppuccin theme set (Auto / Latte / Frappé / Macchiato / Mocha) with localStorage persistence; backtick developer console toggle; F3 FPS counter.

### Modified Capabilities

None. This is a bootstrap; no prior specs exist.

## Impact

- After archive, `openspec/specs/` contains 12 capability directories with their canonical `spec.md` files.
- No code, dependency, or build configuration changes.
- Behaviors that are planned but not yet implemented (`waypoint-wpt-export`, on-map tools, sort-track-points-by-timestamp, recent-projects list, full layer manager UI, PDF print) are intentionally NOT included; each will be introduced via its own future change proposal.
- This change also updates `AGENTS.md` and `CLAUDE.md` to direct contributors to the OpenSpec workflow for behavioral changes.
