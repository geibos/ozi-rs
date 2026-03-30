# ADR-0014: u64 Newtypes for Domain IDs Without Central Generator

- Status: accepted
- Date: 2026-03-23

## Context

The domain model requires stable identifiers for `Project`, `Layer`, `Track`,
`TrackSegment`, `TrackPoint`, and `Waypoint` to support cross-references within a
project, command targeting, and stable serialization.

Design questions:
- What type should IDs be?
- Should they be generated centrally or assigned by the caller?
- How should they serialize?

## Decision

Each entity type has its own **opaque newtype over `u64`**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TrackId(u64);

impl TrackId {
    pub const fn new(value: u64) -> Self { Self(value) }
}
```

- `u64` provides ample range for any realistic project size
- Newtypes prevent accidental mixing of e.g. `TrackId` and `LayerId` at compile time
- `#[serde(transparent)]` serializes as a bare integer — `42`, not `{"TrackId": 42}`
- **No central ID generator**: callers pass the value to `new()`. Import adapters
  assign sequential IDs from the imported data; the UI assigns the next available ID.

## Consequences

### Positive

- Type safety: cannot pass a `WaypointId` where a `TrackId` is expected
- Cheap to copy (`Copy`), hash, and compare
- Flat JSON serialization makes project files readable and stable
- No runtime state needed for ID generation

### Negative

- **No uniqueness guarantee**: if two import sources use overlapping ID spaces, the
  caller is responsible for avoiding collisions. The domain does not validate uniqueness.
- **No introspection**: cannot tell from an ID where it came from or when it was created
- If the schema ever needs to encode origin or version in the ID (e.g., for multi-device
  sync), `u64` would need to be replaced with a structured ID type

## Rejected Alternatives

### UUID v4

Rejected because UUIDs are 128-bit, more expensive to hash and compare, and serialize
as strings which makes project JSON more verbose. The single-user, single-device scope
does not require UUID-level global uniqueness.

### Centrally generated IDs (e.g., auto-incrementing counter in AppState)

Rejected at this stage because it adds mutable global state and complicates testing.
Import adapters generate IDs deterministically from source data (e.g., row index),
which is more predictable.

## Follow-Up

If Phase 7 track point editing requires stable IDs that survive simplification or
segment splits, evaluate whether sequential u64 IDs remain sufficient or whether
a stable hash of source coordinates is preferable.
