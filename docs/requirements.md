# Requirements

## Product Goal

Build a Rust desktop application for working with raster maps, tracks, and waypoints as a modern replacement for the useful core of OziExplorer.

The product must favor predictable behavior, explicit edits, and a testable architecture over legacy convenience shortcuts.

## Primary Users

- operators who open raster maps and inspect field data;
- users who import, create, and edit tracks;
- users who manage waypoint collections;
- contributors who need a maintainable and testable codebase.

## Core User Workflows

### 1. Open map and inspect project context
- open a raster map file;
- view it as an independent map layer;
- load related tracks and waypoint collections into the same project.

### 2. Import and organize field data
- import one or more tracks;
- import one or more waypoint collections;
- inspect imported tracks to keep relevant data and discard noise;
- keep tracks and waypoint collections as separate first-class entities.

### 3. Edit waypoints
- add waypoint;
- move waypoint;
- rename waypoint;
- delete selected waypoint(s).

### 4. Edit tracks
- create track;
- insert track point;
- move track point;
- split segment;
- join segments;
- delete selected elements.

### 5. Save and export
- save the current project state;
- export supported data in external formats.

## MVP Scope

The MVP should include:
- raster map opening;
- project model with independent map, track, and waypoint layers;
- loading multiple tracks and waypoint collections;
- explicit command-driven editing for tracks and waypoints;
- undo/redo support via reversible commands or equivalent operation log;
- project save/load;
- export for core supported data.

## MVP Non-Goals

The MVP should not include:
- datum management;
- advanced geodesy or projection systems beyond strict necessity;
- GPS device sync;
- routes, events, or live telemetry;
- legacy privileged objects such as `Track 1`;
- append-only editing models copied from legacy GIS workflows.

## Functional Requirements

### Project Model
- The system must represent `Project`, `MapLayer`, `TrackLayer`, `WaypointLayer`, `Track`, `TrackSegment`, `TrackPoint`, and `Waypoint` as distinct concepts.
- The system must keep map, track, and waypoint data independent so changing the active map does not implicitly discard loaded field data.
- UI-only selection or edit state should remain outside persisted domain entities where practical.

### Editing
- The system must express edits as explicit commands.
- The system must support reversible edits suitable for undo/redo.
- The system must avoid hidden mutable global state during editing workflows.

### Architecture
- Domain logic must not depend on GUI types.
- Domain entities must remain serializable and testable without UI runtime dependencies.
- Persistence formats must be treated as boundaries, not as the domain model itself.

### Quality
- Every non-trivial feature must ship with tests.
- New functionality must update relevant docs.
- The application must remain aligned with the stated product intent and non-goals.
- Common failure cases such as unsupported access paths or problematic file names should produce clear user-facing errors.

## Acceptance Criteria For Kickoff

- product scope and non-goals are documented;
- architecture boundaries are documented;
- testing strategy is documented;
- implementation roadmap is prioritized;
- initial architecture ADR is recorded.

## Assumptions

- the first implementation phase should prioritize correctness and edit-model clarity over UI polish;
- import/export format breadth can grow incrementally after the project model and command system are stable;
- external OziExplorer add-on references will inform backlog refinement, but they do not override the product constraints in `AGENTS.md`.

## Yonote-Derived Backlog Signals

The currently reviewed Yonote/OziExplorer material suggests these product directions:

Adopt into MVP planning:
- GPX track import;
- GPX waypoint import;
- imported-data triage flows for keeping relevant tracks and removing irrelevant ones;
- preserving loaded tracks and waypoints when the visible map changes;
- explicit handling for common file-open and import failures.

Defer until after the core edit model is stable:
- multi-map convenience workflows similar to OziManyMaps, but reformulated as first-class map/layer management rather than an external helper;
- optional overlay and reference layers;
- reusable style or naming templates for tracks, without legacy privileged categories;
- workflow helpers or normalization tools only after their rules are made explicit and testable.

Reject or reformulate:
- GPS device synchronization during MVP;
- hardware-specific COM configuration;
- polygon-centric workflows not already justified by the MVP model;
- domain-specific privileged callsigns or other hard-coded legacy exceptions.

## Open Questions

- which file formats should be supported first for map loading, track import/export, and waypoint import/export;
- what minimum geometry support is required for split/join and selection workflows;
- which OziExplorer-derived workflows from the Yonote references should land in MVP versus post-MVP once the screenshot-heavy material is fully extracted.
