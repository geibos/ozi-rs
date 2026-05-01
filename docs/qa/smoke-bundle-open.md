# Smoke: Open bundle from local folder

Source: ADR-0020 (MVP scope), section Maps.

## Preconditions

- App built (Task 1, this plan)
- App launched indirectly via `appium_launch_session` (Mac2 driver start the
  app from `appium:bundleId`)
- Fixtures: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/`
  - `2018-09-26_Nizovskaya_Topo_EEKO_z16.sqlitedb` (Topo MBTiles)
  - `2018-09-26_Nizovskaya_Satell_z17.sqlitedb` (Satellite MBTiles)
  - `_unpacked/topo_ozf2/...Maps/` (OZF2 raster, `.map` + `.ozf2`)
  - `_unpacked/satell_ozf2/...Maps/` (OZF2 raster, `.map` + `.ozf2`)

## UI entry point

- Selector candidate: `//XCUIElementTypeButton[@title="Maps…"]` (visible in
  the AX tree as a sidebar button under the `MAP` section).
- Notes: Maps button reaches the user via the left sidebar of the main
  ozi-rs window. Confirmed present in AX tree on 2026-05-01.

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (no parameters)
   **Expected**: Mac2 WebDriver session created; app started by the driver
   for `ru.lizaalert.ozi-rs`.
   **Actual**: ok — session `09232fe5-691a-4dbc-ab3c-451966ae6769` created.
   **Artifact**: `.sisyphus/evidence/native-qa/appium/session.json`

2. **Action**: `appium_screenshot` (baseline before any click)
   **Expected**: Screenshot captures the running app.
   **Actual**: ok.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png`
   (do NOT `Read` — binary 4 MB)

3. **Action**: Inspect AX tree via direct WebDriver `GET /session/{sid}/source`
   (workaround used because the MCP does not yet expose a `page_source` tool).
   **Expected**: HTML controls inside the WKWebView appear as
   `XCUIElementType*` nodes with textual content in `@title`.
   **Actual**: ok — 57 KB AX tree returned; every sidebar control visible:

   - `PROJECT` section: `Open`, `Save`, `↩` (undo), `↪` (redo) buttons,
     `Auto` PopUpButton ("Color theme")
   - `MAP` section: `Maps…` button (the bundle-loader entry), then a
     read-only "Active" group showing the **currently active map**:
     `Local OZI` source, with map name
     `2018-09-26_Nizovskaya_Satell_z17_ozf.map`. There is also a
     `Reveal in Finder` button.
   - `TRACKS` section: `Track layer` PopUpButton ("Tracks"), `Import GPX`,
     `Import PLT`, `Create Track`, `Hide Tracks Panel`, `Show Points Panel`.
   - `WAYPOINTS` section: `Waypoint layer` PopUpButton ("Waypoints"),
     `Show Waypoints Panel`, `Add Waypoint`.
   - Status bar: `Loaded 12761 projects` static text — bundle persistence
     and project enumeration ran successfully on app start.
   - Map view: `Zoom in` / `Zoom out` / `Reset bearing to north` buttons,
     `1 km` scale label, MapLibre + OpenStreetMap attribution, Tracks
     panel header `Tracks (0)` with `No tracks loaded`.

4. **Action**: `appium_click` on `//XCUIElementTypeButton[@title="Maps…"]`
   to open the Map Bundles secondary window.
   **Expected**: Map Bundles window opens within 1 s.
   **Actual**: BLOCKED — `appium_click` returned HTTP 404 from
   `/appium/mac2/click` for every selector form attempted (`name=Maps…`,
   `~Maps…`, `//XCUIElementTypeButton[@title="Maps…"]`,
   `//XCUIElementTypeButton[contains(@title,"Maps")]`). Root cause:
   **F8** in `docs/qa/2026-04-29-tooling-audit-findings.md` — the MCP
   wrapper posts `{selector: ...}` to a Mac2 endpoint that expects
   `{x,y}` or `{elementId}` only. This is an MCP-layer bug, not an
   Appium failure and not an app failure.
   **Artifact**: n/a (click never reached the element)

5. **Action**: `appium_stop_session`
   **Expected**: ok.
   **Actual**: HTTP 404 — Mac2 driver had already terminated the session
   after the click failures. Direct `DELETE /session/{sid}` confirmed
   `invalid session id` (already gone). Safe — the session was cleaned
   up implicitly.

## Classification

- [ ] works
- [x] **partial — verified-by-AX-tree** (works on first-run / persistence
  level; click-driven re-open path NOT tested due to F8)
- [ ] broken
- [ ] hidden
- [ ] missing

**Effective triage classification: `partial` (P2).**

Rationale: From AX-tree evidence, **bundle-open already worked at least
once** — the active map row shows `2018-09-26_Nizovskaya_Satell_z17_ozf.map`,
which is part of the `_unpacked/satell_ozf2/...` fixture, and `Loaded
12761 projects` confirms the project-enumeration pipeline ran. So this
audit run cannot classify the user-facing click flow itself. The user's
report ("bundle open broken") may be about specific aspects (URL open,
remote LizaAlert URL, slow first render, dialog cancellation) that the
AX-only Tier-1 view cannot tell apart. Promoting the smoke from
`partial` to `works`/`broken` requires re-running with F8 fixed
(`appium_click` switched to the standard `find_element` + element-click
flow) so the actual user gesture can be driven.

## Evidence

- Build: pre-existing build at `target/debug/bundle/macos/ozi-rs.app`
- Appium session: `09232fe5-691a-4dbc-ab3c-451966ae6769` (terminated)
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png`
- AX tree dump (transient): `/tmp/ax-tree-2.json` (57 KB, dropped after run)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **`appium_click` HTTP 404 on every selector** → F8: MCP wrapper posts
  unsupported body to `/appium/mac2/click`. Fix per the F8 entry. Until
  fixed, controllers may use direct curl `POST /element` +
  `POST /element/{eid}/click` as a documented workaround — but smoke
  classifications should record that the workaround was used.
- **Mac2 driver terminates the session after failed clicks** → not a
  showstopper for the audit, but means each smoke run gets one click
  attempt before the session needs re-launch. Anti-loop still applies.
- **`Maps…` click does nothing in production** (user-reported) — distinct
  failure mode that this Tier-2 run did NOT reproduce because the click
  itself never landed. See `smoke-maps-window.md` for the prior
  Tauri-capability fix.

---

## Bundle open by URL (LizaAlert)

Source: ADR-0020 (MVP scope), section Maps — "open bundle by URL."

**ADR-0020 scope note:** URL-based bundle open is listed as MVP-must.
Expected flow: user pastes/opens a LizaAlert-hosted bundle URL; app
downloads + mounts the bundle.

**Audit run result:** No URL fixture provided. URL-driver fixture
required to exercise this path.

**Classification: `hidden` (P3)** — UI entry may exist; not driven here
due to missing fixture.

**Recommendation:** Provide a real or local-mock LizaAlert bundle URL in
a follow-up audit run.
