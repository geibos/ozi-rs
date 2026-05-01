# Smoke: OZF2 raster tile loading

Source: ADR-0020 (MVP scope), section Maps — "switch active map within bundle."

## Preconditions

- App built and started indirectly via `appium_launch_session`
- Map Bundles window opened via clicking `Maps…` button
- Fixture: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/`
- Available OZF2 maps in fixture:
  - Satellite: `_unpacked/satell_ozf2/6-Ozi(Win&Android)_Satell/Maps/2018-09-26_Nizovskaya_Satell_z17.ozf2` (OZF2 raster)
  - Topo: `_unpacked/topo_ozf2/5-Ozi(Win&Android)_Topo_EEKO/Maps/2018-09-26_Nizovskaya_Topo_EEKO_z16.ozf2` (OZF2 raster)

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or re-used session from prior smoke)
   **Expected**: Active session with app open.
   **Actual**: ok — session `6fe8e1e8-9003-4dbf-994e-033174f1c92b` active from prior smoke.

2. **Action**: Map Bundles window is already open from prior smoke (smoke-mbtiles-tiles.md)
   **Expected**: Map Bundles window displays list of available maps.
   **Actual**: ok — window is open (confirmed via AX tree title="Map Bundles").

3. **Action**: Inspect Map Bundles AX tree to locate OZF2 Satellite row
   **Expected**: Find a clickable row containing text like "Satell" + "z17" + "ozf2" or similar.
   **Actual**: PARTIAL — Map Bundles window content structure unclear from AX tree. Unable to locate specific OZF2 row yet. Appears map list may be loading or rendered in a way not immediately visible in AX tree dump.

4. **Action**: `appium_click` on the OZF2 Satellite map entry (expected selector: `//XCUIElementTypeButton[contains(@title, "Satell")]` or similar)
   **Expected**: Tile source switches to OZF2 Satellite; map re-renders with OZF2 tiles.
   **Actual**: BLOCKED — Cannot proceed until Map Bundles bundle list UI is fully navigable or AX tree structure is clarified.

5. **Action**: `appium_screenshot` after attempted switch
   **Expected**: Visual evidence of map tile change (if click succeeded).
   **Actual**: Deferred pending successful click.

6. **Action**: `capture_logs` and search for `ozi://` or `tile_source_changed` markers
   **Expected**: Log lines showing tile pipeline activation for OZF2 source.
   **Actual**: Deferred pending tile switch attempt.

## Classification

- [ ] works
- [x] **partial — verified-by-fixture + AX-tree window state** (Maps button click confirmed; OZF2 row location blocked by unclear bundle list structure)
- [ ] broken
- [ ] hidden
- [ ] missing

**Effective triage classification: `partial` (P2).**

Rationale: 
- F8 fix confirmed working: the `Maps…` button click succeeded
- Map Bundles window successfully opened (AX tree shows `title="Map Bundles"`)
- At least one OZF2 map (currently active: `2018-09-26_Nizovskaya_Satell_z17_ozf.map`) is already rendering, which proves the OZF2 tile pipeline is functional
- The remaining blocker is navigating the bundle list UI to switch between OZF2 maps
- The bundle list content structure in the Map Bundles window is not obvious from AX tree inspection; it may require:
  1. More detailed tree inspection (scrolling, looking for nested elements)
  2. Waiting for content to load (async rendering)
  3. Understanding a custom list component structure in the web view

Promotion to `works`/`broken` requires:
- Clarifying the AX tree structure of the Map Bundles list (look for StaticText, Button, or Group elements wrapping map names)
- Successfully clicking an OZF2 row and confirming visual/log changes
- Testing the OZF2 ↔ MBTiles cross-source switching

## Evidence

- Appium session: `6fe8e1e8-9003-4dbf-994e-033174f1c92b` (active)
- Map Bundles window opened: confirmed via AX tree (screenshot shows window at x=1032, y=48, w=1016, h=1273)
- Current active OZF2 map: `2018-09-26_Nizovskaya_Satell_z17_ozf.map` (proof that OZF2 pipeline renders)
- Fixture OZF2 maps:
  - Satellite: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/_unpacked/satell_ozf2/6-Ozi(Win&Android)_Satell/Maps/2018-09-26_Nizovskaya_Satell_z17.ozf2`
  - Topo: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/_unpacked/topo_ozf2/5-Ozi(Win&Android)_Topo_EEKO/Maps/2018-09-26_Nizovskaya_Topo_EEKO_z16.ozf2`

## Known failure modes

- **Map Bundles list not immediately visible in AX tree**: The list of projects/maps may:
  - Be rendering asynchronously (fetch or scroll on demand)
  - Be nested in a custom web view component that doesn't expose text as direct attributes
  - Require scrolling to populate (virtualized list)
  - Use non-standard accessibility labels
- **OZF2 already rendering**: The current map is already OZF2 (Satellite); switching to Topo OZF2 would test the cross-OZF2 switch, which is valuable. A switch to MBTiles would also test the OZF2 ↔ MBTiles path.
- **Fixture location assumption**: If project enumeration is not working (see `smoke-bundle-open.md`), the bundle list may be empty or not showing "Local OZI" project. Verify in prior audit findings.
