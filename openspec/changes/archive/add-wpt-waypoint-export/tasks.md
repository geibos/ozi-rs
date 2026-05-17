# Tasks

## 1. Validate delta
- [x] 1.1 Read current `openspec/specs/waypoints/spec.md` and confirm the PLT-waypoint requirement is the one being replaced.
- [ ] 1.2 Run `openspec validate add-wpt-waypoint-export --strict` and fix any issues before implementation begins.
- [ ] 1.3 Confirm with stakeholders that removing PLT-waypoint export (which never shipped) is acceptable.

## 2. Implement WPT writer
- [x] 2.1 Create `src-tauri/src/infrastructure/export/wpt.rs` with a `write_wpt(waypoints, writer)` function.
- [x] 2.2 Emit the OziExplorer v1.1 four-line header (`OziExplorer Waypoint File Version 1.1`, `WGS 84`, `Reserved 2`, `Reserved 3`).
- [x] 2.3 Emit one comma-separated row per waypoint: number, name, lat, lon, date, symbol, status, map display, fg color, bg color, description, pointer direction, garmin display, proximity distance, altitude, font size, font style, symbol size, proximity-related columns.
- [x] 2.4 Default symbol code when waypoint has no symbol assigned; map internal symbol enum to OziExplorer symbol codes.
- [x] 2.5 Encode output as Windows-1251 (matching existing OziExplorer file handling in the codebase) and CRLF line endings.
- [x] 2.6 Use `.` as decimal separator and 6 decimal places for lat/lon.

## 3. Tauri handler + API wrapper
- [x] 3.1 Add `export_wpt_waypoints(layer_id, path)` Tauri command in `src-tauri/src/application/` alongside `export_gpx`.
- [x] 3.2 Wire dialog suggestion: `<bundle>/<layer>.wpt` when bundle active, else `<layer>.wpt`.
- [x] 3.3 Add TypeScript wrapper in `src/lib/api/` mirroring the `export_gpx` wrapper.
- [x] 3.4 Update `src/lib/types.ts` if any new types are needed (likely none).

## 4. Frontend UI
- [x] 4.1 Add "Export waypoints (WPT)" entry to `WaypointsPanel` next to existing GPX export action.
- [x] 4.2 Surface success / failure toast consistent with GPX export UX.
- [x] 4.3 Disable the action when the active waypoint layer is empty.

## 5. Tests (encoding / round-trip)
- [x] 5.1 Unit test: header lines match OziExplorer v1.1 spec exactly.
- [x] 5.2 Unit test: three waypoints (two with symbols, one without) produce three rows with correct symbol codes and a default for the unset one.
- [x] 5.3 Unit test: Windows-1251 encoding for non-ASCII waypoint names.
- [x] 5.4 Round-trip test: write WPT, parse with existing WPT reader (or a fixture-based assertion) and confirm coordinates within 1e-6.
- [x] 5.5 Dialog test: pre-filled path is `<bundle>/<layer>.wpt` with bundle, `<layer>.wpt` without.

## 6. QA
- [ ] 6.1 Manual verification per `docs/agent-verification.md`: export a 3-waypoint layer, open the resulting `.wpt` in OziExplorer (or hex-inspect the file) and confirm header + rows.
- [x] 6.2 Run `just clippy` and `just test`.
- [x] 6.3 Update `docs/commands-reference.md` with the new `export_wpt_waypoints` command.
