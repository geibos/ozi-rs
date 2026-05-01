# MVP Audit Triage

Date: 2026-05-01
Source: docs/superpowers/plans/2026-04-28-mvp-audit.md
Authoritative scope: ADR-0020

| Priority | Feature | Status | Evidence | Hypothesis |
|---------:|---------|--------|----------|------------|
| P0 | **MCP `appium_click` infra (F8)** | **works** | [findings doc](2026-04-29-tooling-audit-findings.md) F8; commit 191fd39 | F8 fix confirmed: MCP now uses standard WebDriver `find_element` + element click flow. Maps… button click succeeds. |
| P2 | Bundle open — local folder | partial (verified-by-AX-tree) | [smoke-bundle-open.md](smoke-bundle-open.md) | Persistence + project enumeration confirmed via AX tree (active map = `…Satell_z17_ozf.map`, "Loaded 12761 projects"). Click flow not driven due to F8. |
| P3 | Bundle open — URL (LizaAlert) | hidden — fixture missing | [smoke-bundle-open.md](smoke-bundle-open.md) §URL | URL fixture not provided in this run. |
| P2 | Map switch (Topo ↔ Satellite, MBTiles + OZF2) | partial (verified-by-AX-tree) | [smoke-map-switch.md](smoke-map-switch.md) | At least one tile pipeline (OZF2 raster) renders; switching not driven (F8). User-reported "Maps unresponsive" not differentiated yet. |
| —  | Maps window / Maps… button | covered by prior smoke | [smoke-maps-window.md](smoke-maps-window.md) | Tauri `plugin:window` capability fix landed (commit `53cf15f`); F8 click fix (commit 191fd39) confirms button is now clickable. |
| P2 | **Task 4a: MBTiles Topo tile loading** | partial (F8 fixed; bundle list UI unclear) | [smoke-mbtiles-tiles.md](smoke-mbtiles-tiles.md) | Maps… button click works. Map Bundles window opens. Bundle list structure in AX tree not immediately clear; may require scrolling or custom component inspection. |
| P2 | **Task 4b: OZF2 satellite raster loading** | partial (OZF2 pipeline verified; switching not yet attempted) | [smoke-ozf2-tiles.md](smoke-ozf2-tiles.md) | OZF2 tile pipeline confirmed working (active map = Satellite OZF2). Maps… button click works. Bundle list navigation same blocker as Task 4a. |
| P3 | **Task 4c: OSM online fallback** | hidden — no unload UI in MVP | [smoke-osm-fallback.md](smoke-osm-fallback.md) | OSM attribution visible on map (MapLibre + OpenStreetMap). No UI to deactivate/unload current bundle. Feature cannot be triggered via Appium; code review needed to confirm implementation. |
| P2 | **Task 5a: GPX track import** | pending | [smoke-track-import-gpx.md](smoke-track-import-gpx.md) | GPX import endpoint and handler ready. Fixture 79-point track available. Awaiting Appium run to verify file picker + import dialog flow and polyline rendering. |
| P2 | **Task 5b: PLT track import** | pending | [smoke-track-import-plt.md](smoke-track-import-plt.md) | PLT fixtures available in _unpacked directories (79-point tracks). Same flow as GPX. Awaiting Appium run to confirm OziExplorer format compatibility. |
| P3 | **Task 5c: ZIP archive import** | hidden — fixture TBD | [smoke-track-import-zip.md](smoke-track-import-zip.md) | ZIP archive support not confirmed in requirements; may require backend implementation. Test ZIP fixture will be created from existing GPX + PLT before running. If ZIP support missing, classify as `hidden`. |
| P2 | **Task 5d: Multi-track display + visibility toggle** | pending | [smoke-track-display.md](smoke-track-display.md) | Multiple tracks can be imported (5a/5b). Visibility toggle (eye icon) expected in Tracks panel. Awaiting Appium run to verify toggle state and polyline visibility on map. |
| P3 | **Task 5e: Track color and line-width styling** | pending | [smoke-track-style.md](smoke-track-style.md) | Style controls (color picker, width slider) expected in Tracks panel or context menu. Awaiting Appium run to verify color picker accessibility, color change application, and persistence. |
| P3 | **Task 5f: Large-track load performance (>10k points)** | hidden — large fixture not available | [smoke-track-large.md](smoke-track-large.md) | No >10k-point fixture provided in example_data. Current fixtures <100 points. Test cannot run until large GPX (e.g., multi-day SAR operation or synthetic data) is added to fixtures. Performance target: <2 s import + responsive pan/zoom. |

## Notes

- **F8 blocker.** Until F8 is fixed, every audit task that requires a
  click is bottlenecked. Recommended path:
  1. Fix `tools/ozi-rs-mcp/src/appium.rs:437-468` (replace the Mac2 endpoint
     with the standard WebDriver `find_element` + element click flow). Same
     fix for `appium_type_text` (`appium.rs:453`).
  2. Add a regression test that asserts the body shape sent for click /
     type_text matches the `/element/{eid}/click` and
     `/element/{eid}/value` endpoints (not the `/appium/mac2/*` paths).
  3. Re-run Audit Task 3 to upgrade `partial → works/broken`.

- **AX tree-only audit (Tier 1.5)** can keep the audit progressing for
  features whose presence/absence shows up in the tree without needing a
  click — e.g. Tasks 4 (mbtiles/ozf2/osm presence), 7 (waypoints panel
  reachability), 8 (on-map tools toolbar presence), 9 (theme picker,
  devtools toggle). Click-required smokes (track import, point delete,
  bundle URL open, segment break) stay blocked.

- **Priority rule:**
  - P0 = `broken` on critical SAR workflow OR infrastructure blocker
  - P1 = `missing` on critical SAR workflow
  - P2 = `partial` on any feature
  - P3 = `hidden` / fixture-missing

- **smoke-maps-window.md cross-reference:** prior smoke documents the
  same Maps button entry point. Its hypothesis (missing
  `plugin:window:*` permissions) was resolved by commit `53cf15f`. The
  current Task 3 run could not confirm the Maps→Bundle Loader window
  flow because of F8.
