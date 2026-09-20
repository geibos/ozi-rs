## ADDED Requirements

### Requirement: Backend is layered into domain, application, infrastructure, commands

The Rust backend (`src-tauri/src`) SHALL be organised into four modules with these dependency rules:

- `domain` SHALL depend on no other module of the crate and on no I/O, HTTP, database, archive or Tauri crate. It holds the entities (`Project`, `MapLayer`, `TrackLayer`, `WaypointLayer`, `Track`, `TrackSegment`, `TrackPoint`, `Waypoint`), their identifiers, invariants and pure mutation methods.
- `application` and `infrastructure` MAY depend on `domain`. `application` MAY call `infrastructure` adapters (import, export, persistence, LizaAlert client). `infrastructure` MAY use plain data types exported by `application` (`LizaProjectSummary`, `ActiveMapSelection`, `MapCenter`, …) but SHALL NOT reference `AppState`, `CommandStack` or `ProjectCommand`.
- `commands` (Tauri IPC handlers) and `lib.rs` SHALL be the only modules that reference the `tauri` crate.

#### Scenario: Domain imports stay inside domain

- **WHEN** `grep -rhoE 'use crate::[a-z_]+' src-tauri/src/domain | sort -u` is run
- **THEN** the only line printed is `use crate::domain`

#### Scenario: Tauri types do not leak below the commands layer

- **WHEN** `grep -rl tauri src-tauri/src/domain src-tauri/src/application src-tauri/src/infrastructure` is run
- **THEN** it prints nothing

#### Scenario: Infrastructure does not drive application state

- **WHEN** `grep -rn 'AppState\|CommandStack\|ProjectCommand' src-tauri/src/infrastructure` is run
- **THEN** it prints nothing

### Requirement: Tauri command handlers are thin adapters over `AppState`

Every `#[tauri::command]` handler in `src-tauri/src/commands/` SHALL obtain the shared `Arc<Mutex<AppState>>` through `lock_app_state`, delegate to `AppState` methods, emit `state-changed` when the project or session changed, and convert errors to `String`. Handlers SHALL NOT mutate domain entities directly and SHALL NOT hold business rules; project edits reach the domain only through `AppState` (undoable edits as `ProjectCommand`, per the `undo-redo` capability).

#### Scenario: No direct entity mutation in handlers

- **WHEN** `grep -n '_mut()' src-tauri/src/commands/mod.rs` is run
- **THEN** it prints nothing

#### Scenario: Handler failure surfaces as a string error

- **WHEN** the `AppState` method behind a handler returns an error
- **THEN** the IPC call resolves to `Err(String)` carrying the error text and no panic crosses the IPC boundary

### Requirement: Domain identifiers are opaque u64 newtypes assigned by callers

`ProjectId`, `LayerId`, `TrackId`, `TrackSegmentId`, `TrackPointId` and `WaypointId` SHALL each be a distinct `Copy + Eq + Hash` newtype over `u64` annotated `#[serde(transparent)]`, constructed with `const fn new(u64)` and read with `value()`. The domain SHALL NOT provide a central ID generator: the caller (import adapter or `AppState`) assigns the value, using `max + 1` over the IDs already present in the parent collection for new entities. Uniqueness is guaranteed only within that parent collection.

#### Scenario: IDs serialize as bare integers

- **WHEN** `TrackId::new(42)` is serialised with `serde_json`
- **THEN** the output is `42`, not an object or string wrapper

#### Scenario: Mixing ID kinds is a compile error

- **WHEN** a `WaypointId` is passed where a `TrackId` parameter is expected
- **THEN** `cargo check` fails with a type mismatch

#### Scenario: New entity takes the next free ID in its collection

- **WHEN** `AppState` creates a track in a layer whose highest existing track ID is 7
- **THEN** the new track receives `TrackId(8)`

### Requirement: `TrackStyle` is part of the `Track` domain entity

`Track` SHALL own a `TrackStyle { color: [u8; 4] /* RGBA */, line_width: f32, visible: bool, opacity: f32 }` that is persisted in the project file and carried through GPX export. `TrackStyle::default()` SHALL be opaque red (`[255, 0, 0, 255]`) and visible. Only presentation attributes that must survive save/load or export round-trips belong in domain entities; other UI state SHALL stay outside the domain.

#### Scenario: Style survives a project round-trip

- **WHEN** a project whose track has color `[0, 128, 255, 255]` and `line_width` 4.0 is saved to `.ozp` and loaded again
- **THEN** the loaded track reports the same color and line width

#### Scenario: Default style is visible red

- **WHEN** `TrackStyle::default()` is evaluated
- **THEN** `visible` is `true` and `color` is `[255, 0, 0, 255]`

### Requirement: Rust crates use the 2024 edition

Every Rust crate in the workspace (`src-tauri`, `tools/ozi-rs-mcp`) SHALL declare `edition = "2024"` in its `Cargo.toml`. The toolchain version itself is pinned by `rust-toolchain.toml` as specified in the `ci-pipeline` capability.

#### Scenario: Edition is declared in every workspace crate

