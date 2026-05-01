# Smoke: Recent Projects List

Source: ADR-0020 (MVP scope), Project Management section — "Open Recent Projects" (MVP-must per ADR-0020).

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- Prior audit completed: smoke-project-save-load.md (3+ test projects available)
- At least 3 test projects saved in `/tmp/` with different names:
  - `/tmp/test-project-1.ozp`
  - `/tmp/test-project-2.ozp`
  - `/tmp/test-project-3.ozp`
- Each project contains distinct tracks or waypoints for identification

## UI entry point

- **Recent Projects list**: Expected locations:
  1. File menu → Open Recent (standard macOS pattern)
  2. Dedicated "Recent Projects" button in PROJECT sidebar
  3. App startup screen with recent list (if splash screen exists)
  4. Keyboard shortcut: Cmd+R or similar

- **Selector candidates**:
  - File menu: `//XCUIElementTypeMenuBarItem[contains(@label, 'File')]`
  - Recent Projects submenu: `//XCUIElementTypeMenuItem[contains(@label, 'Open Recent')]`
  - Recent Projects button: `//XCUIElementTypeButton[contains(@label, 'Recent')]`

## Steps and expected outcomes

1. **Action**: Create 3 test projects (reuse from Task 9.1 or create new)
   - Open app, import different track sets into each project
   - Save to `/tmp/test-project-1.ozp`, `/tmp/test-project-2.ozp`, `/tmp/test-project-3.ozp`
   **Expected**: 3 project files exist with distinct content.
   **Artifact**: Bash verification: `ls -lh /tmp/test-project-*.ozp`

2. **Action**: Open each project in sequence (in reverse order to test recency)
   - Open `/tmp/test-project-3.ozp`, close
   - Open `/tmp/test-project-2.ozp`, close
   - Open `/tmp/test-project-1.ozp`, close
   **Expected**: Projects load and close successfully; app closes after each.
   **Artifact**: Session logs, screenshots of each project state.

3. **Action**: Close app completely (Cmd+Q)
   **Expected**: App terminates.
   **Artifact**: None.

4. **Action**: Relaunch app
   - `appium_launch_session`
   **Expected**: App opens with default/empty state.
   **Artifact**: Session ID.

5. **Action**: `appium_screenshot` showing app startup state
   **Expected**: App displays initial state (may show startup screen, empty map, or File menu).
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-recent-startup.png`

6. **Action**: Search for Recent Projects UI
   - **Method 1**: Query AX tree for "Recent" label
     - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-startup.json`
     - `grep -i "recent\|open" /tmp/ax-tree-startup.json | head -10`
   - **Method 2**: Try keyboard shortcut Cmd+R
   - **Method 3**: Click File menu and look for "Open Recent" submenu
   **Expected**: If recent list exists, UI element found with label containing "Recent".
   **Artifact**: AX tree snippet or `appium_screenshot` showing File menu.

7. **IF FOUND — Recent Projects UI exists**:

   a. **Action**: Click on File menu to reveal Recent submenu
      **Expected**: File menu opens; "Open Recent" submenu visible with list of recent files.
      **Artifact**: `appium_screenshot` showing File → Open Recent submenu with project list.

   b. **Action**: Verify recent projects are listed in reverse chronological order
      **Expected**: Most recently opened project (`test-project-1.ozp`) appears first in list.
      **Artifact**: `appium_screenshot` with file names visible.

   c. **Action**: Click on first recent project in list
      **Expected**: Project opens; Tracks panel populates with that project's content.
      **Artifact**: `appium_screenshot` showing loaded project.

   d. **Action**: Query AX tree to verify correct project loaded
      - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-recent-loaded.json`
      - Verify track names or waypoint count match expected content for that project.
      **Expected**: AX tree shows tracks/waypoints from opened project.
      **Artifact**: Bash verification: `grep -i "track\|waypoint" /tmp/ax-tree-recent-loaded.json | head -10`

   e. **Action**: Close project (Cmd+Q)
      **Expected**: App closes.
      **Artifact**: None.

   f. **Action**: Relaunch and try opening second recent project
      - Open File → Open Recent → second project in list
      **Expected**: Second project opens with its distinct content.
      **Artifact**: `appium_screenshot` confirming different project content.

   g. **Action**: Verify Recent Projects list updates after opening new project
      - Close app, relaunch, check File → Open Recent order
      **Expected**: Newly opened project moves to top of list (most recent).
      **Artifact**: `appium_screenshot` of updated recent list.

   **Classification goes to: `works`**

8. **IF NOT FOUND — Recent Projects UI missing**:

   a. **Action**: Document missing UI
      - Record that File → Open Recent menu not found
      - Verify AX tree has no "Recent" elements
      **Expected**: No recent projects UI discoverable via standard macOS patterns.
      **Artifact**: AX tree query output showing absence of Recent-related elements.

   b. **Action**: Check docs/persistence-session.md for feature documentation
      - If docs claim recent projects should exist: classify as `missing`
      - If docs note "only last project persists": classify as `hidden` (backend exists but UI absent)
      **Expected**: Design doc clarifies scope.
      **Artifact**: Reference to docs/persistence-session.md section.

   c. **Action**: Verify last opened project auto-restores (MVP fallback)
      - Open app → verify that last opened project is loaded automatically
      - OR verify empty state if no prior project opened
      **Expected**: Last project persists across app close/reopen, or app shows empty.
      **Artifact**: `appium_screenshot` showing auto-restored project OR empty state.

   **Classification goes to: `missing` (or `hidden` if only last-project persistence exists)**

## Classification

- [ ] **works** — Recent Projects UI found (File → Open Recent or dedicated button); list shows ≥3 recent files in reverse chronological order; clicking opens correct project
- [ ] **hidden** — No Recent Projects UI found; only last-opened project auto-restores (per docs/persistence-session.md §Last Project); classified as `hidden` (feature exists in backend but no UI)
- [ ] **missing** — No Recent Projects UI and no last-project auto-restore; ADR-0020 gap confirmed

**Expected classification: `missing` or `hidden` (P1 — MVP-must per ADR-0020).**

Rationale: ADR-0020 lists "Open Recent Projects" as MVP-must feature. docs/persistence-session.md may indicate only last project restores (feature present but not UI). If neither exists, this is critical MVP gap.

## Evidence

- Appium session ID
- Screenshot (app startup): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-recent-startup.png`
- Screenshot (File menu with or without Recent): File-menu-*.png
- Screenshot (Recent Projects list if found): recent-projects-list.png
- Screenshot (opened project from recent list): project-from-recent.png
- AX tree (startup): `/tmp/ax-tree-startup.json` (bash verification only)
- AX tree (after opening recent project): `/tmp/ax-tree-recent-loaded.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **File → Open Recent not visible**: Menu may be greyed out if no projects have been opened. Ensure projects were saved and closed properly before relaunching.
- **Recent list empty**: App may not track opened projects. Check for persistence file (`.recent_projects`, registry, or plist).
- **Recent list not in chronological order**: Timestamp tracking may be broken. Verify app records file open time.
- **Clicking recent project doesn't open it**: File path in recent list may be invalid or symlink-broken. Check persistence store for correct paths.
- **Only 1 project appears in recent list**: App may have low MAX_RECENT limit (e.g., 1 instead of ≥3). Check code for constant.

---

## References

- ADR-0020: MVP scope, Project Management section — "Open Recent Projects"
- docs/persistence-session.md: Session and project persistence design (MVP scope clarification)
- Task 9: Project-level features audit
- Task 9.1: Save and Load Project (prerequisite)
