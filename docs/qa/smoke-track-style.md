# Smoke: Per-track color and line-width customization

Source: ADR-0020 (MVP scope), section Tracks — "customize track style (color, line width)."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- At least one track already imported into the active project (from Tasks 5a/5b
  or pre-loaded from a previous session).
- Tracks panel visible in left sidebar with at least one track row.
- Track style controls (color picker, line-width selector) accessible from
  the track row (may be inline, context menu, or dedicated style panel).

## UI entry point

- Style controls are expected to be near or within each track row in the Tracks panel.
- Potential UI patterns:
  - **Inline color swatch**: Colored rectangle within the track row, clickable to open color picker.
  - **Context menu**: Right-click on track row, menu item "Edit style" or "Color".
  - **Dedicated style panel**: Click on track row to expand style options below.
  - **Style button**: Button labeled "Style" or "⚙️" within or near the track row.
- Selector candidates depend on implementation:
  - Color swatch: `//XCUIElementTypeButton[contains(@label, "color")]` or similar
  - Context menu trigger: right-click on track row
  - Style panel: secondary panel or popover with color picker + slider for width

## Steps and expected outcomes

1. **Action**: Use session from Task 5 (tracks imported) or launch fresh via `appium_launch_session`
   **Expected**: Session active; app running with Tracks panel showing at least 1 track.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` (baseline with default track style)
   **Expected**: Screenshot shows track row with default appearance (color likely gray or teal,
   line width default ~2–3 px). Polyline on map visible with default style.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_style_baseline.png`

3. **Action**: Query AX tree to locate style controls for the track
   **Expected**: AX tree contains track row with:
   - Child elements for color control (Button, StaticText, or similar with color-related label)
   - Potentially child elements for line-width control (slider or numeric input)
   **Artifact**: AX tree dump showing track row elements.

4. **Action**: Identify and click the color control (swatch, button, or menu item)
   **Expected**: Color picker appears within 1 s (as dialog, popover, or inline palette).
   **Artifact**: `appium_screenshot` showing color picker open.

5. **Action**: Inspect color picker and select a visually distinct color (e.g., red, blue, or bright green)
   **Expected**: Color picker shows palette or HSV/RGB input; selected color highlights.
   **Artifact**: `appium_screenshot` showing color selection.

6. **Action**: `appium_click` to confirm color selection (may require clicking a "Done" button
   or clicking the color itself, depending on implementation)
   **Expected**: Color picker closes within 1 s; track polyline on map changes to the new color.
   **Artifact**: `appium_screenshot` showing updated track color on map.

7. **Action**: Query AX tree and inspect polyline to verify color change persisted
   **Expected**: Track row reflects new color (swatch or label shows new color); map polyline still displays new color.
   **Artifact**: AX tree dump; visual inspection of polyline color.

8. **Action**: (If line-width control exists) Locate and interact with width slider/input
   **Expected**: Control appears as slider or numeric field in track row or style panel.
   **Artifact**: `appium_screenshot` showing width control.

9. **Action**: (If line-width control exists) Adjust width (e.g., increase to 4–5 px)
   **Expected**: Map polyline thickness increases visibly within 500 ms.
   **Artifact**: `appium_screenshot` showing thicker polyline on map.

10. **Action**: (If line-width control exists) Inspect AX tree to verify width change persisted
    **Expected**: Track row shows new width value; map polyline still displays new thickness.
    **Artifact**: AX tree dump; visual inspection.

11. **Action**: `capture_logs` and search for "style" or "color" markers
    **Expected**: Log lines showing style change (color + width), no errors.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout_style.txt`

## Classification

- [ ] **works** — color + line-width both customizable; changes visible on map and persist
- [ ] **partial** — only color is customizable, or width control missing/not working
- [ ] **broken** — style controls not found or clickable, or changes don't appear on map
- [ ] **hidden** — style controls not implemented or UI not surfaced
- [ ] **missing**

**Expected classification: `works` (P3).**

Rationale: Per-track styling is a convenience feature for SAR workflow (visually
distinguishing multiple search patterns). Success criteria: (1) color picker
accessible from track row, (2) color change appears on map polyline, (3) change
persists after closing picker, (4) if line-width control exists, width changes
also apply and persist.

## Evidence

- Appium session ID
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_style_baseline.png`
- Color picker open: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_color_picker.png`
- After color change: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_color.png`
- After width change (if applicable): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_width.png`
- AX tree (initial): `.sisyphus/evidence/native-qa/appium/ax_tree_style_initial.json`
- AX tree (post-color change): `.sisyphus/evidence/native-qa/appium/ax_tree_style_post_color.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout_style.txt`

## Known failure modes

- **Color control not found in AX tree**: Style UI may be hidden behind a context menu or
  secondary panel. Try right-clicking on track row or clicking a dedicated style button.
  If no style control is exposed, feature may be missing (classify as `hidden`).
- **Color picker does not appear**: Tauri dialog or custom Svelte component may not be
  initializing. Check `src/components/` for color picker component and ensure it's
  imported and wired to the track style handler.
- **Color change selected but doesn't apply**: Backend command handler may not be wired.
  Check `ProjectCommand::SetTrackStyle` in `application/commands.rs` and `set_track_style`
  handler in `commands/mod.rs`; verify IPC call in `src/lib/api.ts`.
- **Color changes but doesn't persist**: Changes may only be in-memory (current session).
  Check if `SetTrackStyle` includes persistence to project file; verify `.ozp` save logic.
- **Line-width control not found**: If width customization is not yet implemented, classify
  as `partial` with note: "Color customization works; line-width control not yet implemented."

---

## References

- ADR-0020: MVP scope, Tracks section
- `docs/commands-reference.md`: `SetTrackStyle` command (color, line-width)
- `src/components/Sidebar.svelte`: Tracks panel and potential style UI
- `src/components/TrackRowStyle.svelte` or similar: Style picker component (if exists)
- `src-tauri/src/domain/track_style.rs`: TrackStyle struct (color, line-width fields)
- `src-tauri/src/application/commands.rs`: `SetTrackStyle` command apply/reverse logic
- `src-tauri/src/commands/mod.rs`: `set_track_style` IPC handler
