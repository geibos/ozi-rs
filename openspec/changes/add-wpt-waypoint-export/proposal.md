# Add OziExplorer WPT waypoint export

## Why

SAR (search-and-rescue) volunteers run OziExplorer on field GPS devices and laptops, and rely on `.wpt` files for transferring waypoints between systems. Many older Garmin and field GPS units do not parse GPX reliably, so a GPX-only export path leaves a real interoperability gap for our primary user community.

The current `waypoints` spec lists a "System exports waypoints to PLT" requirement. This is incorrect: in OziExplorer the `.plt` extension is reserved for tracks (point lists), and waypoints use the `.wpt` text format ("OziExplorer Waypoint File Version 1.1"). This change corrects the spec by removing the PLT-waypoint requirement and replacing it with a properly-scoped WPT-waypoint export requirement, and implements the writer.

Today only GPX waypoint export exists (`src-tauri/src/infrastructure/export/gpx.rs`); there is no `.wpt` writer at all.

## What Changes

- Remove the misnamed "System exports waypoints to PLT" requirement from the `waypoints` capability (PLT is a track format in OziExplorer, never a waypoint format).
- Add a "System exports waypoints to OziExplorer WPT" requirement covering the v1.1 format with optional symbol, elevation, and timestamp.
- Add a new Tauri command `export_wpt_waypoints` backed by a new module `src-tauri/src/infrastructure/export/wpt.rs` that writes the OziExplorer v1.1 header and one row per waypoint (Latitude/Longitude in WGS84 decimal degrees, name, symbol code, optional elevation/timestamp).
- Add a frontend wrapper next to `export_gpx` and surface an "Export waypoints (WPT)" action in `WaypointsPanel`.
- Path suggestion: when a bundle is active, the file picker pre-fills `<bundle>/<layer>.wpt`; otherwise it suggests `<layer>.wpt` only.

## Impact

- Affected capability: `waypoints`.
- New files: `src-tauri/src/infrastructure/export/wpt.rs`, accompanying unit + round-trip tests, frontend wrapper, UI hook in `WaypointsPanel`.
- No schema or persistence changes; export is a pure read of the existing waypoint model.
- No behavioural change for GPX export.
