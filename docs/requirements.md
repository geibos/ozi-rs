# Requirements

## Product Goal

Build a Tauri 2 desktop application for LizaAlert search-and-rescue volunteers to work with
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
- is saved as a local `.ozp` file (JSON)
- suggests the active bundle's `10-Tracks/` subfolder for GPX/PLT track exports per the LizaAlert standard

### LizaAlert OK Standard

Track names must follow the pattern `YYYYMMDD_Callsign`
(e.g. `20240601_Иванов`). The UI warns when a name does not match.

## Core User Workflows

### 1. Open map and load field data

- Select a LizaAlert project from maps.lizaalert.ru, or open a local bundle
- Download or use a cached tile map or OZF2 raster map
- Import one or more GPX or PLT track files into the project
- View tracks overlaid on the map

### 2. Review and organize tracks

- Inspect per-track statistics (distance, duration, point count)
- Rename tracks and review warning-only OK-standard feedback
- Toggle track visibility
- Change track color and line width
- Remove tracks that are noise or duplicates

### 3. Edit tracks

- Move or delete individual track points
- Insert new track points
- Split segment at a point
- Join adjacent segments
- Create new tracks by drawing on map
- Simplify track with Douglas-Peucker (configurable tolerance, live preview)

### 4. Edit waypoints

- Add waypoint by clicking on map
- Move waypoint by drag
- Rename waypoint
- Delete waypoint
- Assign symbol (flag, camp, danger, water, shelter, etc.)

### 5. Save and export

- Save project state (`.ozp`)
- Export track layer to GPX (with Garmin color extension)
- Export individual tracks to PLT (OziExplorer format)
- Export to the active bundle's `10-Tracks/` subfolder with a suggested filename when a bundle is known

## Track Feature Requirements

These requirements came from direct user input:

- **Import**: GPX (single file and ZIP archive), PLT (including Windows-1251 encoding) — done
- **Export**: GPX with Garmin color extension, PLT with OLE dates and COLORREF BGR — done
- **Display**: name, color, line width, visibility, opacity per track — done
- **Simplification**: Douglas-Peucker with configurable tolerance and live preview — done
- **Point list**: all properties per point (lat/lon, elevation, timestamp, segment) — done
- **Sort by timestamp**: fix out-of-order GPS recordings — deferred
- **Edit mode**: move points by drag on map, delete/insert via context menu — done
- **Segment ops**: split at point, join adjacent segments — done
- **Drawing mode**: create track from scratch on map — done
- **Undo/redo**: track geometry, track/waypoint CRUD, waypoint moves/renames/symbols, drawing, and simplification are reversible via delta-based command stack; track style changes remain immediate non-undoable mutations — done
- **Statistics**: total distance (km), duration (h/m), point count — done

## MVP Scope

- Raster map opening (SQLite tiles, OZF2) — done
- Project model with independent map, track, and waypoint layers — backend done; UI surfaces active-layer selection, not full layer management
- Loading multiple tracks and waypoint collections — done
- Explicit command-driven editing for tracks and waypoints — done
- Delta-based undo/redo with drag coalescing — done
- Project save/load (JSON `.ozp`) — done
- GPX and PLT import/export — done
- LizaAlert project browser and bundle management — done
- Track editing (move, delete, insert, split, join, draw) — done
- Track simplification with preview — done
- Waypoint editing UI with undoable symbols — done

## MVP Non-Goals

- Datum management
- Advanced geodesy or projection systems beyond strict necessity
- GPS device sync and live telemetry
- Routes and events
- Legacy privileged objects such as `Track 1`
- Append-only editing models copied from legacy GIS workflows
- Polygon / search sector drawing (post-MVP)
- Multi-device coordination
- Print to PDF (deferred to post-MVP)

## Functional Requirements

### Project Model

- The system must represent `Project`, `MapLayer`, `TrackLayer`, `WaypointLayer`,
  `Track`, `TrackSegment`, `TrackPoint`, and `Waypoint` as distinct concepts. — done
- Map bundle and project must be separate concepts. Changing the active map must not
  discard loaded tracks or waypoints. — done
- UI-only selection or edit state must remain outside persisted domain entities. — done

### Editing

- All non-trivial edits must be expressed as explicit commands. — done
- The command stack must support undo/redo via delta-based forward/reverse pairs. — done
- Drag operations must coalesce into single undo steps. — done

### LizaAlert Integration

- The application must browse and download projects from maps.lizaalert.ru. — done
- The application must validate track names against the `YYYYMMDD_Callsign` pattern and
  surface a warning for non-conforming names. — done
- The application must suggest the active bundle's `10-Tracks/` directory as the GPX/PLT
  export destination when a bundle is active. — done

### Architecture

- Domain logic must not depend on GUI types. — done
- Domain entities must remain serializable and testable without UI runtime dependencies. — done
- Persistence formats must be treated as boundaries, not as the domain model itself. — done

### Quality

- Every non-trivial feature must ship with tests. — done
- New functionality must update relevant docs.
- Common failure cases must produce clear user-facing errors (surfaced in the console
  and via `tracing` to stdout/stderr). — done
