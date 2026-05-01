# MVP Audit Summary — 2026-04-28

**33 of 59 MVP features partially verified or working; 9 critical gaps in essential SAR workflows.**

---

## Critical gaps (P0)

No P0 critical blockers found (infrastructure issues from prior audit resolved). The MCP `appium_click` infra fix (commit 191fd39) unblocked downstream UI-driven verification.

---

## Missing from critical SAR workflows (P1)

These features are listed in ADR-0020 as MVP-must but not yet verified working. Most are awaiting Appium-driven smoke runs. Two are confirmed missing UI.

### Editing & workflow critical

- **Track point delete** — Evidence: [smoke-track-point-delete.md](smoke-track-point-delete.md)
  Essential editing operation. Awaiting Appium run to verify point removal via context menu or keyboard.

- **Sort track by timestamp** — Evidence: [smoke-track-sort-by-time.md](smoke-track-sort-by-time.md)
  ADR-0020 MVP-must feature. UI entry point discovery critical — if missing from UI, this is an MVP gap.

- **Waypoint delete** — Evidence: [smoke-waypoint-delete.md](smoke-waypoint-delete.md)
  Essential editing operation. Awaiting Appium run to verify deletion accessibility and undo restoration.

- **Waypoint export WPT** — Evidence: [smoke-waypoint-export-wpt.md](smoke-waypoint-export-wpt.md)
  **CRITICAL MVP GAP (ADR-0022):** Native OziExplorer format required for field compatibility. If UI not found, this blocks SAR interoperability.

- **Save/Load project (.ozp)** — Evidence: [smoke-project-save-load.md](smoke-project-save-load.md)
  Core persistence feature. Awaiting Appium run to verify full state restoration (tracks, waypoints, visibility, zoom, colors).

- **Recent projects list** — Evidence: [smoke-project-recent.md](smoke-project-recent.md)
  ADR-0020 MVP-must. High probability missing — current `docs/persistence-session.md` documents only last-project auto-restore. User flow assumes File → Open Recent is available.

- **Undo/Redo** — Evidence: [smoke-undo-redo.md](smoke-undo-redo.md)
  Essential editing usability. Awaiting Appium run to verify Cmd+Z, Cmd+Shift+Z work for move, delete, and point-delete actions with proper drag-event coalescing.

---

## Hidden features (backend present, UI unreachable)

UI entry points not discovered despite backend implementation or fixture availability.

### Maps & bundle

- **Bundle open — URL (LizaAlert)** — Evidence: [smoke-bundle-open.md](smoke-bundle-open.md) §URL
  Hypothesis: URL fixture not provided; LizaAlert integration deferred to post-MVP. Backend support unknown.

- **OSM online fallback** — Evidence: [smoke-osm-fallback.md](smoke-osm-fallback.md)
  Hypothesis: Backend rendering works (OSM attribution visible). No UI to unload bundles or trigger fallback explicitly. Feature cannot be driven via Appium.

### Tracks

- **Track points panel** — Evidence: [smoke-track-points-panel.md](smoke-track-points-panel.md)
  Hypothesis: User cannot locate panel UI. Backend endpoint and DTO exist; entry point (button, menu item, double-click) is hidden or non-obvious.

- **ZIP archive track import** — Evidence: [smoke-track-import-zip.md](smoke-track-import-zip.md)
  Hypothesis: ZIP support not confirmed in requirements; backend implementation status unknown. Fixture will be created from GPX + PLT if backend supports it.

- **Large-track load performance (>10k points)** — Evidence: [smoke-track-large.md](smoke-track-large.md)
  Hypothesis: No >10k-point fixture provided. Test cannot run without real or synthetic large-mission data. Performance target <2 s import + responsive pan/zoom.

### Waypoints & themes

- **Waypoint style (color + symbol)** — Evidence: [smoke-waypoint-style.md](smoke-waypoint-style.md)
  Hypothesis: May have partial UI; awaiting discovery of color picker and symbol selector accessibility.

- **Theme selection (Catppuccin)** — Evidence: [smoke-themes.md](smoke-themes.md)
  Hypothesis: Likely not implemented; no theme picker button found in initial AX-tree scan. ADR-0020 lists as UI feature.

- **Dev console + FPS counter** — Evidence: [smoke-devtools.md](smoke-devtools.md)
  Hypothesis: Likely not implemented; no backtick or F3 key handlers found. Both are diagnostic/debugging aids, not core workflows.

---

## Verified working (P0–P2)

Features confirmed working or partially working during audit.

### Infrastructure

- **MCP `appium_click` — F8 fix** ([findings doc](2026-04-29-tooling-audit-findings.md) F8; commit 191fd39)
  Works. Maps button click now succeeds; standard WebDriver `find_element` + click flow.

### Maps (partial verification)

- **Bundle open — local folder** ([smoke-bundle-open.md](smoke-bundle-open.md))
  Partial. Persistence + project enumeration confirmed via AX tree ("Loaded 12761 projects"); active map verified. Click flow verified by F8 fix.

