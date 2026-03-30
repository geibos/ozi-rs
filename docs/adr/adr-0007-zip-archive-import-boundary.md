# ADR-0007: ZIP Archive as Staged Import Boundary

- Status: accepted
- Date: 2026-03-28

## Context

LizaAlert map bundles arrive as ZIP archives containing heterogeneous content: SQLite
tile databases, OZF2 raster maps, GPX track files, and metadata. The application
needs to import tracks from these archives without coupling the import logic to any
specific archive structure or file format.

Two risks needed to be managed:
1. Pulling OZF2 raster handling into the same code path as GPX import would entangle
   two very different parsers with different risk profiles
2. Adding new archive-backed formats (KML, PLT, WPT) should not require touching
   the archive-reading layer

## Decision

Implement archive import as **three explicit layers**:

1. `infrastructure::import::archive` — reads ZIP containers, exposes entry metadata
   and byte access. Has no knowledge of what the entries contain.

2. `infrastructure::import` format adapters (e.g. `gpx.rs`, `plt.rs`) — classify
   archive entries by supported type and parse them into domain types. Each adapter
   is independent; adding a new format adds a new file.

3. `application` — orchestrates the workflow: calls archive inventory, calls relevant
   adapters, registers results into the project via commands.

The UI triggers the workflow and renders status; it does not inspect archive contents.

## Consequences

### Positive

- Adding a new import format (KML, WPT) requires only a new adapter file
- OZF2 raster risk is isolated; GPX import does not depend on it
- Each adapter is independently testable with fixture ZIP files
- The archive layer is reusable for any future bundle scanning feature

### Negative

- Three layers instead of one adds indirection for a simple "open GPX" action
- Callers must understand the staged workflow

## Rejected Alternatives

### Single monolithic archive importer

Rejected because it would couple all format parsers together and make incremental
format addition harder to review.

### Handle archives only in the UI layer

Rejected because it would put import logic where it cannot be unit-tested cleanly.
