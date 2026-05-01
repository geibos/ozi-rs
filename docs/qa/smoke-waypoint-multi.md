# Smoke: Multiple waypoints display and visibility toggle

Source: ADR-0020 (MVP scope), section Waypoints — "display multiple waypoints; visibility toggle."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Waypoint add feature working (Task 7a prerequisite)
- Map visible with sufficient area for 5+ waypoint markers
- Waypoints panel visible

## UI entry point

- **Multi-waypoint display**: Via add waypoint action (Task 7a) repeated 5+ times
- **Visibility toggle**: Accessible via:
  1. Eye icon next to waypoint row in waypoints panel (show/hide individual waypoint)
  2. Eye icon in waypoints panel header (show/hide all waypoints)
  3. Checkbox in waypoint row

- **Selector candidates**:
  - Eye icon (individual): `//XCUIElementTypeButton[contains(@title, "eye\|show\|hide")]` in waypoint row
  - Eye icon (all): `//XCUIElementTypeButton[contains(@title, "eye\|show\|hide")]` in panel header
  - Visibility checkbox: `//XCUIElementTypeCheckBox` in waypoint row

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; app running)
   **Expected**: Session active; app ready for waypoint operations.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Empty or partially populated waypoints panel.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-multi.png`

3. **Action**: **Add 5 waypoints** to the map (using add feature from Task 7a)
   - Click Add Waypoint button (or draw mode toggle)
   - Click 5 different locations on the map
   **Expected**: 5 waypoint markers visible on map; 5 rows in waypoints panel.
   **Artifact**: `appium_screenshot` after adding 5 waypoints.

4. **Action**: Query AX tree to verify all waypoints in panel:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-5-waypoints.json`
   **Expected**: AX tree contains 5 waypoint rows (countable).
   **Artifact**: Bash output: `grep -c "waypoint\|Waypoint" /tmp/ax-tree-5-waypoints.json`

5. **Action**: **Discover visibility toggle**:
   - Look for eye icon next to each waypoint row
   - Look for eye icon in panel header
   - Look for checkbox in waypoint row
   **Expected**: At least one visibility control found.
   **Artifact**: `appium_screenshot` showing visibility control.

6. **Action**: If **eye icon found next to individual waypoint**, **click it to hide waypoint**
   **Expected**: Waypoint row shows "hidden" state (dimmed, strikethrough, or icon changes); marker disappears from map.
   **Artifact**: `appium_screenshot` of map with one marker hidden.

7. **Action**: **Verify map shows 4 visible markers** (5th hidden)
   **Expected**: Map rendering shows 4 markers; panel shows all 5 rows but one is marked hidden.
   **Artifact**: `appium_screenshot` of map state.

8. **Action**: Query AX tree to verify hidden state:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-hide.json`
   **Expected**: AX tree waypoint row has hidden/disabled attribute.
   **Artifact**: Bash verification: `grep -i "hidden\|disabled" /tmp/ax-tree-after-hide.json | grep -i "waypoint"`

9. **Action**: **Click eye icon again to show hidden waypoint**
   **Expected**: Waypoint row returns to visible state; marker reappears on map.
   **Artifact**: `appium_screenshot` of map with all 5 markers visible again.

10. **Action**: If **panel-level eye icon exists**, **click to hide all waypoints**
    **Expected**: All 5 markers disappear from map; all rows in panel show hidden state.
    **Artifact**: `appium_screenshot` of empty map with hidden waypoints.

11. **Action**: **Click panel eye icon again to show all**
    **Expected**: All 5 markers reappear on map; all rows show visible state.
    **Artifact**: `appium_screenshot` of map with all 5 markers.

12. **Action**: `capture_logs` and search for "visibility\|hide\|show" markers
    **Expected**: Log lines showing visibility toggle events for individual and/or all waypoints.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — 5+ waypoints display simultaneously on map and in panel; individual and/or panel-level visibility toggle works; hidden waypoints don't render
- [ ] **partial** — Multiple waypoints display; visibility toggle present but unreliable or incomplete (individual works, all doesn't; or vice versa)
- [ ] **broken** — Multiple waypoints added but visibility toggle doesn't hide markers; or map performance degraded with 5+ markers
- [ ] **hidden** — Visibility toggle UI not found; markers may be always visible
- [ ] **missing** — Multi-waypoint or visibility feature absent

**Expected classification: `works` (P2).**

Rationale: Multiple waypoints and visibility management essential for SAR operations with complex scenarios. Success criteria: (1) 5+ markers render on map simultaneously, (2) individual toggle hides/shows single marker, (3) panel-level toggle hides/shows all (if present).

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-multi.png`
- Screenshot (5 waypoints added): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-5-waypoints.png`
- Screenshot (one hidden): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-one-hidden.png`
- Screenshot (all hidden): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-all-hidden.png`
- Screenshot (all visible): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-all-visible.png`
- AX tree (5 waypoints): `/tmp/ax-tree-5-waypoints.json` (bash verification only)
- AX tree (after hide): `/tmp/ax-tree-after-hide.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Map shows only 2-3 markers even with 5+ waypoints added**: Marker rendering may have limit or zoom issue. Check AX tree to confirm all 5 rows exist in backend.
- **Visibility eye icon not found**: Feature may not be implemented. Classify as `hidden`.
- **Clicking eye icon doesn't hide marker**: Check logs for visibility toggle event. Rendering engine may not refresh.
- **Performance degrades with 5+ markers**: Document frame rate drop. May indicate need for marker clustering or LOD.
- **Individual toggle works but panel toggle missing**: Classify as `partial`.
- **Session crashes with 5+ waypoints**: Check Appium logs for memory or rendering exception.

---

## References

- ADR-0020: MVP scope, Waypoints section — "display multiple waypoints; visibility toggle"
- Task 7a: Waypoint add (prerequisite)
- Task 7: Waypoint feature audit