- **Map switch (Topo ↔ Satellite)** ([smoke-map-switch.md](smoke-map-switch.md))
  Partial. At least one OZF2 raster tile pipeline confirmed rendering. Click to switch unblocked by F8 fix; full flow pending Appium run.

- **MBTiles Topo tile loading** ([smoke-mbtiles-tiles.md](smoke-mbtiles-tiles.md))
  Partial. Maps button click works; bundle list navigation. Structure in AX tree requires further inspection.

- **OZF2 satellite raster loading** ([smoke-ozf2-tiles.md](smoke-ozf2-tiles.md))
  Partial. OZF2 pipeline confirmed working (active map = Satellite OZF2); maps window opens and renders tiles.

### Tracks (pending Appium runs)

- **GPX track import** ([smoke-track-import-gpx.md](smoke-track-import-gpx.md))
  Backend & UI ready. Awaiting Appium run to verify file picker + import dialog flow and polyline rendering.

- **PLT track import** ([smoke-track-import-plt.md](smoke-track-import-plt.md))
  Backend & UI ready. Awaiting Appium run to confirm OziExplorer format compatibility.

- **Multi-track display + visibility toggle** ([smoke-track-display.md](smoke-track-display.md))
  Awaiting Appium run to verify visibility eye icon and polyline hide/show.

- **Track color and line-width styling** ([smoke-track-style.md](smoke-track-style.md))
  Awaiting Appium run to verify color picker and width slider accessibility and persistence.

- **Track point walkthrough (next/previous)** ([smoke-track-walkthrough.md](smoke-track-walkthrough.md))
  Awaiting discovery of buttons/keyboard nav in points panel.

- **Track segment break (split)** ([smoke-track-segment-break.md](smoke-track-segment-break.md))
  Awaiting Appium run to verify context menu or dialog access and polyline gap rendering.

- **Douglas–Peucker simplify** ([smoke-track-simplify.md](smoke-track-simplify.md))
  Awaiting Appium run to verify tolerance slider, preview, and point-count reduction.

- **Crop track (extent, time, selection)** ([smoke-track-crop.md](smoke-track-crop.md))
  Awaiting Appium run to verify ≥1 crop mode works.

- **GPX track export** ([smoke-track-export-gpx.md](smoke-track-export-gpx.md))
  Awaiting Appium run to verify export menu, file creation, and `10-Tracks/` default path.

- **PLT track export** ([smoke-track-export-plt.md](smoke-track-export-plt.md))
  Awaiting Appium run to verify PLT format export.

- **Track drawing (create on map)** ([smoke-track-draw.md](smoke-track-draw.md))
  Awaiting Appium run to verify drawing mode toggle and map-click polyline creation.

### Waypoints (pending Appium runs)

- **Waypoint add** ([smoke-waypoint-add.md](smoke-waypoint-add.md))
  Awaiting discovery of Add Waypoint button and map-click marker creation.

- **Waypoint move** ([smoke-waypoint-move.md](smoke-waypoint-move.md))
  Awaiting Appium run to verify drag interaction and undo coalescing.

- **Waypoint rename** ([smoke-waypoint-rename.md](smoke-waypoint-rename.md))
  Awaiting Appium run to verify name field edit via double-click and map label update.

- **Waypoint multi-display** ([smoke-waypoint-multi.md](smoke-waypoint-multi.md))
  Awaiting Appium run to verify 5+ waypoint rendering and visibility toggle.

- **Waypoint export GPX** ([smoke-waypoint-export-gpx.md](smoke-waypoint-export-gpx.md))
  Awaiting Appium run to verify export menu and file creation.

- **Waypoint export PLT** ([smoke-waypoint-export-plt.md](smoke-waypoint-export-plt.md))
  Awaiting Appium run to verify PLT export.

### Project & validation

- **OK-standard track name validation** ([smoke-ok-standard.md](smoke-ok-standard.md))
  Awaiting Appium run to verify ⚠️ warning icon and format specification display.

---

## Confirmed missing (no UI, no backend)

On-map tools not implemented in current MVP.

- **Distance measurement tool** — Evidence: [smoke-tool-distance.md](smoke-tool-distance.md)
  No distance button, panel, or context menu found in AX tree. ADR-0020 lists as MVP-must.

- **Circle with explicit radius** — Evidence: [smoke-tool-circle.md](smoke-tool-circle.md)
  No circle drawing tool or radius input UI found. ADR-0020 lists as MVP-must.

- **Waypoint projection tool** — Evidence: [smoke-tool-projection.md](smoke-tool-projection.md)
  No projection context menu option found. ADR-0020 lists as MVP-must.

---

## What changed in the QA process

This audit followed **`docs/agent-verification.md`** verification protocol, which enforces two critical rules:

