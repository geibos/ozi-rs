# MVP Scope — Design Spec

- Date: 2026-04-28
- Status: declaration accepted; reality audit pending (see direction B)
- Authoritative decisions: ADR-0020, ADR-0021, ADR-0022, ADR-0023

## Purpose

This spec is the entry point for "what is the application trying to be." It collects
the MVP-must features, the deferred work, the dropped work, and — critically — flags
the gap between declared scope and verified reality.

Implementation work (bug fixing, UI polish, new features) must trace back to a
section of this spec. Anything not listed here either is in Future, was dropped, or
needs a successor ADR before it earns engineering time.

## Primary user workflow

A LizaAlert search-and-rescue volunteer running an operation:

1. Open a map bundle — by URL from `maps.lizaalert.ru` or by picking a local folder
   they previously downloaded.
2. Switch between maps inside the bundle (typically Topo and Satellite) without
   restarting the app.
3. Either draw new task tracks on the map (with a chosen style and color) or import
   tracks delivered by other operators (GPX or PLT, possibly inside a ZIP).
4. Edit tracks: walk through points to inspect them, delete bad points, split a
   track at a segment break, sort by timestamp, simplify high-density tracks,
   crop by current map extent / time range / point selection, change visual style.
5. Place waypoints by clicking on the map; rename, change symbol/color, drag
   to reposition. Place a waypoint by projection (azimuth + distance) from
   another point. Measure a distance. Draw a circle of explicit radius.
6. Save the operation as a `.ozp` project. Re-open recent projects quickly.
7. Export tracks to GPX/PLT and waypoints to GPX/PLT/WPT for delivery to
   handheld navigators.

The platform priority is Windows first, macOS second, Linux without bespoke
configuration. Mobile platforms are not in scope but should not be designed out.

## In MVP

The detailed list lives in ADR-0020. Summary:

- **Maps:** SQLite (MBTiles), OZF2 raster, OSM fallback; bundle-by-URL and
  local-folder open; map switching inside a bundle.
- **Tracks:** GPX/PLT import (incl. ZIP); display, color, line style/width;
  point-by-point inspection; deletion, segment break, timestamp sort,
  Douglas–Peucker simplification, cropping by extent/time/selection;
  drawing; GPX/PLT export with `10-Tracks/` default.
- **Waypoints:** add/move/rename/delete; color, symbol, visual; many at once;
  GPX/PLT/WPT export.
- **On-map tools:** distance measurement, circle with radius, projection-from-point
  waypoint placement.
- **Project:** `.ozp` save/load, recent projects, undo/redo per ADR-0017
  (reaffirmed in ADR-0021), warning-only OK-standard validation.
- **UI:** Catppuccin themes (5 + Auto), dev console and FPS counter for debugging.

## Future (post-MVP)

Recorded in ADR-0020. Summary:

- USB upload to navigators with stale-file deletion
- USB pull from navigators with naming check and rename UI
- FTP automation for track upload
- Multi-layer UI (create/rename/delete/reorder track and waypoint layers)
- KML import/export

## Dropped

- Map printing — see ADR-0023.

## Performance constraints

- Loading and panning a track with **tens of thousands of points** must remain
  smooth. This is a hard constraint, not a stretch goal — search-and-rescue tracks
  recorded over a full day routinely exceed 30k points.
- Many tracks (dozens) and many waypoints (hundreds) shown simultaneously on the
  same map.
- Bundle open and map switching must feel instant. Current behaviour (slow bundle
  list, "Maps" button unresponsive) is treated as a critical bug to be located and
  fixed during direction B.

## Reality gap and verification

`docs/feature-status.md` lists most of the MVP-must features as "implemented." The
user reports that during normal use most of those features are not visible or do
not respond:

| Area | User report |
|------|-------------|
| Bundle open (LizaAlert and local) | Slow and inconvenient |
| Map switching ("Maps") | No response on click |
| Track import | "Looks like it doesn't work" |
| Track display | "Looks like it doesn't work" |
| Track style/color UI | "Looks like it doesn't work" |
| Track points panel | Not seen in UI |
| Point delete, segment break, sort, simplify, crop, export | Not seen in UI |
| Waypoint workflows (incl. WPT export) | Not seen in UI |
| Undo/redo | Not tested |

This contradiction is the central finding from this brainstorming round. **The MVP
declaration is not actionable until reality is reconciled with it.** Therefore:

- Direction B (QA process and audit) must execute before any MVP fix-up work.
- The output of B is an updated `docs/feature-status.md` with verified states
  (works / partial / broken / hidden) and a triage list.
- Direction A (UX/native look) builds on B's findings — there is no point polishing
  visuals around features that are broken or hidden.

## Acceptance for MVP

A feature counts as "shipped in MVP" when:

1. It is reachable from the main UI without dev-console workarounds.
2. It works on Windows for a fresh install with no manual setup beyond the
   documented quick-start steps.
3. It has at least one automated test (Rust or Vitest, depending on layer) that
   protects against regression.
4. The relevant `docs/feature-status.md` row is updated and the evidence column
   points at a real artefact (test name, screenshot, or QA log).

A bug counts as "fixed for MVP" when the same four conditions hold for the failing
case.

## Out of scope for this spec

- Implementation strategy. Direction B will produce a debug-and-QA process spec;
  direction A will produce a UX/visual spec. Each gets its own implementation plan.
- Detailed UI mockups, layer hierarchy in MapLibre, or rendering pipeline changes.
- Telemetry decisions, crash reporting, error budgets.

## Related

- ADR-0017 — delta-based undo/redo
- ADR-0019 — documentation audit reconciliation (predecessor in spirit)
- ADR-0020 — MVP scope decision
- ADR-0021 — undo stack reaffirmation
- ADR-0022 — WPT waypoint export in MVP
- ADR-0023 — map printing not in scope
- `docs/feature-status.md` — to be updated after direction B audit
