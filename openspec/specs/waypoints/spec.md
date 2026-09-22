# waypoints Specification

## Purpose
Covers waypoint layers as user data: placing, moving, renaming, deleting and hiding waypoints, the optional symbol attached to each waypoint, and exporting a waypoint layer to GPX or OziExplorer WPT. Undo semantics come from `undo-redo`; waypoints arriving from GPX files are created by `track-import`.

### Decision history

- ADR-0018 (2026-04-06, accepted): `Waypoint.symbol` is `Option<String>`, the known set lives in the frontend picker and `None` means the default marker; rationale: new symbols without a domain change or project-file migration, and saved projects with unknown symbols degrade gracefully. Codified as: Waypoints support an optional symbol; Waypoint symbol is an open optional string. Not codified: the ADR's emoji table (rendering detail; the current `SymbolPicker` uses different glyphs for `flag`, `camp`, `meeting-point` and `finish`) and map markers — `MapView` draws a plain `waypoint-marker` element, so the glyph appears only in the Waypoints panel.
- ADR-0022 (2026-04-28, accepted): add OziExplorer WPT export for waypoint layers; rationale: field navigators consume OziExplorer formats and PLT cannot carry waypoints. Codified as: System exports waypoints to OziExplorer WPT (tightened in `codify-architecture-decisions` to the exact 1.1 layout, cp1251/CRLF and the symbol-code fallback). Superseded by the implementation: the ADR's default folder ("the bundle's track folder") — the dialog pre-fills `<bundle>/<layer>.wpt` at the bundle root; "optional elevation and timestamp" — waypoints model neither, WPT writes `-777` and `0`; the ADR's claim that the export menu offers GPX for waypoints — no command or UI wires `build_waypoint_gpx_xml`.
- Change `add-wpt-waypoint-export` (2026-05-17): removed the mistaken "waypoints to PLT" requirement (PLT is a track format) and introduced the WPT writer.
- Changes `add-waypoint-visibility-toggle` and `fix-non-destructive-waypoint-rendering`: per-waypoint visibility outside the undo history and non-destructive marker updates; codified as: Multiple waypoints render simultaneously with per-waypoint visibility.

- "System exports waypoints to GPX" was a target requirement with no command or UI behind it; the owner kept it on 2026-09-19 ("это по-любасу нужно") and it was implemented on 2026-09-21 — both the Waypoints tab row menu and the Waypoint Inspector offer GPX and WPT.
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

### Requirement: Waypoints can be found by name

The Waypoints tab SHALL offer a search field that narrows the listed waypoints to
those whose name contains the query, matched case-insensitively as a substring in
either alphabet. While a query is active the tab SHALL report how many waypoints
are shown out of the total and SHALL offer a way to clear the query. The matching
rule SHALL be the same one the Tracks tab uses.

#### Scenario: Narrowing a long list

- **WHEN** a project holds waypoints named `ШТАБ`, `Задача 1` and `Задача 2` and the operator types `задача`
- **THEN** only `Задача 1` and `Задача 2` are listed and the tab reports 2 of 3

#### Scenario: No waypoint matches

- **WHEN** the query matches no waypoint name
- **THEN** the tab says so, distinctly from the empty state shown when the project has no waypoints at all

#### Scenario: Clearing the query

- **WHEN** the operator clears the search field
- **THEN** every waypoint is listed again

### Requirement: Waypoint visibility can be changed for all waypoints at once

The system SHALL provide commands to show every waypoint and to hide every
waypoint in the project, each applied as one operation rather than per-waypoint
toggles, and SHALL provide an action that leaves one waypoint visible and hides
the rest. The Waypoints tab SHALL expose all three. Visibility remains a
non-undoable style mutation, so these operations SHALL mark the project dirty
without adding undo entries.

#### Scenario: Hiding every waypoint

- **WHEN** a project holds visible waypoints across several layers and the operator chooses hide-all
- **THEN** every waypoint becomes hidden in a single state update

#### Scenario: Isolating one waypoint

- **WHEN** the operator picks "only this one" on a waypoint's row
- **THEN** that waypoint is visible and every other waypoint in every layer is hidden

#### Scenario: Isolating a waypoint that does not exist

- **WHEN** the requested waypoint is absent from the project
- **THEN** visibility is left as it was rather than hiding everything

### Requirement: A waypoint row shows where it is and can put it on the map

Each waypoint row SHALL display the waypoint's coordinates, and SHALL offer a
control that moves the map to that waypoint without changing which waypoint is
selected for editing.

#### Scenario: Locating a waypoint

- **WHEN** the operator presses "show on map" on a waypoint row
- **THEN** the map centres on that waypoint's coordinates

#### Scenario: Reading a waypoint's position

- **WHEN** a waypoint sits at 55.75123, 37.61754
- **THEN** its row shows those coordinates to five decimal places, the same precision the track point list uses

### Requirement: Only the newest waypoint-list reload may be shown

