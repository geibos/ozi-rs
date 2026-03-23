# Architecture

## Overview

The system should be organized around a small, explicit core that keeps domain rules independent from UI concerns.

Target layers:
- `domain`
- `application`
- `infrastructure`
- `ui`

## Architectural Principles

- prioritize correctness, clarity, and testability;
- keep business logic outside the GUI;
- use explicit commands for edits;
- design undo/redo from the beginning rather than as a retrofit;
- avoid reproducing legacy OziExplorer constraints in the data model or UX.

## Layer Responsibilities

### Domain

Owns:
- core entities;
- invariants;
- pure value objects;
- domain-level validation.

Constraints:
- no GUI dependencies;
- no persistence-format leakage;
- deterministic behavior suitable for unit tests.

### Application

Owns:
- use-cases and command handling;
- orchestration of edits;
- undo/redo command stack or equivalent reversible log;
- transactions across domain objects when needed.

Constraints:
- may depend on domain;
- should expose stable operations to the UI;
- should remain testable with lightweight fakes.

### Infrastructure

Owns:
- file parsing and serialization;
- import/export adapters;
- project persistence;
- storage and format boundaries.

Constraints:
- may depend on domain and application contracts;
- should keep format-specific behavior out of the core model.

### UI

Owns:
- rendering;
- interaction state;
- selections and view state;
- command dispatch into the application layer.

Constraints:
- should act as an adapter, not as the home for business rules;
- UI state should not leak into persisted domain entities unless justified.

## Initial Domain Model

The initial model should include at least:
- `Project`
- `MapLayer`
- `TrackLayer`
- `WaypointLayer`
- `Track`
- `TrackSegment`
- `TrackPoint`
- `Waypoint`

Likely supporting concepts:
- identifiers for stable references;
- geometry primitives for coordinates and bounds;
- selection state outside persistence;
- edit context managed by UI/application, not by domain entities.

## Editing Model

All non-trivial edits should flow through explicit commands such as:
- add waypoint;
- move waypoint;
- rename waypoint;
- create track;
- insert track point;
- move track point;
- split track segment;
- join segments;
- delete selection.

This gives the application layer a natural seam for:
- validation before mutation;
- inverse operations for undo/redo;
- workflow tests at the use-case level.

## Persistence Boundaries

- project persistence should capture the user-owned project state;
- import/export adapters should translate external formats into the internal model;
- UI-only state should remain transient unless there is a strong documented reason to persist it.

## Suggested Initial Module Layout

```text
src/
  domain/
    project/
    map/
    track/
    waypoint/
    geometry/
  application/
    commands/
    undo/
    services/
  infrastructure/
    io/
    persistence/
    formats/
  ui/
    view/
    state/
    interaction/
```

## Architectural Risks

- letting UI concerns leak into the domain too early;
- tying the project model too closely to a specific file format;
- under-designing undo/redo and having to retrofit reversibility later;
- importing legacy workflow assumptions wholesale from OziExplorer add-ons without filtering them through the repo non-goals.

## Near-Term Architectural Decisions

- choose the first persistence boundary for project save/load;
- define command abstractions for reversible edits;
- define minimal geometry primitives required for track and waypoint editing;
- decide the first external formats to support.
