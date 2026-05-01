# Smoke: Developer Tools (Console and FPS Counter)

Source: ADR-0020 (MVP scope), UI Features section — "Dev console toggle and FPS counter."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session`
- Map visible and rendering (some animation or activity for FPS measurement)

## UI entry point

- **Dev console toggle**: Accessible via:
  1. Keyboard shortcut: Backtick (`) key
  2. Menu option: View → Developer Console (if present)
  3. Keyboard shortcut: F12 (alternative)

- **FPS counter toggle**: Accessible via:
  1. Keyboard shortcut: F3 key
  2. Menu option: View → Show FPS (if present)
  3. Keyboard shortcut: Ctrl+Shift+F (alternative)

- **Selector candidates** (for menu approach):
  - View menu: `//XCUIElementTypeMenuBarItem[contains(@label, 'View')]`
  - Console option: `//XCUIElementTypeMenuItem[contains(@label, 'Console')]`

## Steps and expected outcomes

### Part A: Dev Console Toggle

1. **Action**: `appium_launch_session` with map visible
   **Expected**: Session active; app running with map rendered.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state (no console visible)
   **Expected**: App displays map; no console panel visible.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-devtools.png`

3. **Action**: Query AX tree to confirm no console elements
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-devtools-baseline.json`
   - `grep -i "console\|debug" /tmp/ax-tree-devtools-baseline.json | head -10`
   **Expected**: No console or debug-related elements in AX tree.
   **Artifact**: Bash output (should be empty or minimal).

