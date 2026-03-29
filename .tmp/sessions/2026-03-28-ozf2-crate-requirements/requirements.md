# Requirements: Experimental OZF2/OZFX3 Rust Crate

## Goal

Build a separate experimental Rust crate in its own subdirectory that can read legacy OziExplorer raster formats `ozf2` and, if feasible, `ozfx3`.

The crate is intended to support `ozi-rs` later through an optional infrastructure adapter, but it must be designed as a standalone library first.

## Why this crate should be separate

- `ozf2/ozfx3` are proprietary formats with community reverse-engineered decoding rather than a clean official open spec.
- There is no credible maintained Rust crate in the current ecosystem.
- The work is high-risk and should not block the main archive/OZI import roadmap.
- The main app already treats `.ozf2/.ozfx3` as deferred payloads while `.map` metadata support proceeds independently.

## Product Positioning

- Status: experimental
- Support level: read-only, best-effort
- Integration: optional
- Primary target: older `ozf2` files first
- Secondary target: `ozfx3` only if the first decoder architecture can support it cleanly
- Explicitly out of scope: `OZF4`

## Intended consumers

1. A future `ozi-rs` infrastructure adapter that needs decoded raster tiles or full raster images.
2. Command-line/debug tooling for validating decoder behavior against fixture files.
3. Tests that compare decoded output against known fixtures and metadata.

## Functional Requirements

### 1. File opening
- The crate must open `ozf2` files from a filesystem path and from any `Read + Seek` source where practical.
- The crate should expose explicit, typed errors for invalid headers, unsupported variants, truncated data, decompression failures, and integrity issues.

### 2. Metadata inspection
- The crate must expose basic raster metadata without requiring a full decode:
  - format variant (`ozf2` vs `ozfx3` when detectable)
  - image width and height
  - zoom/overview count
  - tile size if encoded in the format
  - palette information for paletted rasters if applicable

### 3. Raster access
- The crate must support deterministic access to decoded raster content for at least the base image level.
- The preferred API is tile/block oriented first, with full-image assembly built on top.
- The crate should expose:
  - image level enumeration
  - block/tile enumeration or random access
  - block decode to raw indexed or RGBA pixels

### 4. Full-image reconstruction
- The crate should provide a helper to reconstruct a full raster image for a chosen level from decoded blocks.
- The output format should be a simple in-memory representation suitable for conversion into `image` crate buffers or UI textures later.

### 5. `ozfx3` support
- The crate must treat `ozfx3` as a separate compatibility tier.
- If `ozfx3` support is implemented, it must be clearly documented as best-effort and backed by dedicated fixtures.
- If `ozfx3` cannot be supported safely in the first implementation, the crate must fail explicitly with a typed `UnsupportedVariant`-style error rather than silently misdecode.

## Non-Goals

- No write support.
- No encoder.
- No `OZF4` support.
- No UI code.
- No map calibration parsing (`.map` belongs elsewhere).
- No direct coupling to `ozi-rs` domain/application types.
- No hidden conversion through external binaries in the core library API.

## Architecture Requirements

- The crate must be standalone and reusable outside `ozi-rs`.
- Keep parsing/decoding logic in small, testable modules.
- Suggested internal module split:
  - `header`
  - `index` / `levels`
  - `palette`
  - `tile`
  - `decode`
  - `error`
  - `image` or `raster`
- Prefer explicit data structures over opaque stateful decoder objects where possible.
- Avoid leaking UI/runtime concerns into the crate.

## Safety and Legal Requirements

- The implementation must be clean-room in spirit: use public references as research input, not proprietary source code.
- The crate must document that the formats are proprietary and reverse-engineered.
- The crate must use conservative wording: best-effort support for older variants only.
- The crate must document known uncertainty around `ozfx3` and lack of viable `OZF4` support.

## Fixture Requirements

- The crate must be developed against real fixture files kept outside the library logic.
- Initial fixture set should include at least:
  - one known-good `ozf2`
  - one real `.map` file that references that `ozf2`
  - if available, one `ozfx3`
  - one intentionally truncated/corrupt fixture
- The current `example_data/2021-07-30_Murino/.../2021-07-30_Murino_Topo_EEKO_z16.ozf2` dataset should be treated as an initial research fixture candidate.

## Testing Requirements

- Unit tests for header parsing, level tables, palette decoding, and tile decode.
- Regression tests for every real-world decoding bug.
- Deterministic fixture-based integration tests.
- Tests for unsupported or malformed inputs must verify explicit failures.
- Prefer validating dimensions, tile counts, and representative pixel output rather than brittle whole-file snapshots unless snapshots are readable and reviewable.

## Suggested API Shape

The exact API can change, but it should roughly support:

```rust
let dataset = OziRaster::open(path)?;
let info = dataset.info();
let levels = dataset.levels();
let tile = dataset.decode_tile(level_index, tile_x, tile_y)?;
let image = dataset.decode_full_image(level_index)?;
```

And explicit errors such as:

- `UnsupportedVariant`
- `InvalidHeader`
- `CorruptIndex`
- `CorruptTile`
- `Decompression`
- `UnsupportedFeature`

## Milestones

### Milestone 0: Feasibility spike
- Confirm one real `ozf2` fixture can be parsed enough to extract metadata.
- Identify whether level tables, tile offsets, and compression assumptions match public references.
- Produce a short note on whether `ozfx3` looks achievable in the same architecture.

### Milestone 1: Metadata reader
- Parse file header and expose basic dataset info.
- No full image decode required yet.

### Milestone 2: Base-level tile decode
- Decode at least one tile/block from a real `ozf2` fixture.
- Validate dimensions, palette use, and decompression.

### Milestone 3: Full base image decode
- Reconstruct a complete image for one real `ozf2` file.

### Milestone 4: Optional `ozfx3`
- Either add best-effort `ozfx3` support with fixtures, or explicitly document deferral.

### Milestone 5: `ozi-rs` adapter planning
- Only after the crate is stable enough, define the adapter boundary for `ozi-rs` infrastructure.

## Acceptance Criteria for the crate spike

- A separate Rust crate exists in its own subdirectory.
- It can parse metadata from at least one real `ozf2` file.
- It can decode at least one real tile or a complete base image from that fixture.
- It fails explicitly on malformed input.
- It documents that support is experimental, read-only, and limited to older reverse-engineered variants.
- It makes no promise of `OZF4` support.

## Recommended handoff note for the next session

Build a separate experimental Rust crate for `ozf2` first, with `ozfx3` only if the architecture and fixtures support it cleanly. Keep it standalone, read-only, fixture-driven, and conservative about legal/format claims. Treat this as an infrastructure decoder experiment, not as a required dependency for the main `ozi-rs` application.
