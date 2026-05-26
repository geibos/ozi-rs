## Context

The `layers` spec describes a project as a composition of independent layers (track, waypoint, map), and the active-layer concept routes edits. But neither the spec nor the backend `Project` model requires that any track or waypoint layer exist at any given moment. Today the default constructor produces a project with zero track and zero waypoint layers, the frontend `syncActiveLayer` (`stores.ts:47`) honestly returns `null` when `layers.length === 0`, and every edit pathway has a silent early-return for `layerId === null`.

The user can reach this state in three ways:
1. Open a freshly created project (most common path for new SAR missions).
2. Load a bundle whose `.ozp` was authored externally without layers.
3. (Hypothetical) delete the last layer via a future Layers panel. Not reachable today; mentioned for completeness.

In all three, the failure mode is the same: the user clicks, nothing happens.

## Goals / Non-Goals

**Goals:**
- Guarantee at all times that an open project has ≥1 track layer and ≥1 waypoint layer.
- Make this guarantee a backend invariant, so frontend code does not have to repeat null-checks.
- Migrate legacy `.ozp` files on read, not on save (preserve disk format compatibility).

**Non-Goals:**
- Building a Layers panel UI (create / rename / delete / reorder). That is the deferred "full layer management UI" from `docs/roadmap.md`.
- Preventing the user from renaming default layers. After creation they are ordinary layers.
- Preventing the user from deleting the last layer through some future workflow. When that workflow exists, _that_ change will be responsible for preserving the invariant.
- Distinguishing default layers from user-created layers in the type system. They are not special.

## Decisions

### Decision 1: Default layers are created at Project construction, not lazily

`Project::default()` and any other construction path (load-from-bundle, new-empty) SHALL produce a project that already contains one `TrackLayer` and one `WaypointLayer`. No lazy-create on first edit.

**Rationale**: Lazy-create spreads the invariant across many call sites (every edit operation must check and create). Eager-create concentrates it at construction, which is a much smaller and more testable surface.

**Alternatives considered**:
- _Lazy-create at first edit_: spreads invariant logic into command handlers. Rejected.
- _Refuse to construct a project without explicit layers_: forces callers to know about defaults. Rejected because external `.ozp` loaders cannot reasonably be expected to do this.

### Decision 2: Load-path normalization, not save-path normalization

The `.ozp` deserialization path SHALL check whether the loaded `Project` contains ≥1 track layer and ≥1 waypoint layer. If either count is zero, the loader SHALL append a default layer for the missing kind before returning the project to the application layer.

`.ozp` save format SHALL NOT change. Projects saved before this change deserialize cleanly; projects saved after this change deserialize cleanly in older binaries as long as those binaries can handle layers being present (which they always have).

**Rationale**: Modifying files on read is reversible if we get it wrong; modifying on save is not.

**Alternatives considered**:
- _Migrate on save (rewrite the file the first time a legacy `.ozp` is opened)_: writes side-effect on read, which is surprising for users. Rejected.

### Decision 3: Default layer names

Track default: `"Tracks"`. Waypoint default: `"Waypoints"`. English, simple, matching the panel headings the user already sees. No localization, no per-language defaults — these are immediately renameable, and the rest of the app does not localize layer names either.

### Decision 4: Frontend drops null-guards

`Sidebar.svelte:109` (`if (layerId === null) return;` in `toggleTrackDrawingMode`) SHALL be removed. `MapView.svelte:377` (`if (layerId === null) return;` in `handleMapClickForWaypoint`) SHALL be removed. `stores.ts:syncActiveLayer` SHALL still handle empty arrays defensively (returning null) for the brief instant before `appState` settles, but the type expectation in handlers becomes "non-null when a project is open".

We don't enforce non-null at the type level (the stores remain `bigint | null`) because the "no project open" state is still legitimately null. The invariant is: _when a project is open, IDs are non-null_.

## Risks / Trade-offs

- **Risk**: A user has tooling outside ozi-rs that produces `.ozp` files without layers and relies on layers being optional in the on-disk schema. **Mitigation**: read-side normalization preserves the file as written; only the in-memory representation gets the defaults. The user's external tool keeps working.
- **Risk**: A future deletion workflow violates the invariant by allowing the last layer to be removed. **Mitigation**: that future change is responsible for preserving the invariant (either prevent the deletion or auto-create a replacement). This change does not preempt that decision.
- **Trade-off**: Two empty default layers add a small amount of clutter to a brand-new project that the user might not have wanted. Acceptable — the alternative is broken Add Waypoint / Create Track, which is worse.

## Migration Plan

No on-disk migration is performed. Legacy projects are normalized in memory on each load. The first time the user saves a normalized project, the saved `.ozp` will include the default layers, which is the desired steady state.
