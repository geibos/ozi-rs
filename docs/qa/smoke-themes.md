# Smoke: Theme Selection

Source: ADR-0020 (MVP scope), UI Features section — "Theme selection (Catppuccin: Latte, Frappé, Macchiato, Mocha, Auto)."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session`
- Map and UI elements visible for visual inspection
- No theme preference set (or reset to default)

## UI entry point

- **Theme picker**: Expected locations:
  1. PROJECT sidebar — "Auto" popupbutton or theme selector (per ADR-0020)
  2. Settings menu or preferences panel
  3. Keyboard shortcut: Alt+T or similar
  4. Right-click context menu on app window

- **Theme options**: Per ADR-0020, should support:
  1. **Latte** — Light theme (Catppuccin Latte: warm, light palette)
  2. **Frappé** — Dark theme (Catppuccin Frappé: cool, dark palette)
  3. **Macchiato** — Dark theme (Catppuccin Macchiato: medium-dark, cool)
  4. **Mocha** — Dark theme (Catppuccin Mocha: very dark, cool)
  5. **Auto** — System preference (follow macOS light/dark setting)

- **Selector candidates**:
  - Theme button in sidebar: `//XCUIElementTypePopUpButton[contains(@label, 'Auto')]`
  - Settings gear icon: `//XCUIElementTypeButton[contains(@label, 'Settings')]`
  - Theme menu: `//XCUIElementTypeMenu[contains(@label, 'Theme')]`

## Steps and expected outcomes

1. **Action**: `appium_launch_session`
   **Expected**: Session active; app running with visible map and sidebar.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline theme (default state)
   **Expected**: App displays with current theme (likely light or system-default).
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-theme-default.png`

3. **Action**: Discover theme picker UI
   - Query AX tree for theme elements:
     - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-themes.json`
     - `grep -i "theme\|auto\|popupbutton\|menu" /tmp/ax-tree-themes.json | head -20`
   - Look for PROJECT sidebar section with theme selector
   - Try keyboard shortcut Alt+T
   **Expected**: Theme picker found in AX tree OR keyboard shortcut opens theme selection.
   **Artifact**: AX tree snippet; or `appium_screenshot` if shortcut works.

4. **IF THEME PICKER FOUND**:

   ### Test Latte Theme

   a. **Action**: Click or select Latte theme from picker
      **Expected**: Theme menu/popover opens (if not already open).
      **Artifact**: `appium_screenshot` showing theme options.

   b. **Action**: Click on "Latte" option
      **Expected**: UI applies light theme; background becomes light (whitish), text becomes dark, buttons show light background.
      **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/theme-latte.png`

   c. **Action**: Wait 0.5 seconds for theme transition
      - `sleep 0.5`
      **Expected**: Theme fully applied (no loading animation visible).
      **Artifact**: None (covered by next screenshot).

   d. **Action**: `appium_screenshot` to verify Latte theme applied
      **Expected**: Light theme visible; sidebar background light, map background light, text dark.
      **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/theme-latte-applied.png`

   ### Test Frappé Theme

   e. **Action**: Click theme picker again
      **Expected**: Theme menu/popover visible.
      **Artifact**: `appium_screenshot`.

   f. **Action**: Click on "Frappé" option
      **Expected**: UI switches to cool dark theme.
      **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/theme-frappe-applied.png`

   ### Test Macchiato Theme

   g. **Action**: Click theme picker again
      **Expected**: Theme menu/popover visible.
      **Artifact**: `appium_screenshot`.

   h. **Action**: Click on "Macchiato" option
      **Expected**: UI switches to medium-dark, cool theme (slightly lighter than Mocha).
      **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/theme-macchiato-applied.png`

   ### Test Mocha Theme

   i. **Action**: Click theme picker again
      **Expected**: Theme menu/popover visible.
      **Artifact**: `appium_screenshot`.

   j. **Action**: Click on "Mocha" option
      **Expected**: UI switches to very dark theme (darkest of dark themes).
      **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/theme-mocha-applied.png`

   ### Test Auto Theme

   k. **Action**: Click theme picker again
      **Expected**: Theme menu/popover visible.
      **Artifact**: `appium_screenshot`.

   l. **Action**: Click on "Auto" option
      **Expected**: UI switches to match system preference (light or dark based on macOS setting).
      **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/theme-auto-applied.png`

   ### Verify Theme Persistence

   m. **Action**: Close app completely (Cmd+Q)
      **Expected**: App terminates; all UI closed.
      **Artifact**: None.

   n. **Action**: Relaunch app
      - `appium_launch_session`
      **Expected**: App opens; last selected theme still applied.
      **Artifact**: Session ID.

   o. **Action**: `appium_screenshot` to verify theme persisted
      **Expected**: Theme matches what was selected before close (e.g., if Mocha was selected, very dark theme visible on relaunch).
      **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/theme-persisted.png`

   p. **Action**: Query AX tree to verify theme selector state
      - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-theme-persisted.json`
      - `grep -i "auto\|mocha\|theme" /tmp/ax-tree-theme-persisted.json | grep -i "selected\|value"`
      **Expected**: AX tree shows persisted theme is selected.
      **Artifact**: AX tree snippet showing selected theme.

   **Classification goes to: `works`**

