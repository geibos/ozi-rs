# Requirements

## Product Goal

Build a Rust desktop application for LizaAlert search-and-rescue volunteers to work with
raster maps, tracks, and waypoints offline — as a modern replacement for the useful core
of OziExplorer.

The product must favor predictable behavior, explicit edits, and a testable architecture
over legacy convenience shortcuts.

## Context

[LizaAlert](https://lizaalert.org) is a Russian volunteer SAR organization. Volunteers
at field HQ use laptop computers to work with topo and satellite raster maps downloaded
from [maps.lizaalert.ru](https://maps.lizaalert.ru). Field teams record GPS tracks
(`.plt`, `.gpx`) that are later loaded and analyzed at HQ.

The LizaAlert operational cartography standard defines naming and file layout
conventions that the application must support and validate.

## Primary Users

- **HQ operators** who open raster maps, load field tracks, and organize project data
- **Track analysts** who import, review, simplify, and export GPS tracks
- **Cartographers** who manage map bundles and maintain naming standards
- **Contributors** who need a maintainable and testable codebase

## Key Concepts

### Map Bundle

A directory containing one or more georeferenced raster maps for a geographic area.
Bundles are downloaded from maps.lizaalert.ru or opened from a local folder.
Stored in a user-configurable location. One bundle can be used by multiple projects.

Typical bundle layout:

```
BundleName/
  map.sqlitedb        # SQLite tile map
  map.map + map.ozf2  # OziExplorer raster map
  10-Tracks/          # tracks exported from this bundle's projects
```

### Project

One SAR search operation. A project:
- references a map bundle
- contains track layers and waypoint layers
- is saved as a local `.json` file
- stores exported tracks in a `10-Tracks/` subfolder per the LizaAlert standard

### LizaAlert OK Standard

Track names must follow the pattern `YYYYMMDD_Callsign`
(e.g. `20240601_Иванов`). The UI warns when a name does not match.

## Core User Workflows

### 1. Open map and load field data

- select a LizaAlert project from maps.lizaalert.ru, or open a local bundle
- download or use a cached tile map or OZF2 raster map
- import one or more GPX or PLT track files into the project
- view tracks overlaid on the map

### 2. Review and organize tracks

- inspect per-track statistics (distance, duration, point count)
- rename tracks to follow the OK standard
- toggle track visibility
- change track color and line width
- remove tracks that are noise or duplicates

### 3. Edit tracks

- move or delete individual track points
- split segment at a point
- join adjacent segments
- simplify track with Douglas-Peucker

### 4. Edit waypoints

- add waypoint by clicking on map
- move waypoint
- rename waypoint
- delete waypoint

### 5. Save and export

- save project state
- export track layer to GPX (with Garmin color extension)
- export to `10-Tracks/` subfolder with suggested filename

## Track Feature Requirements

These requirements came from direct user input and must be reflected in design:

- **Import**: GPX (single file and ZIP archive), PLT (including Windows-1251 encoding)
- **Export**: GPX with Garmin color extension; PLT (planned)
- **Display**: name, color, line width, visibility, opacity per track
- **Simplification**: Douglas-Peucker with configurable tolerance (not stride-based)
- **Point list**: all properties per point (lat/lon, elevation, timestamp, segment)
- **Sort by timestamp**: fix out-of-order GPS recordings
- **Edit mode**: move points by drag on map, delete selected points
- **Segment ops**: split at point, join adjacent segments
- **Undo/redo**: all edits reversible via command stack
- **Statistics**: total distance (km), duration (h/m), point count

## MVP Scope

- raster map opening (SQLite tiles, OZF2)
- project model with independent map, track, and waypoint layers
- loading multiple tracks and waypoint collections
- explicit command-driven editing for tracks and waypoints
- undo/redo support
- project save/load
- GPX and PLT import/export
- LizaAlert project browser and bundle management

## MVP Non-Goals

- datum management
- advanced geodesy or projection systems beyond strict necessity
- GPS device sync and live telemetry
- routes and events
- legacy privileged objects such as `Track 1`
- append-only editing models copied from legacy GIS workflows
- polygon / search sector drawing (post-MVP)
- multi-device coordination

## Functional Requirements

### Project Model

- The system must represent `Project`, `MapLayer`, `TrackLayer`, `WaypointLayer`,
  `Track`, `TrackSegment`, `TrackPoint`, and `Waypoint` as distinct concepts.
- Map bundle and project must be separate concepts. Changing the active map must not
  discard loaded tracks or waypoints.
- UI-only selection or edit state must remain outside persisted domain entities.

### Editing

- All non-trivial edits must be expressed as explicit commands.
- The command stack must support undo/redo.
- The system must avoid hidden mutable global state during editing workflows.

### LizaAlert Integration

- The application must be able to browse and download projects from maps.lizaalert.ru.
- The application must validate track names against the `YYYYMMDD_Callsign` pattern and
  surface a warning for non-conforming names.
- The application must suggest `10-Tracks/` as the export destination when a bundle is
  active.

### Architecture

- Domain logic must not depend on GUI types.
- Domain entities must remain serializable and testable without UI runtime dependencies.
- Persistence formats must be treated as boundaries, not as the domain model itself.

### Quality

- Every non-trivial feature must ship with tests.
- New functionality must update relevant docs.
- Common failure cases must produce clear user-facing errors (surfaced in the console
  and via `tracing` to stdout/stderr).

## Open Questions

- Which UI framework best supports drag-based point editing and native multi-window
  layout? (egui has known limitations; iced and slint are candidates.)
- Should sector/polygon drawing be added to the geometry model now to avoid a retrofit
  later?
- What is the priority of PLT export relative to track editing features?
