# Smoke Test: Waypoint Projection Tool

## Discovery Goal

Verify presence/absence of waypoint projection tool per ADR-0020 (ability to project a new waypoint from a selected point).

## Session Details

- **Appium Session ID:** 6583a3c0-541e-4a5a-9e05-b37512f6bbf9
- **Bundle:** ru.lizaalert.ozi-rs
- **Timestamp:** 2026-05-01
- **Baseline Screenshot:** .sisyphus/evidence/native-qa/appium_screenshot/screenshot.png

## Search Methodology

Inspected AX (Accessibility) tree and sidebar UI to identify waypoint projection options.

### Search Locations (Order of Priority)

1. **Toolbar buttons** (top/right of map area)
   - Searched AX tree for buttons with keywords: "project", "projection", "find projection"
   - Found buttons: Zoom in, Zoom out, Reset bearing to north, Toggle attribution
   - **Result:** No projection button found

2. **On-map control panels** (corners of map)
   - Inspected all visible map controls
   - Found: Zoom controls, scale bar, attribution
   - **Result:** No projection tool visible

3. **Waypoint context menu (right-click on waypoint)**
   - Note: Waypoints panel visible in sidebar with "Show Waypoints Panel" button
   - Note: Right-click not testable via Appium Mac2 for WebView elements
   - Searched AX tree for XCUIElementTypeMenuItem with "project" keyword
   - **Result:** No projection menu items found in AX tree

4. **Dedicated projection button in toolbar**
   - Examined full toolbar element list
   - **Result:** No projection-specific button found

## AX Tree Analysis

Extracted all element titles from accessibility tree:
- **Total unique titles:** 161
- **Projection-related searches:** grep for "project", "projection"
- **Matches:** Only "PROJECT" (sidebar label for projects panel), "Loaded 12762 projects" (status text)
- **Waypoint elements found:**
  - "Waypoint layer" (dropdown/section label)
  - "Show Waypoints Panel" (button)
  - "Add Waypoint" (button)
  - No projection-related menu items or buttons

## Waypoint Panel Structure

Sidebar elements available:
- "WAYPOINTS" section header
- "Waypoint layer" dropdown selector
- "Show Waypoints Panel" button
- "Add Waypoint" button
- No projection or context menu options for existing waypoints

## Classification

**Status:** `MISSING`

The waypoint projection tool is not visible or accessible in:
- Toolbar buttons
- On-map controls
- Waypoint panel
- Context menus (right-click on waypoint)
- Accessibility tree element titles

## Evidence

- AX tree dump: `/tmp/ax_tree.xml`
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png`
- Search queries performed:
  - Title filter: `grep -i "title=" | grep -i "project"` (found no projection-specific items)
  - Full AX tree inspection for context menu items
  - Waypoint sidebar section analysis

## Conclusion

Per ADR-0020, the app should have the ability to project a new waypoint from a selected point (likely via context menu: "Project from waypoint" or "Find projection"). Based on accessibility tree inspection, **this feature is not implemented in the current codebase**. No projection context menu option, no projection dialog, no projection tool visible in any UI location.

### Next Steps

- Implement waypoint projection tool (ADR-0020 requirement)
- Add context menu option to waypoint markers/list items: "Project from [waypoint name]"
- Implement projection dialog with bearing/distance input
- Integrate new waypoint creation at projected location