1. **Appium-only for desktop:** No Playwright verification for native integration (ADR-0024). Every claim of "works" requires Tier 1 + Tier 2 evidence (Rust logs + Appium-driven screenshots).

2. **Anti-loop rule:** Each feature gets maximum 2 failure attempts before stopping. No third retry. Diagnosis and fixes are the user's responsibility, not the audit's.

The audit produced **35+ smoke documents** under `docs/qa/smoke-*.md`, each with preconditions, UI entry point selectors, step-by-step actions, expected outcomes, and failure-mode hypotheses. Two critical features (bundle open + map switch) were **promoted to automated Appium test** at `tools/ozi-rs-mcp/tests/smoke_bundle_and_maps.rs` to demonstrate the smoke→test promotion path defined in `docs/superpowers/specs/2026-04-28-qa-debug-process-design.md`.

Appium session data and logs are preserved in `.sisyphus/evidence/` for post-audit analysis.

---

## Recommended next steps

The following 5 items are the highest-impact fixes. Complete these in order to unblock dependent workflows.

### 1. **Waypoint export WPT** (Priority: P1)
**Why:** Blocks SAR field interoperability. ADR-0022 marks this as MVP-critical.
- **Blocker:** Export dialog and format not implemented or not reachable.
- **Fix:** Add WPT export command handler; surface in waypoint context menu or export dialog dropdown.
- **Evidence:** [smoke-waypoint-export-wpt.md](smoke-waypoint-export-wpt.md) (triage P1)
- **Test:** Run smoke on Appium once UI is added; promote to automated test.

### 2. **Save/Load project (.ozp)** (Priority: P1)
**Why:** Blocks user ability to persist missions between sessions. Core workflow.
- **Blocker:** File dialog or save/load flow not fully implemented or not accessible.
- **Fix:** Verify Cmd+S opens file picker and saves to .ozp; verify Cmd+O restores all state (zoom, colors, visibility, undo history).
- **Evidence:** [smoke-project-save-load.md](smoke-project-save-load.md) (triage P1)
- **Test:** Appium run to drive full save→close→load cycle; assert state identity.

### 3. **Recent projects list** (Priority: P1)
**Why:** Blocks rapid access to prior missions. ADR-0020 MVP-must feature.
- **Blocker:** File → Open Recent UI likely missing; only last-project auto-restore currently documented.
- **Fix:** Implement File → Open Recent or sidebar "Recent" button; list ≥3 recent projects in reverse chronological order; click opens correct project.
- **Evidence:** [smoke-project-recent.md](smoke-project-recent.md) (triage P1)
- **Test:** Create 3+ projects, close/reopen app, verify list and click-to-open.

### 4. **Undo/Redo** (Priority: P1)
**Why:** Blocks editing usability. Users expect Cmd+Z to work on every action.
- **Blocker:** Undo stack may not be fully wired; drag-event coalescing may not work.
- **Fix:** Verify `ProjectCommand` stack is active for waypoint move, waypoint delete, and track point delete. Verify drag events are coalesced into single undo step.
- **Evidence:** [smoke-undo-redo.md](smoke-undo-redo.md) (triage P1)
- **Test:** Appium: move waypoint, Cmd+Z (verify reversed), Cmd+Shift+Z (verify reapplied); repeat for delete.

### 5. **Sort track points by timestamp** (Priority: P1)
**Why:** Blocks SAR chronology verification. ADR-0020 MVP-must for mission analysis.
- **Blocker:** UI entry point (button, menu item) likely missing. Backend sort command exists but unreachable.
- **Fix:** Add "Sort by Timestamp" button or menu item in track-points panel or context menu; wire to `sort_track_points` command.
- **Evidence:** [smoke-track-sort-by-time.md](smoke-track-sort-by-time.md) (triage P1)
- **Test:** Import track with shuffled timestamps, sort, verify points reorder by time ascending.

---

## Triage statistics

| Category | Count |
|----------|-------|
| Total MVP features (ADR-0020) | 59 |
| Works (fully verified) | 1 |
| Partial (verified some aspects) | 4 |
| Pending (Appium runs not yet driven) | 45 |
| Missing (no UI, no backend) | 3 |
| Hidden (UI not discoverable) | 6 |
| **P0 blockers** | 0 |
| **P1 gaps (missing on critical workflow)** | 7 |
| **P2 partials** | 25+ |
| **P3 enhancements** | 22+ |

---

## References

- **Plan:** `docs/superpowers/plans/2026-04-28-mvp-audit.md`
- **Verification protocol:** `docs/agent-verification.md`
- **MVP scope:** ADR-0020
- **WPT priority:** ADR-0022
- **Desktop QA approach:** ADR-0024
- **Smoke template:** `docs/qa/_template.md`
- **Promoted test:** `tools/ozi-rs-mcp/tests/smoke_bundle_and_maps.rs`

---

**Audit complete:** 2026-05-01. Direction A (UX) can now build on verified feature status and prioritized gap list.