The waypoint list SHALL show the result of the most recently started reload
only, and SHALL discard an earlier reload's rows and an earlier reload's
failure alike. Reloads overlap because the list reloads on every
application-state change.

#### Scenario: A reload overtaken while a bundle downloads

- **WHEN** a waypoint-list reload is still running and a newer one completes first
- **THEN** the rows shown are the newer reload's, and the older one changes nothing when it finishes

### Requirement: Waypoint layers are read together

A reload of the waypoint list SHALL request every layer's waypoints
concurrently rather than one layer after another, so that the cost of the list
does not grow with a round trip per layer.

#### Scenario: A project of many import-created layers

- **WHEN** the waypoint list is reloaded for a project with many waypoint layers
- **THEN** the layers are requested together

### Requirement: A waypoint exported to GPX and read back is the same waypoint

A waypoint written to GPX and imported again SHALL carry the same name, the
same coordinates, and the same symbol — including the absence of one, because a
waypoint that acquires a symbol on the way back puts a mark on the map that
nobody placed.

#### Scenario: Waypoints go out and come back

- **WHEN** waypoints, one with a symbol and one without, are exported to GPX and imported again
- **THEN** each carries its name, its coordinates and its symbol, and the one without a symbol still has none

### Requirement: A waypoint's symbol is drawn on the map

A waypoint's symbol SHALL be shown on its map marker, not only in the lists and
the exports, because the map is where a crew reads which mark is which — the
task point, what was found, where the danger is.

The marker SHALL remain findable as a marker: the symbol is drawn on a shape
that stands out against imagery rather than alone. A symbol the build does not
recognise SHALL fall back to the default marker rather than leaving the
waypoint undrawn.

Changing a waypoint's symbol SHALL redraw its marker.

#### Scenario: Marks of different kinds on one search

- **WHEN** the map shows a waypoint with a symbol and one without
- **THEN** the first draws that symbol's glyph and the second the default one, and both are legible as markers

#### Scenario: A symbol from another tool

- **WHEN** a waypoint carries a symbol this build does not know
- **THEN** its marker is drawn with the default symbol

### Requirement: A waypoint can carry a colour of its own

A waypoint SHALL be able to carry a colour, set and cleared by the operator,
and the map SHALL draw its marker in that colour. The symbol says what a mark
is; the colour says whose it is, and a search collects marks from every group
working it.

Having no colour SHALL NOT be the same as having the default one: a waypoint
that has never been coloured SHALL follow whatever the map draws waypoints
with, and clearing a colour SHALL return it to that rather than setting a
colour that resembles it.

Setting or clearing a colour SHALL be undoable. A project saved before
waypoints could carry a colour SHALL load with every waypoint uncoloured.

Every surface that draws a waypoint — the marker, the row in the list and the
inspector — SHALL draw it in the same colour, because the list is what a crew
reads to find what they are looking at on the map.

#### Scenario: Two groups' marks on one map

- **WHEN** one waypoint is given a colour and another is left without
- **THEN** the map draws the first in that colour and the second in the default, and each row in the list matches its marker

#### Scenario: Clearing a colour

- **WHEN** the operator clears a waypoint's colour
- **THEN** the waypoint follows the default again, and the change can be undone

#### Scenario: A project from before this existed

- **WHEN** a project saved without waypoint colours is loaded
- **THEN** it loads, with every waypoint uncoloured

### Requirement: Marks that could not be read are not marks that are gone

When the marks of a layer cannot be read, the system SHALL treat the layer's
contents as unknown rather than empty: the markers already on the map SHALL
stay, and the operator SHALL be told the read failed.

#### Scenario: One layer's read fails during a refresh

- **WHEN** reading one layer's marks fails while the others succeed
- **THEN** that layer's markers remain on the map, the others are refreshed, and the failure is reported

### Requirement: Only the newest waypoint-marker refresh may draw

The map SHALL draw the marker set of the most recently started refresh only,
and SHALL discard an earlier refresh's result, so that a waypoint the operator
has just added or moved is not undone by a refresh that started before it.
Refreshes overlap because the map refreshes both on state changes and on every
waypoint action.

The map SHALL request every visible layer's waypoints concurrently rather than
one layer after another.

#### Scenario: A waypoint added while a refresh is in flight

- **WHEN** a waypoint is added while an earlier marker refresh is still running
- **AND** that earlier refresh completes afterwards
- **THEN** the markers on the map include the new waypoint

### Requirement: A refused waypoint edit tells the operator

Adding or moving a waypoint that the system declines SHALL be reported to the
operator with the reason, rather than only to a developer console. A failure to
load a waypoint for inspection SHALL likewise be reported instead of leaving an
empty pane.

#### Scenario: Placing a waypoint that is declined

- **WHEN** adding a waypoint by clicking the map is declined
- **THEN** the operator is told, with the reason

#### Scenario: A waypoint that cannot be loaded for inspection

- **WHEN** loading a waypoint's detail fails
- **THEN** the operator is told rather than shown an empty inspector

