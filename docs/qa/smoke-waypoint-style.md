# Smoke: Waypoint color and symbol customization

Source: ADR-0020 (MVP scope), section Waypoints — "style waypoint (color, symbol)."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- At least one waypoint exists on map and in waypoints panel
- Waypoint's current color and symbol recorded
- Waypoints panel visible

## UI entry point

- **Style action**: Accessible via:
  1. Right-click on waypoint row in panel → "Style" or "Properties" menu
  2. Color swatch button next to waypoint name in panel
  3. Double-click on waypoint marker on map (opens properties dialog)
  4. Waypoint row → expand arrow revealing style controls (if accordion layout)
  5. Select waypoint → toolbar with color/symbol buttons

- **Selector candidates**:
  - Color swatch: `//XCUIElementTypeButton[contains(@title, "color")]` in waypoint row
  - Style menu: `//XCUIElementTypeMenuItem[contains(@title, "Style\|Properties")]`
  - Symbol picker: `//XCUIElementTypeButton[contains(@title, "symbol\|marker")]`

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; must have at least one waypoint)
   **Expected**: Session active; app running with waypoint visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Waypoint row in panel with current color/symbol; marker on map visible.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-style.png`

3. **Action**: Query AX tree to record current waypoint style:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-before-style.json`
   **Expected**: AX tree contains waypoint row with color/style label or attribute.
   **Artifact**: Bash output: `grep -i "color\|red\|blue\|symbol" /tmp/ax-tree-before-style.json | head -10`

4. **Action**: **Discover style entry point**:
   - Look for color swatch button next to waypoint name
   - Try right-clicking on waypoint row for context menu
   - Try double-clicking on waypoint marker on map
   **Expected**: Style controls or color picker accessible.
   **Artifact**: `appium_screenshot` of color picker or style panel.

5. **Action**: If **color swatch button found**, **click on it**
   **Expected**: Color picker dialog opens or inline color palette appears.
   **Artifact**: `appium_screenshot` of color picker.

6. **Action**: **Select new color** (e.g., red, blue, green, etc.)
   - Click on color swatch in picker
   **Expected**: Picker closes (if modal); waypoint marker color changes on map; row color indicator updates.
   **Artifact**: `appium_screenshot` of waypoint with new color.

7. **Action**: If **symbol/marker picker exists**, **click to change symbol**
   - Symbol options may include: pin, circle, square, cross, flag, etc.
   **Expected**: Symbol changes on map marker; panel row reflects new symbol.
   **Artifact**: `appium_screenshot` of waypoint with new symbol.

8. **Action**: Query AX tree to verify style changed:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-style.json`
   **Expected**: AX tree waypoint row contains new color/symbol label.
   **Artifact**: Bash verification: `diff <(grep -i "color" /tmp/ax-tree-before-style.json) <(grep -i "color" /tmp/ax-tree-after-style.json)`

9. **Action**: `capture_logs` and search for "style" or "color" markers
   **Expected**: Log lines showing waypoint style update operation.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

10. **Action**: **Test persistence** — Close and reopen app (if session allows)
    **Expected**: Waypoint color/symbol persisted to storage and restored on reload.
    **Artifact**: `appium_screenshot` after app restart.

## Classification

- [ ] **works** — Color/symbol picker accessible; changes apply immediately to map and panel; persistence confirmed; undo works
- [ ] **partial** — Picker accessible and color changes but symbol picker missing; or persistence incomplete
- [ ] **broken** — Picker accessible but changes don't apply or cause visual glitches
- [ ] **hidden** — No color picker UI found; style may be hidden or context-menu-only
- [ ] **missing** — No waypoint styling feature

**Expected classification: `partial` (P3).**

Rationale: Visual differentiation aids SAR operations but is lower priority than add/delete/move. May be partial if symbol picker not implemented. Success criteria: (1) color picker accessible, (2) color changes visible on map and panel, (3) persistence across app reload.

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-style.png`
- Screenshot (color picker): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-color-picker.png`
- Screenshot (after color change): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-color.png`
- Screenshot (after symbol change): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-symbol.png`
- AX tree (before style): `/tmp/ax-tree-before-style.json` (bash verification only)
- AX tree (after style): `/tmp/ax-tree-after-style.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Color swatch not found in panel**: Style controls may be in right-click context menu or separate properties panel.
- **Color picker doesn't open**: Swatch may be display-only. Check for double-click or right-click to edit.
- **Color changes don't persist**: Check persistence layer in code. May need manual save or session close.
- **Symbol picker missing**: This feature may be not yet implemented. Classify as `partial`.
- **Marker color doesn't update on map**: Rendering engine may have stale marker. Check for renderer update event.
- **Multiple colors/symbols available but unclear which is which**: UX issue. Document color names/symbols observed in picker.

---

## References

- ADR-0020: MVP scope, Waypoints section — "style waypoint"
- Task 7: Waypoint feature audit
- Task 9.3: Undo/redo operations (related)
- Task 5e: Track styling (related pattern)
