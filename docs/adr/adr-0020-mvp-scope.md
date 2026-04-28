# ADR-0020: MVP Scope Declaration

- Status: accepted
- Date: 2026-04-28

## Context

After Phase 7-10 feature work, the application accumulated functionality faster than
the user could verify it. During a brainstorming session the user reported that several
features described as "implemented" in `docs/feature-status.md` are not visible or not
reachable in the UI: track import, track display, color/line-width controls, the track
points panel, point deletion, segment splitting, simplification, all waypoint workflows,
and undo/redo. The map list ("Maps") is reportedly broken, bundle loading is slow.

This ADR declares the intended MVP scope so that subsequent QA, UX, and bug-fixing work
have a single source of truth for "what the application is trying to be." Real
implementation status is recorded separately in `docs/feature-status.md` after audit.

## Decision

The MVP supports a complete LizaAlert search-and-rescue track-handling workflow:
open a map bundle, import or draw tracks, edit and visualize them, place waypoints,
and export results in formats consumable by external navigators.

### MVP — must work

#### Maps
- Open LizaAlert bundle by URL
- Open bundle from local folder
- Switch between maps inside a bundle (Topo / Satellite)
- SQLite (MBTiles) tile maps
- OZF2 raster maps (always present in LizaAlert bundles)
- OpenStreetMap as online fallback

#### Tracks
- Import GPX and PLT, including ZIP archives
- Display many tracks simultaneously
- Fast load of tracks with tens of thousands of points
- Per-track color and line style/width
- Walk through track points one by one with per-point info
- Delete track points
- Split a track into segments (segment breaks)
- Sort track points by timestamp
- Douglas–Peucker simplification with tolerance preview
- Crop track by current map extent, by time range, by selected points
- Draw a new track on the map
- Export to GPX and PLT, defaulting to active bundle's `10-Tracks/` when known

#### Waypoints
- Add, move, rename, delete
- Color, symbol, visual customization
- Display many simultaneously
- Export to GPX, PLT, and WPT (see ADR-0022)

#### On-map tools
- Distance measurement
- Circle with center at point/cursor and explicit radius
- Place waypoint by projection (azimuth + distance) from a selected point

#### Project
- Save/load in `.ozp` (JSON) format
- Recent projects list
- Undo/redo using existing delta-based command stack (see ADR-0021)
- OK-standard track name validation, warning-only

#### Platform and UI
- Windows is the primary target platform
- macOS is the secondary target
- Linux must work without out-of-the-box configuration
- Catppuccin themes (Auto, Latte, Frappé, Macchiato, Mocha)
- Developer console and FPS counter remain available for debugging

### Future (post-MVP)

- USB upload of tracks/waypoints to navigators with deletion of stale files
- USB pull from navigators with naming check and rename UI
- FTP automation for track upload
- Multi-layer UI: full management of multiple track and waypoint layers (create,
  rename, delete, reorder)
- KML import/export

### Decisions made within this ADR

- **Single layer in UI for MVP.** The backend supports multiple track and waypoint
  layers (`AppStateDto` exposes layer summaries). For MVP the UI exposes one track
  layer and one waypoint layer; the active-layer selector and layer manager are
  hidden. Backend is unchanged.
- **No map printing.** Recorded separately in ADR-0023 with rationale.

### Out of scope (not Future, not planned)

- Datum management
- Advanced geodesy beyond what the on-map tools require
- GPS device live telemetry
- Routes and events
- Polygon / search sector drawing
- Privileged legacy concepts such as a special `Track 1`

## Consequences

### Positive

- Brainstorming, planning, and code-review have a single declared MVP target.
- Future automation requirements (USB, FTP) are recorded so they are not lost
  when prioritising current work.
- Multi-layer is explicitly deferred so MVP UI stays simpler.

### Negative

- The declared scope is broader than the currently shipped UI. A separate audit
  (direction B, see brainstorming session) must reconcile the declaration with
  reality before scope decisions can be acted on.
- The single-layer UI for MVP means later re-introduction of multi-layer UI will
  need its own design pass.

## Related

- ADR-0017 — delta-based undo/redo (kept as-is, see ADR-0021)
- ADR-0019 — documentation audit reconciliation (predecessor: smaller scope)
- ADR-0021 — undo depth and stack reaffirmation (this ADR)
- ADR-0022 — WPT waypoint export inclusion in MVP
- ADR-0023 — map printing explicitly out of scope