4. **Action**: Toggle dev console by pressing backtick (`) key
   - `appium_key("grave")` (grave accent is backtick in Appium)
   **Expected**: Dev console panel appears on screen (typically bottom or side panel).
   **Artifact**: `appium_screenshot` showing console visible.

5. **Action**: Verify console panel visible in AX tree
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-console-open.json`
   - `grep -i "console\|debug\|textarea\|text.*input" /tmp/ax-tree-console-open.json | head -10`
   **Expected**: AX tree contains console-related elements (input field, output area, etc.).
   **Artifact**: AX tree snippet showing console elements.

6. **Action**: `appium_screenshot` with console visible
   **Expected**: Console panel shows with content area and possible prompt/input field.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/console-open.png`

7. **Action**: Verify console is functional
   - Type a test message into console input field (if visible)
   - OR check if console displays any startup messages or logs
   **Expected**: Console responds to input or displays output.
   **Artifact**: `appium_screenshot` showing console with content.

8. **Action**: Toggle console closed by pressing backtick again
   - `appium_key("grave")`
   **Expected**: Console panel disappears; app shows only map again.
   **Artifact**: `appium_screenshot` showing console gone.

9. **Action**: Verify console removed from AX tree
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-console-closed.json`
   - `grep -i "console\|debug" /tmp/ax-tree-console-closed.json | head -10`
   **Expected**: No console elements in AX tree.
   **Artifact**: Bash output (should be empty).

### Part B: FPS Counter Toggle

10. **Action**: `appium_screenshot` showing baseline state (no FPS counter visible)
    **Expected**: App displays map; no frame-rate number visible in corner.
    **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-fps.png`

11. **Action**: Query AX tree for FPS counter
    - `grep -i "fps\|frame\|rate" /tmp/ax-tree-devtools-baseline.json | head -10`
    **Expected**: No FPS-related elements (or hidden by default).
    **Artifact**: Bash output (should be empty).

12. **Action**: Toggle FPS counter by pressing F3 key
    - `appium_key("F3")`
    **Expected**: FPS frame-rate number appears on screen (typically top-right, top-left, bottom-right, or bottom-left corner).
    **Artifact**: `appium_screenshot` showing FPS counter visible.

13. **Action**: `appium_screenshot` with FPS counter visible
    **Expected**: Frame-rate number visible (e.g., "60 FPS", "59.8 FPS", etc.) in corner of screen.
    **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/fps-counter-visible.png`

14. **Action**: Verify FPS counter in AX tree
    - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-fps-open.json`
    - `grep -i "fps\|frame.*rate\|[0-9][0-9]*\s*fps" /tmp/ax-tree-fps-open.json | head -5`
    **Expected**: AX tree contains FPS counter element with numeric value.
    **Artifact**: AX tree snippet showing FPS element.

15. **Action**: Observe FPS counter for activity
    - Wait 1-2 seconds and take multiple screenshots
    - FPS number may change as app renders frames
    **Expected**: FPS value may vary slightly (59-61 FPS for smooth 60 Hz rendering).
    **Artifact**: Multiple `appium_screenshot` captures showing FPS changes.

16. **Action**: Toggle FPS counter off by pressing F3 again
    - `appium_key("F3")`
    **Expected**: FPS counter disappears from screen.
    **Artifact**: `appium_screenshot` showing FPS gone.

17. **Action**: Verify FPS counter removed from AX tree
    - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-fps-closed.json`
    - `grep -i "fps\|frame" /tmp/ax-tree-fps-closed.json | head -10`
    **Expected**: No FPS elements in AX tree.
    **Artifact**: Bash output (should be empty).

### Part C: Console and FPS Together

18. **Action**: Toggle both console and FPS counter on
    - Press backtick (`) to open console
    - Press F3 to show FPS counter
    **Expected**: Both panels visible simultaneously (console panel + FPS number in corner).
    **Artifact**: `appium_screenshot` showing console and FPS together.

19. **Action**: Verify both elements in AX tree
    - `grep -i "console\|fps\|frame" /tmp/ax-tree-fps-console-both.json | head -10`
    **Expected**: Both console and FPS elements present.
    **Artifact**: AX tree snippet.

20. **Action**: Close console (backtick key)
    **Expected**: FPS counter still visible.
    **Artifact**: `appium_screenshot` showing FPS visible, console gone.

21. **Action**: Close FPS counter (F3 key)
    **Expected**: FPS counter disappears; app shows only map.
    **Artifact**: `appium_screenshot` showing both gone.

## Classification

- [ ] **works** — Dev console toggle (`) works; console displays and can be toggled off; FPS counter toggle (F3) works; FPS number visible and updates; both can be used together
- [ ] **partial** — Console works but FPS missing; OR FPS works but console missing; OR toggle exists but doesn't display content properly
- [ ] **broken** — Keyboard shortcuts recognized (no error) but panels don't appear or don't display content
- [ ] **hidden** — No keyboard shortcuts found; dev tools may exist in menu (not tested) or behind developer flag
- [ ] **missing** — Dev console and FPS counter not implemented

**Expected classification: `works` or `partial` (P3).**

Rationale: Developer tools are optional but useful for debugging. Success criteria: (1) backtick and F3 toggle panels on/off, (2) console displays output, (3) FPS counter shows numeric frame rate.

## Evidence

- Appium session ID
- Screenshot (baseline, no devtools): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-devtools.png`
- Screenshot (console open): `.sisyphus/evidence/native-qa/appium_screenshot/console-open.png`
- Screenshot (console with content): `.sisyphus/evidence/native-qa/appium_screenshot/console-with-content.png`
- Screenshot (console closed): `.sisyphus/evidence/native-qa/appium_screenshot/console-closed.png`
- Screenshot (FPS counter visible): `.sisyphus/evidence/native-qa/appium_screenshot/fps-counter-visible.png`
- Screenshots (FPS counter changing values): fps-frame-1.png, fps-frame-2.png
- Screenshot (FPS counter closed): `.sisyphus/evidence/native-qa/appium_screenshot/fps-counter-closed.png`
- Screenshot (console + FPS together): console-and-fps-together.png
- AX tree (baseline): `/tmp/ax-tree-devtools-baseline.json`
- AX tree (console open): `/tmp/ax-tree-console-open.json`
- AX tree (FPS open): `/tmp/ax-tree-fps-open.json`
- AX tree (both open): `/tmp/ax-tree-fps-console-both.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Backtick key doesn't work**: Keyboard input may not reach app. Try pressing F12 as alternative for console.
- **F3 key doesn't work**: Keyboard input may not reach app. Try View menu → Show FPS instead.
- **Console opens but is empty**: App may not have any log output. Check if console input is functional by typing.
- **FPS counter visible but shows "NaN" or "0 FPS"**: Rendering may have issues or frame-rate calculation broken. Check logs.
- **Console or FPS panel appears but doesn't close**: Toggle state not properly maintained. Try pressing shortcut multiple times.
- **App crashes when opening console**: Console initialization bug. Check logs for exceptions.

---

## Alternative Entry Points

If keyboard shortcuts don't work, try:
- View menu → Developer Tools / Console / FPS
- Settings → Developer Mode (may need to enable first)
- Right-click context menu → Open Developer Tools

---

## References

- ADR-0020: MVP scope, UI Features section — "Dev console toggle and FPS counter"
- Task 9: Project-level features audit