5. **IF THEME PICKER NOT FOUND**:

   a. **Action**: Document missing theme UI
      - Verify no theme selector in sidebar, menus, or settings
      - Verify keyboard shortcut Alt+T does nothing
      - Search AX tree for any "theme" or "appearance" elements
      **Expected**: No theme UI discoverable.
      **Artifact**: AX tree query output; `appium_screenshot` of menus checked.

   b. **Action**: Check if theme can be set via settings file or config
      - Look for theme preference in app data directory
      - May exist in `~/.config/ozi-rs/` or app cache
      **Expected**: If config file exists, theme is configurable but not via UI.
      **Artifact**: None (backend-only feature).

   **Classification goes to: `missing` (UI absent)**

## Classification

- [ ] **works** — Theme picker found and functional; all 5 themes (Latte, Frappé, Macchiato, Mocha, Auto) apply correctly; visual appearance changes noticeably for each; theme persists across relaunch
- [ ] **partial** — Theme picker found but only some themes work; OR theme applies but doesn't persist; OR picker is in settings but not easily discoverable
- [ ] **hidden** — Theme picker UI not found but theme may be configurable via config file (backend-only)
- [ ] **missing** — No theme support; all app sessions use default single theme

**Expected classification: `works` or `missing` (P3).**

Rationale: Theme selection is UI polish per ADR-0020. If implemented, should support all 5 Catppuccin flavors and persist selection. If missing, app still functional but lacks user preference customization.

## Evidence

- Appium session ID (before and after theme changes)
- Screenshot (baseline/default theme): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-theme-default.png`
- Screenshot (theme picker visible if found): theme-picker-menu.png
- Screenshot (Latte theme applied): `.sisyphus/evidence/native-qa/appium_screenshot/theme-latte-applied.png`
- Screenshot (Frappé theme applied): `.sisyphus/evidence/native-qa/appium_screenshot/theme-frappe-applied.png`
- Screenshot (Macchiato theme applied): `.sisyphus/evidence/native-qa/appium_screenshot/theme-macchiato-applied.png`
- Screenshot (Mocha theme applied): `.sisyphus/evidence/native-qa/appium_screenshot/theme-mocha-applied.png`
- Screenshot (Auto theme applied): `.sisyphus/evidence/native-qa/appium_screenshot/theme-auto-applied.png`
- Screenshot (theme persisted after relaunch): `.sisyphus/evidence/native-qa/appium_screenshot/theme-persisted.png`
- AX tree (themes): `/tmp/ax-tree-themes.json` (bash verification only)
- AX tree (persisted theme): `/tmp/ax-tree-theme-persisted.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Theme picker not found in sidebar**: May be located in Settings menu instead. Check all menus and sidebars.
- **Theme applies but reverts after 1-2 seconds**: Theme transition animation may not be complete. Wait longer before taking screenshot.
- **Only 3-4 themes available instead of 5**: Not all Catppuccin flavors may be implemented. Document which are missing.
- **Theme doesn't persist across relaunch**: Preference storage not implemented. Check for config file; may need manual persistence code.
- **Theme colors appear wrong or washed out**: May be a rendering issue or color palette bug. Compare with reference Catppuccin colors.
- **Switching themes crashes app**: Theme switching code may have bug. Check logs for exceptions.

---

## Approach Note: Visual Verification

Theme testing relies on visual comparison rather than Appium assertions. Each screenshot should show a distinct color palette. Compare:
- **Latte**: Warm, light palette (light beige/white backgrounds, dark text)
- **Frappé**: Cool, dark palette (dark blue-tinted backgrounds, light text)
- **Macchiato**: Medium-dark, cool (slightly lighter than Frappé)
- **Mocha**: Very dark, cool (darkest available)

Do NOT read theme screenshots back as images; use AX tree to verify theme selector state changes instead. Visual inspection is sufficient for classification.

---

## References

- ADR-0020: MVP scope, UI Features section — "Theme selection"
- Catppuccin color palettes: https://catppuccin.com/palette
- Task 9: Project-level features audit
