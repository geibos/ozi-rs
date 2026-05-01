# Smoke: Map switch

Source: ADR-0020 (MVP scope), section Maps — "switch active map within bundle."

## Preconditions

- App built and started indirectly via `appium_launch_session`
- A bundle is already open (verified by AX-tree evidence — see
  `smoke-bundle-open.md`).
- Fixture: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/`

## UI entry point

Per the AX tree captured 2026-05-01, the MAP section of the sidebar
shows:

- **Active map row**: a non-button group with three labels:
  `Local OZI` / `2018-09-26_Nizovskaya_Satell_z17_ozf.map` /
  a `Reveal in Finder` button.

The active row is currently **read-only** in the AX tree — there is no
`Topo` / `Satellite` / OZF2 selector visible at the sidebar level. The
expected place to switch maps is the Map Bundles secondary window
(reached by clicking `Maps…`), but this audit run could NOT open that
window because of F8 (see `smoke-bundle-open.md`).

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (re-used from smoke-bundle-open)
   **Expected**: Active session.
   **Actual**: ok — session `09232fe5-691a-4dbc-ab3c-451966ae6769`.

2. **Action**: `appium_click` on `Maps…`, then on the alternate map row in
   the Map Bundles window.
   **Expected**: Active-map indicator updates to the chosen alternate;
   tile layer re-renders.
   **Actual**: BLOCKED at the first click step — F8 (`appium_click`
   returns HTTP 404). The Maps Bundles window was therefore not opened
   and the alternate map could not be selected.
   **Artifact**: n/a

3. **Action**: AX-tree only inspection (workaround).
   **Expected**: Confirm an active map is rendered.
   **Actual**: ok — active map = `2018-09-26_Nizovskaya_Satell_z17_ozf.map`
   (OZF2 raster from the fixture). Backend tile pipeline reached at
   least once: a satellite OZF2 map is currently visible.

4. **Action**: `capture_logs` for tile-source switch traces.
   **Expected**: Lines containing `tile_source_changed`, `map_switched`,
   `ozi://`, or `sqlite://` scheme markers from a switch event.
   **Actual**: log file captured (3 MB); no switch-event lines surfaced
   because no switch was driven.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] works
- [x] **partial — verified-by-AX-tree** (active map renders; switching
  not driven)
- [ ] broken
- [ ] hidden
- [ ] missing

**Effective triage classification: `partial` (P2).**

Rationale: AX-tree evidence proves at least one tile pipeline (OZF2
raster, the `Local OZI` source named
`2018-09-26_Nizovskaya_Satell_z17_ozf.map`) is rendering. The user's
report ("Maps button unresponsive, cannot switch maps") might describe
either (a) the click itself dropping (covered by `smoke-maps-window.md`
+ the recent capability-permission fix), or (b) the Map Bundles window
not exposing alternate maps as selectable rows. F8 prevents this audit
run from differentiating those.

Promotion to `works`/`broken` requires re-running with F8 fixed and the
following sub-cases:
- Topo (MBTiles) ↔ Satellite (MBTiles) switch
- MBTiles ↔ OZF2 raster switch
- OZF2 ↔ OZF2 switch (multiple OZF2 entries in same bundle)

## Evidence

- Appium session: `09232fe5-691a-4dbc-ab3c-451966ae6769` (terminated)
- Active map at audit time: `2018-09-26_Nizovskaya_Satell_z17_ozf.map`
  (visible in AX tree under MAP > Active)
- Sidebar baseline screenshot:
  `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png` (do
  NOT `Read` — binary, 4 MB)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **F8: `appium_click` HTTP 404** — see findings doc; fix MCP wrapper
  before re-running this smoke.
- **No alternate-map selector in sidebar** → expected, by design;
  selection happens inside Map Bundles window.
- **Tile source does not switch after click** (user-reported, NOT
  reproduced here) → potential causes if it persists after F8 fix:
  click handler not wired in `BundleLoaderView.svelte`; Tauri command
  for `set_active_map` not dispatched; reactive state not bound to the
  rendered tile layer. Investigate after F8.
- **OZF2 vs MBTiles render path divergence** — both should be tested
  separately once switching is drivable; today's run only confirmed
  OZF2 renders.
- **stop_app reliability** — In this run no `stop_app` call was issued
  by the controller (the Mac2 driver had already torn down the
  session and the app process exited along with the session). If a
  future run hits a -1712 timeout, classify as a known sub-issue, do
  not downgrade the feature classification.
