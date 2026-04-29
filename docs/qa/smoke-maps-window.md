# Smoke: Maps window

## Preconditions

- App is built with the current `src-tauri/capabilities/default.json`.
- App is launched via the native QA `launch_app` tool.
- The main `ozi-rs` window is visible.

## Steps and expected outcomes

1. Observe the baseline main window.
   Expected: the sidebar is visible and contains the `Maps…` button in the `Map` section.
2. Click the `Maps…` button.
   Expected: a separate `Map Bundles` window becomes visible within 1 second.
3. Observe logs after the click.
   Expected: no Tauri ACL/permission error for `plugin:window|get_all_windows`, `plugin:window|show`, `plugin:window|set_focus`, or `plugin:window|hide`.

## Known failure modes

- `Maps…` appears to do nothing and logs mention a forbidden `plugin:window|...` command → missing Tauri capability permission in `src-tauri/capabilities/default.json`.
- `Map Bundles` opens but is empty → bundle root or LizaAlert project loading state issue; inspect `BundleLoaderView.svelte` state and backend `load_projects` logs.
- Appium cannot click the button → verify macOS Accessibility permissions with `appium_doctor`; Tier 1 screenshot/log evidence is still useful but does not complete the full protocol.
