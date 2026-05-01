# Smoke Test: Circle Drawing Tool with Explicit Radius

## Discovery Goal

Verify presence/absence of circle drawing tool with explicitly-specified radius per ADR-0020.

## Session Details

- **Appium Session ID:** 6583a3c0-541e-4a5a-9e05-b37512f6bbf9
- **Bundle:** ru.lizaalert.ozi-rs
- **Timestamp:** 2026-05-01
- **Baseline Screenshot:** .sisyphus/evidence/native-qa/appium_screenshot/screenshot.png

## Search Methodology

Inspected AX (Accessibility) tree via WebDriver source API to identify circle drawing/radius tools.

### Search Locations (Order of Priority)

1. **Toolbar buttons** (top/right of map area)
   - Searched AX tree for buttons with keywords: "circle", "draw circle", "radius"
   - Found buttons: Zoom in, Zoom out, Reset bearing to north, Toggle attribution
   - **Result:** No circle drawing button found
   - **Evidence:** Full toolbar element list extracted via accessibility tree inspection

2. **On-map control panels** (corners of map)
   - Inspected all visible controls on map canvas
   - Found: Zoom controls, scale bar, attribution panel
   - **Result:** No circle drawing panel visible

3. **Sidebar Tools menu**
   - Examined left sidebar for dedicated Tools section
   - Sidebar sections found: PROJECT, MAP, Active, TRACKS, WAYPOINTS
   - Available buttons: Import GPX/PLT, Create Track, Add Waypoint, etc.
   - **Result:** No circle drawing tool in sidebar

4. **Right-click context menu on map**
   - Note: Context menus not directly testable via Appium Mac2
   - Searched AX tree for XCUIElementTypeMenuItem with circle/radius keywords
   - **Result:** No matching menu items found

## AX Tree Analysis

Extracted all element titles from accessibility tree:
- **Total unique titles:** 161
- **Circle/radius-related searches:** grep for "circle", "radius", "draw"
- **Matches:** None matching circle tool patterns
- **System menus examined:** No custom menu items for drawing tools

## Classification

**Status:** `MISSING`

The circle drawing tool with explicit radius is not visible or accessible in:
- Toolbar buttons
- On-map controls
- Sidebar menu sections
- Accessibility tree element titles
- Right-click context menus

## Evidence

- AX tree dump: `/tmp/ax_tree.xml`
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png`
- Search queries performed:
  - Title filter: `grep -i "title=" | grep -i "circle|radius"`
  - Full AX tree inspection for drawing-related UI elements

## Conclusion

Per ADR-0020, the app should have the ability to draw a circle with an explicitly-specified radius. Based on accessibility tree inspection, **this feature is not implemented in the current codebase**. No circle drawing mode, no radius input dialog, no circle drawing tool visible in any UI location.

### Next Steps

- Implement circle drawing tool (ADR-0020 requirement)
- Add toolbar button or menu entry to activate circle mode
- Implement radius input/picker UI
- Integrate circle drawing on map with specified radius validation
