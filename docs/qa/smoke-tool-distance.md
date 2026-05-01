# Smoke Test: Distance Measurement Tool

## Discovery Goal

Verify presence/absence of distance measurement tool per ADR-0020 (on-map measurement tools).

## Session Details

- **Appium Session ID:** 6583a3c0-541e-4a5a-9e05-b37512f6bbf9
- **Bundle:** ru.lizaalert.ozi-rs
- **Timestamp:** 2026-05-01
- **Baseline Screenshot:** .sisyphus/evidence/native-qa/appium_screenshot/screenshot.png

## Search Methodology

Inspected AX (Accessibility) tree via WebDriver source API to identify measurement tools.

### Search Locations (Order of Priority)

1. **Toolbar buttons** (top/right of map area)
   - Searched AX tree for buttons with keywords: "measure", "distance", "ruler"
   - Found buttons: Zoom in, Zoom out, Reset bearing to north, Toggle attribution
   - **Result:** No distance measurement button found
   - **Evidence:** Full toolbar element list extracted via jq filtering

2. **On-map control panels** (corners of map)
   - Inspected left sidebar and right-side controls
   - Found: Zoom controls, scale indicator (1 km), attribution
   - **Result:** No measurement panel visible

3. **Sidebar menu / Tools section**
   - Examined left sidebar controls under "TRACKS", "WAYPOINTS" sections
   - Found: Import GPX, Import PLT, Create Track, Add Waypoint, Show/Hide panels
   - **Result:** No "Tools" menu with measurement options

4. **Right-click context menu on map**
   - Note: Context menus not directly testable via Appium Mac2 (no right-click event support in WebView automation)
   - Searched AX tree for XCUIElementTypeMenuItem with measurement keywords
   - **Result:** Found only system writing tools in menu tree

## AX Tree Analysis

Extracted all element titles from accessibility tree:
- **Total unique titles:** 161
- **Measurement-related searches:** grep for "measure", "distance", "ruler", "circle", "radius", "project", "draw"
- **Matches:** None matching measurement tool patterns
- **Only matches found:** "PROJECT" (sidebar label for projects panel), "Writing Tools" (system menu)

## Classification

**Status:** `MISSING`

The distance measurement tool is not visible or accessible in:
- Toolbar buttons
- On-map controls
- Sidebar menu sections
- Accessibility tree element titles

## Evidence

- AX tree dump: `/tmp/ax_tree.xml` (full WebDriver source)
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png`
- Search queries performed:
  - Title filter: `grep -i "title=" | grep -i "measure|distance|ruler|circle|radius|project|draw"`
  - jq AX tree filtering for button/menu items with measurement keywords

## Conclusion

Per ADR-0020, the app should have a distance measurement tool. Based on accessibility tree inspection, **this feature is not implemented in the current codebase**. No measurement mode, no distance calculation tool, no ruler widget visible in UI.

### Next Steps

- Implement distance measurement tool (ADR-0020 requirement)
- Add toolbar button or menu entry to activate measurement mode
- Integrate distance calculation between two map points
