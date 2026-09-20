# waypoints Specification

## Purpose
Covers waypoint layers as user data: placing, moving, renaming, deleting and hiding waypoints, the optional symbol attached to each waypoint, and exporting a waypoint layer to OziExplorer WPT. Undo semantics come from `undo-redo`; waypoints arriving from GPX files are created by `track-import`.

### Decision history

- ADR-0018 (2026-04-06, accepted): `Waypoint.symbol` is `Option<String>`, the known set lives in the frontend picker and `None` means the default marker; rationale: new symbols without a domain change or project-file migration, and saved projects with unknown symbols degrade gracefully. Codified as: Waypoints support an optional symbol; Waypoint symbol is an open optional string. Not codified: the ADR's emoji table (rendering detail; the current `SymbolPicker` uses different glyphs for `flag`, `camp`, `meeting-point` and `finish`) and map markers — `MapView` draws a plain `waypoint-marker` element, so the glyph appears only in the Waypoints panel.
- ADR-0022 (2026-04-28, accepted): add OziExplorer WPT export for waypoint layers; rationale: field navigators consume OziExplorer formats and PLT cannot carry waypoints. Codified as: System exports waypoints to OziExplorer WPT (tightened in `codify-architecture-decisions` to the exact 1.1 layout, cp1251/CRLF and the symbol-code fallback). Superseded by the implementation: the ADR's default folder ("the bundle's track folder") — the dialog pre-fills `<bundle>/<layer>.wpt` at the bundle root; "optional elevation and timestamp" — waypoints model neither, WPT writes `-777` and `0`; the ADR's claim that the export menu offers GPX for waypoints — no command or UI wires `build_waypoint_gpx_xml`.
- Change `add-wpt-waypoint-export` (2026-05-17): removed the mistaken "waypoints to PLT" requirement (PLT is a track format) and introduced the WPT writer.
- Changes `add-waypoint-visibility-toggle` and `fix-non-destructive-waypoint-rendering`: per-waypoint visibility outside the undo history and non-destructive marker updates; codified as: Multiple waypoints render simultaneously with per-waypoint visibility.

- Owner decision (2026-09-19): "System exports waypoints to GPX" stays as a target requirement even though no command or UI exists yet; implementation is scheduled with CJ-6 (revive-ui-cycle roadmap, phase 4).

## Requirements
### Requirement: User can add a waypoint by clicking on the map

The system SHALL allow the user to enter waypoint placement mode and click on the map to create a new waypoint in the active waypoint layer at the clicked coordinates as an undoable edit.

#### Scenario: Place a waypoint

- **WHEN** the user activates waypoint placement and clicks on the map
- **THEN** a new waypoint marker appears at the clicked coordinates, appears in the Waypoints panel, and the creation is reversible via undo

### Requirement: User can move a waypoint by dragging on the map

The system SHALL allow dragging waypoint markers; drag-and-release SHALL commit the new coordinates as an undoable edit, with drag coalescing per the `undo-redo` spec.

#### Scenario: Drag a waypoint

- **WHEN** the user drags a waypoint marker to a new location
- **THEN** the waypoint's coordinates update and a single coalesced undo step is added

### Requirement: User can rename a waypoint

The system SHALL allow renaming a waypoint via the Waypoints panel as an undoable edit; the map label SHALL update to reflect the new name.

#### Scenario: Rename a waypoint

- **WHEN** the user edits a waypoint name in the panel and confirms
- **THEN** the new name appears in the panel, on the map label, and is reversible via undo

### Requirement: User can delete a waypoint

The system SHALL allow deleting a waypoint as an undoable edit. The marker and panel row SHALL disappear; undo SHALL restore the full waypoint including its symbol.

#### Scenario: Delete and undo restores symbol

- **WHEN** the user deletes a waypoint that has a symbol set, then invokes undo
- **THEN** the waypoint reappears with its previous coordinates, name, and symbol intact

### Requirement: Waypoints support an optional symbol

The system SHALL allow assigning an optional symbol (e.g. flag, camp, danger, water, shelter) to each waypoint. Symbol changes SHALL be undoable, with the previous symbol value (including "no symbol") stored in the inverse command.

#### Scenario: Symbol picker commits choice

- **WHEN** the user picks a "shelter" symbol for a waypoint
- **THEN** the waypoint renders with the shelter glyph and undo restores the previous symbol (or absence of symbol)

### Requirement: Multiple waypoints render simultaneously with per-waypoint visibility

The system SHALL render all waypoints in visible layers simultaneously and SHALL allow toggling individual waypoint visibility; hidden waypoints SHALL NOT render on the map.

#### Scenario: Toggle a waypoint hidden

- **WHEN** the user toggles a waypoint's visibility off
- **THEN** its marker disappears from the map but the panel row remains and indicates hidden state

#### Scenario: Toggle visibility off and back on

- **WHEN** the user toggles a waypoint's visibility off via the Waypoints panel and then toggles it on again
- **THEN** the marker disappears from the map after the first toggle and reappears unchanged after the second; the panel row remains visible during both states

#### Scenario: Visibility toggle is not undoable

- **WHEN** the user toggles a waypoint's visibility off and then invokes undo
- **THEN** undo reverts the most recent undoable edit (not the visibility toggle) and the waypoint stays hidden

### Requirement: System exports waypoints to GPX

The system SHALL provide an "Export waypoints to GPX" action that writes all waypoints in a chosen layer to a user-selected `.gpx` file.

#### Scenario: Export waypoints to GPX

- **WHEN** the user exports waypoints from a layer with three named waypoints
- **THEN** the resulting `.gpx` contains three `<wpt>` elements with names, coordinates, and (where present) symbols

### Requirement: System exports waypoints to OziExplorer WPT

The system SHALL provide an "Export waypoints (WPT)" action that writes all waypoints in a chosen layer to a user-selected `.wpt` file in the OziExplorer WPT format (version 1.1 header, latitude/longitude in WGS84 decimal degrees, optional symbol code, optional elevation and timestamp).

#### Scenario: Export waypoints with symbols to WPT

- **WHEN** the user invokes "Export waypoints (WPT)" on a layer with three waypoints, two of which have symbols
- **THEN** the resulting `.wpt` file contains exactly three rows in OziExplorer 1.1 WPT format with symbol codes preserved for the two waypoints and a default symbol for the third

#### Scenario: WPT export dialog suggests `<bundle>/<layer>.wpt`

- **WHEN** the user invokes WPT export with a bundle active and a waypoint layer named `points`
- **THEN** the file picker pre-fills with `<bundle>/points.wpt`

#### Scenario: WPT export with no active bundle suggests filename only

- **WHEN** the user invokes WPT export with no active bundle
- **THEN** the file picker suggests `<layer>.wpt` only, without a directory

