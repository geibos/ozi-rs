# MVP Audit Triage

Date: 2026-05-01
Source: docs/superpowers/plans/2026-04-28-mvp-audit.md
Authoritative scope: ADR-0020

| Priority | Feature | Status | Evidence | Hypothesis |
|---------:|---------|--------|----------|------------|
| P0 | **MCP `appium_click` infra (F8)** | broken | [findings doc](2026-04-29-tooling-audit-findings.md) F8 | MCP wrapper posts `{selector}` to `/appium/mac2/click` which only accepts `{x,y}` or `{elementId}`; needs a switch to `POST /element` + `POST /element/{eid}/click`. Fix unblocks every Tier-2 smoke. |
| P2 | Bundle open — local folder | partial (verified-by-AX-tree) | [smoke-bundle-open.md](smoke-bundle-open.md) | Persistence + project enumeration confirmed via AX tree (active map = `…Satell_z17_ozf.map`, "Loaded 12761 projects"). Click flow not driven due to F8. |
| P3 | Bundle open — URL (LizaAlert) | hidden — fixture missing | [smoke-bundle-open.md](smoke-bundle-open.md) §URL | URL fixture not provided in this run. |
| P2 | Map switch (Topo ↔ Satellite, MBTiles + OZF2) | partial (verified-by-AX-tree) | [smoke-map-switch.md](smoke-map-switch.md) | At least one tile pipeline (OZF2 raster) renders; switching not driven (F8). User-reported "Maps unresponsive" not differentiated yet. |
| —  | Maps window / Maps… button | covered by prior smoke | [smoke-maps-window.md](smoke-maps-window.md) | Tauri `plugin:window` capability fix landed (commit `53cf15f`); resolution still pending Tier-2 click confirmation post-F8. |

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
