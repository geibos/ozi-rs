# Roadmap

## Phase 0 - Kickoff

- define scope and non-goals;
- record initial architecture;
- define testing strategy;
- create prioritized backlog.

Status: complete in documentation, with early implementation underway.

## Phase 1 - Core Model And Commands

Target outcomes:
- initial Rust workspace/application skeleton;
- domain entities for project, layers, tracks, and waypoints;
- explicit command model for edits;
- undo/redo foundation.

Priority backlog:
1. scaffold Rust project structure around `domain`, `application`, `infrastructure`, and `ui` boundaries;
2. implement core entities and identifiers;
3. ensure map switching does not implicitly discard loaded tracks or waypoints;
4. implement command abstractions and reversible operations;
5. add unit and workflow tests for core edits.

## Phase 2 - Project Persistence

Target outcomes:
- save/load for internal project representation;
- persistence boundaries separated from UI state;
- integration tests for round-trip behavior.

Priority backlog:
1. define persisted project schema;
2. implement serialization boundaries;
3. add integration tests for project round-trips.

## Phase 3 - Data Import/Export

Target outcomes:
- first supported import/export paths for tracks and waypoints;
- failure handling for malformed files;
- regression tests for parser edge cases.

Priority backlog:
1. add ZIP archive inventory and supported-entry detection in the infrastructure layer;
2. implement GPX import as the first archive-backed track/waypoint path;
3. support imported-data triage workflows so users can inspect results and remove irrelevant tracks or point sets;
4. add later adapters for KML and Ozi text formats (`.plt`, `.wpt`, `.map`) once the archive boundary is stable;
5. investigate `ozf2` feasibility separately before committing to native raster support;
6. implement export adapters;
7. add parser and round-trip tests;
8. add clear error reporting for common file-open and import failures.

## Phase 4 - Initial UI Workflow

Target outcomes:
- open a project with map, track, and waypoint layers;
- perform basic command-driven edits from the UI;
- view and manipulate selection state outside persisted domain entities.

Priority backlog:
1. define minimal UI shell;
2. connect UI actions to application commands;
3. add smoke-level workflow coverage where feasible.

## Phase 5 - Workflow Expansion

Candidate areas after the core is stable:
- richer track editing workflows;
- polygon-like or task-oriented workflows if they fit the modern model;
- configurable styles and templates;
- settings and workflow helpers inspired by OziExplorer add-ons, filtered through current product goals.

## Yonote-Derived Triage

Adopt soon:
- GPX-oriented import workflows;
- ZIP-backed GPX import so field data can arrive bundled with other artifacts;
- imported-track review and cleanup;
- project semantics where field data survives map changes;
- clearer file-open diagnostics.

Defer pending stronger product evidence or architecture maturity:
- multi-map helper workflows inspired by OziManyMaps;
- overlay/reference layers such as wiki, hybrid, OSM, or archive views;
- KML and Ozi text import after the first archive-backed GPX slice is stable;
- style and naming templates for tracks;
- explicit normalization helpers for tracks and waypoints.

Reject or heavily reformulate:
- GPS device sync;
- COM-port configuration UX;
- legacy privileged track or callsign behavior;
- helper-specific hidden config files as the primary user workflow.

## Deferred Until Better Evidence

- any feature that depends heavily on screenshot-only Yonote material not yet fully extracted;
- native `ozf2` decoding until feasibility and licensing are clearer;
- advanced GIS/projection behavior;
- GPS-device workflows;
- legacy workflow quirks that conflict with explicit commands or clean data boundaries.
