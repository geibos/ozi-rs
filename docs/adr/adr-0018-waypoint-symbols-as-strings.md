# ADR-0018: Waypoint Symbols as Optional Strings

- Status: accepted
- Date: 2026-04-06

## Context

Waypoints need visual differentiation on the map. The domain model needs a way to
represent waypoint type or icon. Two approaches were considered:

1. **Rust enum** — `WaypointSymbol` with variants like `Flag`, `Camp`, `Danger`, etc.
2. **Optional string** — `symbol: Option<String>` with a fixed display set in the UI.

## Decision

Use `symbol: Option<String>` on the `Waypoint` entity. The UI defines a fixed set of
known symbols with emoji rendering:

| Key | Display |
|-----|---------|
| `flag` | 🚩 |
| `camp` | ⛺ |
| `danger` | ⚠️ |
| `water` | 💧 |
| `shelter` | 🏠 |
| `meeting-point` | 📍 |
| `start` | 🟢 |
| `finish` | 🏁 |
| `viewpoint` | 👁️ |
| `parking` | 🅿️ |

`None` means no symbol (default marker).

## Rationale

- **Extensibility**: new symbols can be added without changing the Rust domain model
  or migrating saved project files
- **Serialization stability**: JSON persistence stores plain strings; no enum
  deserialization breakage when symbols change
- **UI ownership**: the symbol-to-emoji mapping is a rendering concern, not a domain
  concern; keeping it in the frontend is correct per the architecture
- **Simplicity**: `Option<String>` is the simplest possible representation

## Consequences

### Positive

- Adding a new symbol requires only a frontend change (SymbolPicker component)
- Saved projects with unknown symbols degrade gracefully (show default marker)
- No Rust recompilation needed for symbol set changes

### Negative

- No compile-time exhaustiveness check for symbol handling
- Typos in symbol strings are not caught until runtime
- The "known symbols" set is implicit (defined in UI), not explicit in the domain