- **WHEN** `grep -h '^edition' src-tauri/Cargo.toml tools/ozi-rs-mcp/Cargo.toml` is run
- **THEN** every line printed is `edition = "2024"`

### Requirement: Backend logging goes through `tracing` filtered by `RUST_LOG`

The backend SHALL emit diagnostics only through the `tracing` macros; `println!` and `eprintln!` SHALL NOT appear in `src-tauri/src`. `run()` in `lib.rs` SHALL install a `tracing_subscriber::fmt` subscriber whose `EnvFilter` is read from `RUST_LOG` and defaults to `info`. `AppState::push_diagnostic` SHALL be the single path that both emits a `tracing` event at the matching level (`Info` → `info!`, `Warning` → `warn!`, `Error` → `error!`) and appends the entry to the in-app diagnostics ring buffer of 200 entries exposed through `AppStateDto.diagnostics`.

#### Scenario: No ad-hoc printing in the backend

- **WHEN** `grep -rn 'println!\|eprintln!' src-tauri/src` is run
- **THEN** it prints nothing

#### Scenario: Log level follows the environment

- **WHEN** the app is started with `RUST_LOG=ozi_rs_lib=debug`
- **THEN** debug-level events from the backend crate are printed to the terminal, while an unset `RUST_LOG` prints only `info` and above

#### Scenario: Ring buffer is bounded

- **WHEN** 201 diagnostics have been pushed in one session
- **THEN** `AppStateDto.diagnostics` contains the most recent 200 and the oldest entry is dropped

### Requirement: Long-running work runs off the IPC thread and reports via events

Every `#[tauri::command]` handler SHALL be a synchronous function that returns promptly. Long-running work (catalog fetch, bundle preview, bundle and map downloads, local bundle open) SHALL be moved to a background task — `std::thread::spawn` for blocking `reqwest`/file work, `tauri::async_runtime::spawn` (tokio) for the concurrent multi-file bundle download built on `JoinSet` + `Semaphore` and the async `reqwest::Client` — and SHALL report back by re-locking `AppState` through `lock_app_state` and emitting Tauri events (`projects-chunk`, `download-progress`, `bundle-progress`, `bundle-file-ready`, `state-changed`). The `std::sync::Mutex<AppState>` SHALL NOT be held across an `.await` or across blocking I/O.

#### Scenario: Commands are synchronous

- **WHEN** `grep -rn -A2 '#\[tauri::command' src-tauri/src | grep -c 'async fn'` is run
- **THEN** the count is `0`

#### Scenario: Starting a bundle download does not block the caller

- **WHEN** the frontend invokes `load_project` for a bundle that takes minutes to download
- **THEN** the call resolves with a `download_id` before any file finishes, and progress arrives through `bundle-progress` / `bundle-file-ready` events followed by a final `state-changed`

#### Scenario: State lock is released between progress updates

- **WHEN** the download forwarder task applies a progress notification
- **THEN** it locks `AppState`, applies the update, releases the lock and only then emits the event; the lock is never held while awaiting the next notification

### Requirement: OZF2 rasters are decoded only through the `ozf2` crate adapter

OZF2 decoding SHALL be provided by the external `ozf2` crate (crates.io, `ozf2 = "0.1"` in `src-tauri/Cargo.toml`) and SHALL be reached only through the adapter `src-tauri/src/infrastructure/import/ozi_raster.rs`, which wraps it in `OziRasterTileSource` / `DecodedOziRasterTile`. No other module SHALL import `ozf2::` types. A `.map` file whose raster is not OZF2 SHALL fail with `OziRasterDecodeError::UnsupportedRasterKind` rather than being partially decoded.

#### Scenario: The adapter is the only user of the crate

- **WHEN** `grep -rln 'ozf2::' src-tauri/src` is run
- **THEN** the only file printed is `src-tauri/src/infrastructure/import/ozi_raster.rs`

#### Scenario: Unsupported raster kind is rejected

- **WHEN** `open_ozi_raster_tile_source` is called with metadata whose `raster_kind()` is not `Ozf2`
- **THEN** it returns `Err(OziRasterDecodeError::UnsupportedRasterKind(_))`

### Requirement: Typed IPC commands are registered once and generate TypeScript bindings

All typed `#[tauri::command]` handlers SHALL be listed once in `specta_builder()` (`collect_commands![]` in `src-tauri/src/lib.rs`), which feeds both the runtime invoke handler and the generated `src/lib/bindings.ts`. The test `typescript_bindings_are_up_to_date` SHALL regenerate the bindings and fail while the committed file is stale. Commands returning raw bytes (`tauri::ipc::Response`) are the only exception and are registered on the plain `generate_handler!` (see the `tile-rendering` capability).

#### Scenario: Stale bindings fail the test suite

- **WHEN** a typed command's signature changes and `cargo test --manifest-path src-tauri/Cargo.toml typescript_bindings` is run
- **THEN** the test rewrites `src/lib/bindings.ts` and fails once, passing on the next run with the regenerated file committed

#### Scenario: Unregistered command is unreachable

- **WHEN** a typed command is defined but omitted from `collect_commands![]`
- **THEN** invoking it from the frontend fails and it does not appear in `bindings.ts`
