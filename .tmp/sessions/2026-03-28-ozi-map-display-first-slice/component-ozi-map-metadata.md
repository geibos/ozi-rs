# Component: OZI MAP metadata import

## Goal
Parse OziExplorer `.map` files into explicit metadata that can later be registered by the application layer and rendered by the UI if the referenced raster is directly supported.

## Interface
- Add `src/infrastructure/import/ozi_map.rs`
- Define explicit parser output and errors for:
  - map title/name
  - source `.map` path
  - referenced raster path
  - whether raster type is directly supported vs deferred (`ozf2/ozfx3`)
  - calibration metadata needed for first-slice display decisions

## Test Strategy
- successful parse of minimal `.map` fixture with relative raster reference
- missing or malformed header fails explicitly
- `ozf2/ozfx3` raster reference is classified as deferred/unsupported for this slice
- path resolution keeps provenance deterministic

## Tasks
- [x] Define parser output/error types in `src/infrastructure/import/ozi_map.rs`
- [x] Implement minimal `.map` header/raster-reference parser
- [x] Implement raster kind classification and safe path resolution helpers
- [x] Add focused tests for success, malformed input, and deferred `ozf2/ozfx3`

## Verification
- Parser returns deterministic metadata for valid `.map` input.
- Failures are explicit and test-covered.
- No UI or application types leak into the parser.

## Result
- Completed and validated.
- Public parser API is exported through `src/infrastructure/import/mod.rs` for use by the next application-level slice.
