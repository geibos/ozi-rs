# ADR-0002: Separate Map Bundle from Project

- Status: accepted
- Date: 2026-03-29

## Context

Early versions of the application conflated the map source (a downloaded tile set or
raster map) with the project (a SAR search operation containing tracks and waypoints).
This caused two problems:

1. Loading a different map implicitly affected or lost loaded track data.
2. There was no natural place to store multiple projects that share the same geographic
   map coverage.

LizaAlert volunteers download map bundles for a geographic area once and then run
multiple separate search operations in that area over time.

## Decision

Treat **map bundle** and **project** as distinct concepts with a clear boundary:

**Map bundle** — a directory on disk containing one or more georeferenced raster maps
for a geographic area. Bundles are downloaded from maps.lizaalert.ru or opened from a
local folder. The bundle directory is the unit of storage and sharing. Multiple projects
may reference the same bundle.

**Project** — one SAR search operation. A project holds track layers and waypoint
layers, references an active map from a bundle, and is saved as a standalone `.json`
file. The project does not own the bundle directory.

The application maintains a configurable **bundles root** directory. Individual bundles
live as subdirectories inside it.

## Consequences

### Positive

- Switching the active map within a project does not discard tracks or waypoints.
- Multiple projects can reference the same bundle without duplicating map data.
- The `10-Tracks/` subfolder convention (LizaAlert standard) has a natural home inside
  the bundle directory, shared across all projects that use it.
- Bundle download and project management are clearly separate UI workflows.

### Negative

- The application must track both a bundle root path and a project file path in
  persistent settings.
- Opening a project on a different machine requires the bundle to be present at a
  matching path, or the active map reference must be re-established manually.

## Rejected Alternatives

### Project owns the bundle

Rejected because it duplicates map data when multiple projects cover the same area,
and because bundles can be tens of gigabytes.

### Single monolithic workspace

Rejected because it conflates map provenance with search operation data and makes
project portability harder.

## Follow-Up

- Define how a project records its active map reference (currently: `ActiveMapSelection`
  with a `local_path` stored in eframe persistent storage).
- Consider how to handle the case where a referenced bundle path no longer exists when
  the project is reopened.
