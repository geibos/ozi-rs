# ADR-0006: OZF2 Decoding via Sibling Crate (ozf2-rs)

- Status: accepted
- Date: 2026-03-29

## Context

LizaAlert map bundles include OziExplorer raster maps in the OZF2 format (`.ozf2`
files paired with `.map` georeference files). Rendering these maps is a primary use
case.

OZF2 is a **proprietary, reverse-engineered** format. Key facts established during
feasibility research (see `.tmp/external-context/ozf2/`):

- No published Rust crate for OZF2 existed on crates.io at evaluation time
- The format is not officially documented by OziExplorer
- GDAL removed its OZI driver in GDAL 3.11, citing the format as not strategically
  worth maintaining
- OziExplorer has since moved to OZF4, which has not been reverse-engineered
- Public implementations exist based on community reverse-engineering of OZF2/OZFX3

The decoder is also potentially sensitive: distributing a reverse-engineered proprietary
format decoder in the main application crate conflates that risk with the rest of the
codebase.

## Decision

Implement OZF2 decoding in a **separate sibling Rust crate (`ozf2-rs`)** located at
`../ozf2-rs` relative to this repository. Reference it as a path dependency:

```toml
ozf2-rs = { path = "../ozf2-rs" }
```

The `ozi-rs` codebase accesses OZF2 only through a narrow adapter in
`src/infrastructure/import/ozi_raster.rs`. Domain and application layers have no
knowledge of the OZF2 format.

## Consequences

### Positive

- Legal/licensing risk of reverse-engineered decoder is isolated to one crate
- The decoder can be developed, versioned, and replaced independently
- If a proper open-source OZF2 crate appears on crates.io, swapping it requires
  only changes to the infrastructure adapter
- Domain and application code never sees OZF2 decoding details

### Negative

- Requires both repositories to be present on the developer's machine
- No crates.io release path without resolving the licensing question
- If OZF4 becomes necessary, a new decoder crate would be needed

## Rejected Alternatives

### Inline OZF2 decoder in ozi-rs

Rejected because it mixes legal risk with the main codebase and makes the boundary
harder to replace later.

### Skip OZF2 support entirely

Not viable: OZF2 is the primary raster format used in LizaAlert bundles.
