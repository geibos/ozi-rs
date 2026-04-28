# ADR-0022: WPT Waypoint Export in MVP

- Status: accepted
- Date: 2026-04-28

## Context

The application currently exports tracks to GPX and PLT (OziExplorer). Waypoints can
be embedded in GPX, but there is no dedicated waypoint-only export, and the
OziExplorer-native WPT format is not produced.

LizaAlert volunteers transfer waypoints to handheld navigators that consume legacy
OziExplorer formats. PLT carries tracks; WPT carries waypoints. Without WPT export,
operators cannot deliver waypoints to those navigators in their expected format.

## Decision

Add WPT (OziExplorer waypoint format) export for waypoint layers as part of the MVP.

- The export covers all waypoints in the active waypoint layer.
- A WPT export dialog defaults to the active bundle's track folder (or another
  bundle-relative folder if a separate convention emerges); users may pick another
  destination.
- WPT export sits alongside existing GPX track export: the export menu offers GPX
  (tracks/waypoints), PLT (tracks), and WPT (waypoints).

The detailed format specification (column layout, datum line, symbol mapping) is
left to the implementation plan. The infrastructure layer adapter pattern in
`src-tauri/src/infrastructure/` is the home for the new exporter, mirroring
`infrastructure/export/plt.rs`.

## Consequences

### Positive

- Closes the gap between tracks (GPX/PLT) and waypoints (GPX only) in delivery to
  navigators that expect OziExplorer-native formats.
- Keeps the export taxonomy consistent (one format per format, regardless of geometry
  type).

### Negative

- Symbol/icon vocabulary must be mapped from internal symbol identifiers to the
  WPT symbol numeric codes. Mismatch falls back to a default symbol; this loss is
  documented but not blocking.
- Datum/projection assumptions must be explicit in the WPT header line — the
  `domain` layer stores coordinates as WGS-84 lat/lon, so the WPT header is fixed
  to WGS-84.

## Out of scope

- WPT *import* — not part of MVP. Waypoint import remains GPX-only for now.
- Round-trip fidelity tests beyond core fields (name, position, symbol, comment).

## Related

- ADR-0020 — MVP scope (lists WPT export as MVP-must, references this ADR)
