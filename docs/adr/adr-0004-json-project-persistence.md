# ADR-0004: JSON Project Persistence (.ozp Files)

- Status: accepted
- Date: 2026-03-29

## Context

The project needed a persistence format for saving and loading the user's project state
(map layer references, track layers, waypoint layers, and command history metadata).

Requirements:
- human-readable and debuggable without special tooling
- round-trip stable for all domain types
- low-friction implementation in Rust
- not coupled to any specific UI or external format

## Decision

Persist projects as **JSON files with the `.ozp` extension** using `serde_json`.

All domain types (`Project`, `Track`, `TrackSegment`, `TrackPoint`, `Waypoint`, layer
types, and their ID newtypes) derive `serde::Serialize` and `serde::Deserialize`.
ID newtypes use `#[serde(transparent)]` to serialize as plain numbers.
Optional fields use `#[serde(default, skip_serializing_if = "Option::is_none")]` to
keep the file compact.

The persistence boundary lives in `src/infrastructure/persistence.rs`. The domain
model has no direct dependency on serde at the trait level; derives are additive.

## Consequences

### Positive

- Easy to inspect and edit project files manually
- `serde_json` is mature and well-tested
- No custom parser to maintain
- Schema can evolve with `#[serde(default)]` for new optional fields

### Negative

- No migration framework; schema changes that remove or rename required fields will
  silently lose data or fail to load without explicit version handling
- Large projects with thousands of track points produce large files; binary formats
  would be more compact

## Rejected Alternatives

### Binary format (e.g. bincode, MessagePack)

Rejected because human readability was valued more than file size at this stage.
Can be revisited if performance becomes an issue.

### SQLite project database

Rejected as over-engineering for the current scope; would complicate the persistence
boundary without a clear benefit.

### Reuse of GPX or PLT as project format

Rejected because those formats are external interchange formats, not a suitable
internal project representation. Mixing concerns would couple the domain model to a
specific third-party schema.

## Follow-Up

- Add a `version` field to the project JSON schema before the format is considered
  stable, to enable explicit migration paths.
