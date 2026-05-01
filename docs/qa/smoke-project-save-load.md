# Smoke: Save and Load Project

Source: ADR-0020 (MVP scope), Project Management section — "Save project to .ozp file" and "Load existing project."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` with a bundle loaded and tracks/waypoints added
- Sample tracks and waypoints available from prior audit tasks (e.g., imported from GPX)
- Temp directory available for project file: `/tmp/test-project.ozp`

## UI entry point

- **Save action**: Accessible via:
  1. Keyboard shortcut: Cmd+S
  2. File menu → Save
  3. PROJECT sidebar — look for "Save" button or icon

- **Open action**: Accessible via:
  1. File menu → Open
  2. Keyboard shortcut: Cmd+O
  3. Recent Projects list (if implemented)

- **Selector candidates**:
  - File menu: `//XCUIElementTypeMenuBarItem[contains(@label, 'File')]`
  - Save button in sidebar: `//XCUIElementTypeButton[contains(@label, 'Save')]`

## Steps and expected outcomes

1. **Action**: `appium_launch_session` with bundle and content loaded (map visible, tracks panel populated)
   **Expected**: Session active; app running with visible project state (map, tracks, waypoints).
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state with tracks and waypoints visible
   **Expected**: Map with rendered tracks; Tracks panel and Waypoints panel populated with elements.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-save-load.png`

3. **Action**: Query AX tree to record current tracks and waypoints:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-before-save.json`
   **Expected**: AX tree contains Tracks panel rows and Waypoints panel rows with names/counts.
   **Artifact**: Bash verification: `grep -i "track\|waypoint" /tmp/ax-tree-before-save.json | head -10`

4. **Action**: Discover Save entry point
   - Try Cmd+S keyboard shortcut
   - Or locate File menu and select Save
   **Expected**: File dialog opens with default location.
   **Artifact**: `appium_screenshot` showing file dialog.

5. **Action**: Verify file dialog defaults to project directory
   **Expected**: Current path shows default project location (likely app data folder or user documents).
   **Artifact**: `appium_screenshot` of file dialog path field.

6. **Action**: Navigate to `/tmp` and save file as `test-project.ozp`
   - Type filename in file dialog
   - Click Save button
   **Expected**: Dialog closes; project saved to `/tmp/test-project.ozp`; confirmation message may appear.
   **Artifact**: `appium_screenshot` after save; bash verification: `ls -lh /tmp/test-project.ozp`

7. **Action**: Verify file exists with reasonable size
   - `ls -lh /tmp/test-project.ozp`
   **Expected**: File exists and has non-zero size (at least 1 KB).
   **Artifact**: Bash output showing file size.

8. **Action**: Close app completely
   - Cmd+Q or File → Exit
   **Expected**: App terminates.
   **Artifact**: Session ID null.

9. **Action**: Relaunch app
   - `appium_launch_session` with same app path
   **Expected**: App opens with empty/default project state.
   **Artifact**: Session ID.

10. **Action**: `appium_screenshot` showing empty state
    **Expected**: No tracks or waypoints visible initially.
    **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-empty-after-relaunch.png`

11. **Action**: Discover Open entry point
    - Try Cmd+O keyboard shortcut
    - Or locate File menu and select Open
    **Expected**: File dialog opens.
    **Artifact**: `appium_screenshot` showing file dialog.

12. **Action**: Navigate to `/tmp` and open `test-project.ozp`
    - Type `/tmp/test-project.ozp` in path field or browse to file
    - Click Open button
    **Expected**: File dialog closes; project loads.
    **Artifact**: `appium_screenshot` after load operation.

13. **Action**: Wait 1-2 seconds for project load to complete
    - Observe map rendering and panel population
    **Expected**: Tracks render on map; Tracks and Waypoints panels populate with same data as baseline.
    **Artifact**: `appium_screenshot` showing loaded project.

14. **Action**: Query AX tree to verify restored state:
    - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-load.json`
    **Expected**: Waypoint and track counts match baseline; panel structure intact.
    **Artifact**: Bash verification: `grep -i "track\|waypoint" /tmp/ax-tree-after-load.json | head -10`

15. **Action**: Compare AX tree counts before save and after load
    - `diff <(grep -oP '(?<=label=")[^"]+' /tmp/ax-tree-before-save.json | sort) <(grep -oP '(?<=label=")[^"]+' /tmp/ax-tree-after-load.json | sort)`
    **Expected**: Track names, waypoint names, and counts are identical (or minimal diff for generated IDs).
    **Artifact**: Bash verification output.

16. **Action**: Visually verify map state
    - Check zoom level, center point, track colors, waypoint styles
    **Expected**: Map view, track colors, and waypoint styles match baseline.
    **Artifact**: `appium_screenshot` for visual comparison.

17. **Action**: `capture_logs` and search for project load/save markers
    - `grep -i "load\|save\|project" .sisyphus/evidence/native-qa/capture_logs/stdout.txt | head -20`
    **Expected**: Log lines showing successful project load, parsing, and state initialization.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — Project saves with all tracks/waypoints; reloads with identical state (positions, names, colors, zoom)
- [ ] **partial** — Save succeeds but load only restores tracks (not waypoints), OR colors/zoom not restored
- [ ] **broken** — Save dialog opens but save fails; OR file created but load fails/corrupts state
- [ ] **hidden** — No Save/Open UI found in File menu or via Cmd+S/Cmd+O
- [ ] **missing** — Save/Load feature absent; .ozp format not implemented

**Expected classification: `works` (P1).**

Rationale: Project persistence is critical MVP feature per ADR-0020. Success criteria: (1) save dialog reachable, (2) file created at requested path, (3) project reopens with all tracks/waypoints/state restored.

## Evidence

- Appium session ID (before and after)
- Screenshot (baseline before save): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-save-load.png`
- Screenshot (file dialog during save): file-dialog-save.png
- Screenshot (empty after relaunch): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-empty-after-relaunch.png`
- Screenshot (file dialog during open): file-dialog-open.png
- Screenshot (after load): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-load.png`
- AX tree (before save): `/tmp/ax-tree-before-save.json` (bash verification only)
- AX tree (after load): `/tmp/ax-tree-after-load.json` (bash verification only)
- Saved file: `/tmp/test-project.ozp` (for manual inspection if needed)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **File dialog doesn't open**: Cmd+S may not work on Mac2. Try File menu click instead.
- **Save dialog shows wrong default path**: Check app configuration for project directory setting.
- **File created but zero-size**: Serialization may have failed. Check logs for JSON encoding errors.
- **Project loads but tracks missing**: Track list may be in a different serialization key. Check JSON structure.
- **Map zoom/center not restored**: View state may not be persisted. Verify `viewState` is included in .ozp serialization.
- **Waypoint colors not restored**: Style struct may not be serialized. Check Rust struct derivation (Serialize/Deserialize).
- **Session crashes during load**: Corrupted .ozp file or parsing error. Validate JSON with `jq` before retrying.

---

## References

- ADR-0020: MVP scope, Project Management section
- Task 9: Project-level features audit
- docs/persistence-session.md: Session and project persistence design
