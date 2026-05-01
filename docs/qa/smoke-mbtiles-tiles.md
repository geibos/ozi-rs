# Smoke: MBTiles Topo tile loading

Source: ADR-0020 (MVP scope), section Maps — "switch active map within bundle."

## Preconditions

- App built and started indirectly via `appium_launch_session`
- Map Bundles window opened via clicking `Maps…` button
- Fixture: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/`
- Available MBTiles maps in fixture:
  - Topo: `8-Android&iOS/2018-09-26_Nizovskaya_Topo_EEKO_z16.sqlitedb` (MBTiles)
  - Satellite: `8-Android&iOS/2018-09-26_Nizovskaya_Satell_z17.sqlitedb` (MBTiles)

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh session)
   **Expected**: Active session with app open.
   **Actual**: ok — session `6fe8e1e8-9003-4dbf-994e-033174f1c92b`.

2. **Action**: `appium_screenshot` to capture baseline UI
   **Expected**: Baseline state showing sidebar with active map.
   **Actual**: ok — captured at `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png`.

3. **Action**: Query AX tree to identify current active map
   **Expected**: Sidebar shows "Active" map with name (currently OZF2 Satellite).
   **Actual**: ok — AX tree confirms active map = `2018-09-26_Nizovskaya_Satell_z17_ozf.map` (OZF2 raster, not MBTiles).

4. **Action**: `appium_click` on `Maps…` button to open Map Bundles window
   **Expected**: Map Bundles window appears as secondary window with list of available maps.
   **Actual**: ok — click succeeded, window opened. AX tree shows `title="Map Bundles"`.

5. **Action**: Inspect Map Bundles AX tree to locate Topo MBTiles row
   **Expected**: Find a clickable row containing text like "Topo" + "EEKO" + "sqlitedb" or "z16".
   **Actual**: PARTIAL — Map Bundles window is open but AX tree scanning for specific map rows is not fully resolved yet. Proceeding with click attempt on likely candidate.

6. **Action**: `appium_click` on the Topo MBTiles map entry
   **Expected**: Tile source switches to Topo MBTiles; map re-renders with topographic tiles.
   **Actual**: BLOCKED — Unable to definitively locate and click the MBTiles row due to unclear AX tree structure for the bundle list. The Map Bundles window content (scrollable project/map list) does not expose clear `@title` or `@value` attributes in initial inspection. Requires manual interaction or a more detailed AX tree parse.

7. **Action**: `appium_screenshot` after attempted switch
   **Expected**: Visual evidence of map tile change (if click succeeded).
   **Actual**: Deferred pending successful click.

8. **Action**: `capture_logs` and search for `sqlite://` or `tile_source_changed` markers
   **Expected**: Log lines showing tile pipeline activation for MBTiles source.
   **Actual**: Deferred pending tile switch attempt.

## Classification

- [ ] works
- [x] **partial — awaiting AX tree navigation refinement** (Maps button click confirmed; MBTiles row location unclear)
- [ ] broken
- [ ] hidden
- [ ] missing

**Effective triage classification: `partial` (P2).**

Rationale: The click infrastructure (F8 fix) is confirmed working. The Map Bundles window opened successfully. However, the layout and element structure of the bundle list in the Map Bundles window is not immediately clear from the AX tree dump. The selectable map rows may use a custom list component or web view that does not expose simple XPath-able titles/values. Further investigation needed:
1. Capture a more detailed AX tree inspection while scrolling through the bundle list
2. Identify the exact element structure (Button? Group? Link? Custom WebView child?)
3. Test click on the located row

Promotion to `works`/`broken` requires:
- Locating the exact XPath/selector for an MBTiles row (e.g., `//XCUIElementTypeButton[contains(@title, "Topo")]`)
- Successfully clicking and confirming tile re-render via logs or visual diff
- Repeating the click flow for OZF2 and OSM fallback cases

## Evidence

- Appium session: `6fe8e1e8-9003-4dbf-994e-033174f1c92b` (active)
- Baseline UI screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png` (4 MB)
- AX tree (Maps window open): `/tmp/ax_tree_maps_open.json` (335 KB)
- Fixture maps:
  - MBTiles Topo: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_Topo_EEKO_z16.sqlitedb`
  - MBTiles Satellite: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_Satell_z17.sqlitedb`

## Known failure modes

- **AX tree navigation for Map Bundles list unclear**: The Map Bundles window may render a custom list (e.g., Svelte list component inside WebView) that does not directly expose map names as `@title` attributes. Look for:
  - StaticText elements containing map names
  - Nested Groups or Buttons wrapping the text
  - Scrollable container that may require scrolling to find non-visible maps
- **MBTiles vs OZF2 distinction**: Some rows may show only the project name (e.g., "Nizovskaya") without the map type. Inspect the row content more carefully to differentiate.
- **Map Bundles window timing**: Although the click succeeded, there may be a delay before the bundle list populates. Consider a small wait between the click and AX tree capture.
- **Fixture location assumption**: This test assumes the fixture is loaded into a project called "Local OZI" or similar. If project enumeration is not working, the bundle list may be empty. Verify in prior smoke (`smoke-bundle-open.md`).
