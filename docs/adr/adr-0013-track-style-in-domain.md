# ADR-0013: TrackStyle Stored in Domain Entity

- Status: accepted (tension with ADR-0001 noted)
- Date: 2026-03-29

## Context

Each track needs a visual style: color, line width, visibility flag, and opacity.
This style must survive project save/load and round-trip through GPX export/import
(via the Garmin color extension).

Two options:
1. Store style in the `Track` domain entity — persisted in `.ozp` and exported to GPX
2. Store style as UI-layer state — transient, not persisted in the domain model

## Decision

Store `TrackStyle` **inside the `Track` domain entity**:

```rust
pub struct TrackStyle {
    pub color: [u8; 4],   // RGBA
    pub line_width: f32,
    pub visible: bool,
    pub opacity: f32,
}
```

`Track` holds a `style: TrackStyle` field, persisted in JSON and exported to GPX
via the Garmin color extension.

## Rationale

The style is **user-authored data**, not transient rendering state:

- A LizaAlert volunteer explicitly sets track color to distinguish team routes
  (e.g., red = Team A, blue = Team B). This is meaningful domain information.
- The LizaAlert OK standard implies specific visual conventions for track review.
- GPX export must carry the color so the recipient sees the same visual in their
  OziExplorer or other tool. This requires the color to be in the data model.
- If style were UI-only, it would be lost on project reload or export/import round-trip.

## Tension with ADR-0001

ADR-0001 states that domain entities should be UI-agnostic. `TrackStyle` contains
rendering details (RGBA bytes, pixel line width) that are arguably presentation
concerns.

This tension is accepted because:
- The color is semantically meaningful, not purely decorative
- Export formats (GPX Garmin extension) require it at the data boundary
- The alternative (re-derive style from metadata) has no practical benefit here

Future editors of this code should not use this as a precedent for adding arbitrary
UI state to domain entities. Only style properties that must survive persistence
and export round-trips belong in the domain.

## Consequences

### Positive

- Track color survives project save/load and GPX export/import
- No separate UI state synchronization needed for style
- Commands that change style (`SetTrackColor`, `SetTrackVisible`) get undo/redo for
  free via the snapshot-based command stack (ADR-0005)

### Negative

- Domain entity carries rendering knowledge (RGBA, line_width in pixels)
- If the UI ever adopts a theming system that derives track colors dynamically, the
  stored color and the rendered color could diverge
- `TrackStyle::default()` bakes in red as the default color, which may conflict with
  future OK-standard color conventions
