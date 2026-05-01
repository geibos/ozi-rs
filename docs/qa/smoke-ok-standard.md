# Smoke: OK-standard Track Name Validation

Source: LizaAlert OK Standard (YYYYMMDD_Callsign format), referenced in project memory.

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` with at least one track imported
- Track name currently follows or does NOT follow OK-standard format
- Waypoints panel and Tracks panel visible

## UI entry point

- **Track rename action**: Accessible via:
  1. Double-click track name in Tracks panel
  2. Right-click track row → Rename option
  3. Select track row and press F2 or Enter key

- **Validation indicator**: Expected at:
  1. Next to track name in panel (⚠️ icon or red highlight)
  2. In a validation message or status bar
  3. In an info panel or tooltip

- **Selector candidates**:
  - Track name field: `//XCUIElementTypeTextField[contains(@label, 'track')]` or by position in Tracks panel
  - Warning icon: `//XCUIElementTypeImage[contains(@label, 'warning')]`
  - Validation message: `//XCUIElementTypeStaticText[contains(@label, 'format')]`

## OK-Standard Format

**Expected format**: `YYYYMMDD_Callsign`
- `YYYY` = 4-digit year (e.g., 2026)
- `MM` = 2-digit month (01-12)
- `DD` = 2-digit day (01-31)
- `_` = underscore separator
- `Callsign` = alphanumeric search/rescue team identifier (e.g., ALPHA, K9-01, TEST)

**Example valid names**:
- `20260501_ALPHA` (May 1, 2026, team ALPHA)
- `20260215_K9_01` (Feb 15, 2026, team K9-01)
- `20260428_Search001` (April 28, 2026, team Search001)

**Example invalid names**:
- `foo` (no date, no separator)
- `2026-05-01_team` (date format wrong; use YYYYMMDD not YYYY-MM-DD)
- `20260501` (no callsign)
- `ALPHA_20260501` (reverse order)
- `2026/05/01_ALPHA` (wrong date separator)

## Steps and expected outcomes

### Part A: Test Non-Conforming Name Warning

1. **Action**: `appium_launch_session` with track(s) loaded
   **Expected**: Session active; Tracks panel visible with at least one track.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing Tracks panel with current track names
   **Expected**: Track name(s) visible in panel.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-ok-standard.png`

3. **Action**: Record baseline track name from AX tree
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-ok-baseline.json`
   - `grep -i "track" /tmp/ax-tree-ok-baseline.json | grep -i "label\|value" | head -5`
   **Expected**: AX tree shows track name(s).
   **Artifact**: Bash output.

4. **Action**: Rename track to non-conforming name: `foo`
   - Double-click track name field OR right-click and select Rename
   - Clear current name and type `foo`
   - Press Enter to confirm
   **Expected**: Track name changes to `foo` in panel.
   **Artifact**: `appium_screenshot` showing renamed track.

5. **Action**: Look for validation warning
   - Check AX tree for warning elements:
     - `grep -i "warning\|invalid\|format" /tmp/ax-tree-ok-warning.json`
   - Check for visual indicator next to track name (⚠️ icon, red highlight, etc.)
   **Expected**: Warning icon or message appears near track name, e.g., "Track name does not follow YYYYMMDD_Callsign format".
   **Artifact**: `appium_screenshot` showing warning; AX tree snippet showing warning element.

6. **Action**: Verify warning is specific to invalid format
   - AX tree should show warning label or tooltip indicating format requirement
   - `grep -C2 "YYYYMMDD\|format\|standard" /tmp/ax-tree-ok-warning.json`
   **Expected**: Warning text mentions expected format.
   **Artifact**: AX tree snippet with warning message.

### Part B: Test Valid Name Validation Pass

7. **Action**: Rename track to conforming name: `20260501_TestTeam`
   - Double-click track name field again
   - Clear and type valid name: `20260501_TestTeam`
   - Press Enter to confirm
   **Expected**: Track name updates to `20260501_TestTeam`.
   **Artifact**: `appium_screenshot` showing new name.

8. **Action**: Verify warning disappears
   - Check AX tree for absence of warning elements:
     - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-ok-valid.json`
     - `grep -i "warning\|invalid" /tmp/ax-tree-ok-valid.json | grep -i "track\|20260501"` (should be empty)
   - Check visual appearance: no ⚠️ icon, no red highlight next to track name
   **Expected**: Warning disappears; track name displays normally (black text, no decoration).
   **Artifact**: `appium_screenshot` showing valid name without warning; AX tree snippet confirming no warning.

### Part C: Edge Cases (Optional, if time permits)

9. **Action**: Test other invalid formats to verify validation breadth
   - Rename to: `2026-05-01_ALPHA` (wrong date format)
   - Expected: Warning appears
   - Artifact: screenshot

10. **Action**: Rename to: `20260501` (missing callsign)
    - Expected: Warning appears
    - Artifact: screenshot

11. **Action**: Rename to: `ALPHA_20260501` (reversed order)
    - Expected: Warning appears
    - Artifact: screenshot

12. **Action**: Rename to valid format with underscores in callsign: `20260501_ALPHA_TEAM`
    - Expected: No warning (underscores allowed in callsign portion)
    - Artifact: screenshot

## Classification

- [ ] **works** — Non-conforming track name triggers warning (visible icon or message); warning specifies YYYYMMDD_Callsign format; warning disappears when valid name entered
- [ ] **partial** — Warning appears for invalid format, but doesn't disappear on valid name; OR warning is generic (doesn't specify format)
- [ ] **hidden** — No validation warning found in UI; backend may perform validation on save/export
- [ ] **missing** — No validation mechanism; any track name accepted without warning

**Expected classification: `works` or `partial` (P2).**

Rationale: OK-standard track naming is a SAR convention per LizaAlert. Validation should guide users to follow the standard without blocking operations. Success criteria: (1) non-conforming name triggers visible warning, (2) warning message specifies expected format, (3) warning clears on valid name.

## Evidence

- Appium session ID
- Screenshot (baseline track names): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-ok-standard.png`
- Screenshot (renamed to invalid name): track-renamed-invalid.png
- Screenshot (warning visible): track-warning-visible.png
- AX tree (with warning): `/tmp/ax-tree-ok-warning.json` (bash verification only)
- Screenshot (renamed to valid name): track-renamed-valid.png
- Screenshot (warning gone): track-warning-gone.png
- AX tree (valid name): `/tmp/ax-tree-ok-valid.json` (bash verification only)
- Screenshots (edge cases): edge-case-*.png
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **No validation warning appears**: Validation feature may not be implemented. Check if backend stores tracks with any name validation.
- **Warning appears but is generic**: Message may not mention YYYYMMDD_Callsign. Classify as `partial` and note that format specification is missing.
- **Warning remains after valid name entered**: Validation may be based on last keystroke only; try waiting 500ms after typing for debounce.
- **Track rename doesn't work**: Rename field may not be editable. Try context menu → Rename instead of double-click.
- **Session crashes on rename**: Text field handling may cause crash. Check logs for exceptions.

---

## References

- Project memory: LizaAlert OK Standard (YYYYMMDD_Callsign format)
- Task 5: Track import and feature audit
- Task 9: Project-level features audit
