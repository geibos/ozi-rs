# MVP Audit Summary — 2026-04-28

**5 of 40 audited features were driven through to a verdict on this run; 3 were confirmed missing; 4 are hidden by design or fixture limit; the remaining 28 have smoke documents but were not driven through Appium and stay as `pending`. Direction A (UX) should treat `pending` rows as "test plan ready, not result."**

The audit's deliverable was **the smoke harness, the triage, and the F8 fix that unblocks future runs** — not a fully driven Appium pass over the MVP. Treat the numbers below accordingly.

---

## What was actually verified on this run

These five rows have evidence beyond the smoke document itself.

| Feature | Verdict | Evidence |
|---------|---------|----------|
| MCP `appium_click` infrastructure (F8) | **works** | Unit tests + live MCP re-verification; commit `191fd39`. The Maps… button now clicks end-to-end (AX tree grew 57 KB → 304 KB, two windows). |
| Bundle open — local folder | **partial** | AX tree shows `Loaded 12761 projects`, active map = `2018-09-26_Nizovskaya_Satell_z17_ozf.map`. Bundle persistence and project enumeration confirmed. The user-facing click flow itself is *not* yet driven. |
| OZF2 satellite raster loading | **partial** | Same AX tree evidence — OZF2 pipeline is rendering on startup. The pipeline is alive; switching between OZF2 entries was not driven. |
| Map switch (Topo ↔ Satellite) | **partial** | At least one tile pipeline (OZF2) is alive. The actual switch gesture (click an alternate map row) was not driven; bundle list AX-tree structure is unclear and needs a follow-up pass now that F8 is fixed. |
| MBTiles Topo tile loading | **partial** | Maps button + window open verified. Bundle list rows for `.sqlitedb` entries not yet identified in the AX tree. |

Five rows. Four of them are `partial — verified-by-AX-tree`; only the F8 infra fix has a full pass.

---

## Confirmed missing on this run (3)

These three were searched for in the AX tree across toolbar, sidebar, on-map controls, and context menus — and not found. ADR-0020 lists each as MVP-must.

- **On-map distance measurement** — [smoke-tool-distance.md](smoke-tool-distance.md). No "measure" button, no distance panel, no context-menu item.
- **Circle with explicit radius** — [smoke-tool-circle.md](smoke-tool-circle.md). No circle tool, no radius input.
- **Waypoint projection** — [smoke-tool-projection.md](smoke-tool-projection.md). No "project from waypoint" menu item, no dedicated tool.

The On-map Tools section of ADR-0020 is, in practice, unimplemented today. This is the cleanest finding in the audit and the clearest backlog item.

---

## Hidden by design or fixture (4)

Backend may exist, but the audit cannot reach it.

- **Bundle open by URL (LizaAlert)** — [smoke-bundle-open.md](smoke-bundle-open.md) §URL. No URL fixture provided this run; ADR-0020 schedules LizaAlert integration as out-of-MVP.
- **OSM online fallback** — [smoke-osm-fallback.md](smoke-osm-fallback.md). OSM attribution is visible in the rendered map, but there is no UI to deactivate the active bundle and trigger fallback, so the path cannot be driven via Appium.
- **ZIP archive track import** — [smoke-track-import-zip.md](smoke-track-import-zip.md). Backend support not confirmed; fixture creation deferred.
- **Large-track (>10k points) load** — [smoke-track-large.md](smoke-track-large.md). No fixture in `example_data` exceeds ~80 points. Performance target of <2 s import + responsive pan is untested.

---

## Not driven this run — pending Appium (28)

These have smoke documents in `docs/qa/smoke-*.md`, but the actions inside (clicks, drags, exports, keyboard shortcuts) **were not executed** during this audit. Each is "test plan ready, result unknown."

A run that drives these will move them to one of `works / partial / broken / missing`. Until then, the rows in `triage.md` are *predictions* informed by the AX tree and code reading, not findings.

| Area | Pending rows |
|------|--------------|
| Maps & bundles | (none — covered by partials above) |
| Tracks — import / display / style | GPX import, PLT import, multi-track display + visibility, track color & line-width styling |
| Tracks — editing | points panel discovery, walkthrough, point delete, segment break, sort by timestamp, Douglas–Peucker simplify, crop (extent/time/selection), drawing |
| Tracks — export | GPX export, PLT export |
| Waypoints | add, move, rename, delete, style (color + symbol), multi-display + toggle, GPX export, PLT export, **WPT export** |
| Project & UI | save/load `.ozp`, recent projects, undo/redo, OK-standard validation, theme selection, dev console + FPS |

**WPT export is the highest-stakes pending item.** Per ADR-0022 it is MVP-critical for OziExplorer field-device interoperability. The audit could neither confirm nor deny the UI exists, because the export menu was never opened.

---

## P0 / P1 status — what to fix once smokes are driven

The current `triage.md` priority assignments reflect predicted risk. Once Appium runs land, each row will move to a verified verdict and either drop down (works/partial) or stay (missing).

- **P0 (broken on critical path):** 0 rows. The infrastructure that previously gated the audit (F8) has been fixed.
- **P1 (missing on critical path) — predicted, not yet verified:** 7 rows. WPT export, save/load `.ozp`, recent projects, undo/redo, track point delete, waypoint delete, sort by timestamp.
- **P2 (partial / pending on any feature):** 23 rows.
- **P3 (hidden / missing off critical path):** 10 rows including all three on-map tools (confirmed missing) and theme/devtools (predicted missing).

The seven P1 predictions should be the *first* targets of the next driven run, because each one's verdict materially changes the MVP gap list.

---

## What this audit produced

- **40 smoke documents** under `docs/qa/smoke-*.md`. Each one has preconditions, an entry-point candidate (selector or AX path), step-by-step actions, and a classification rubric.
- **A prioritized triage** at `docs/qa/triage.md` (sorted P0 → P3, then by task number).
- **An audit-verified column** in `docs/feature-status.md` linking each ADR-0020 row to its smoke.
- **One promoted automated test** at `tools/ozi-rs-mcp/tests/smoke_bundle_and_maps.rs`. The test currently runs the launch + screenshot + log-capture skeleton on the bundle-open / map-switch path; it does not yet drive the clicks (the smoke docs are the spec for adding those assertions). The promoted-test slot exists; the assertions inside need filling out as the click-driven smokes are run.
- **Tooling fixes** that unblock everything else:
  - F8 (`191fd39`) — `appium_click` and `appium_type_text` now use the standard W3C WebDriver `find_element` + element-action flow instead of the Mac2-specific endpoint that didn't accept selectors.
  - F1 / F6 / F7 fixes from the prior tooling audit (`c25c2a9`, `998bc13`, `d206239`) — Mac2 session creation now succeeds reliably with `appium:bundleId` and a 60 s session-create budget.

---

## QA process changes anchored by this run

- **Verification protocol** at `docs/agent-verification.md` is now the canonical "what counts as evidence" rule. Two failed attempts on a single feature → stop, classify, hand back; no third try.
- **ADR-0024** (Appium-only desktop verification, no Playwright) is now load-bearing: every smoke in this batch assumes Mac2 driver evidence, and the promoted test asserts the same.
- **The smoke → automated test promotion path** is demonstrated end-to-end (smoke doc → Rust test under `tools/ozi-rs-mcp/tests/`). This is the pattern the next round of driven smokes should follow as they pass.

---

## Recommended next steps (in order)

1. **Drive the seven predicted-P1 smokes through Appium.** WPT export, save/load `.ozp`, recent projects, undo/redo, track point delete, waypoint delete, sort by timestamp. Each verdict here is high-leverage: a `missing` verdict is a real MVP gap; a `works` verdict closes 7 / 28 of the pending list. Budget: one driven run per smoke, anti-loop rule honored.
2. **Drive the four Maps partials forward.** Now that F8 is fixed, the Map Bundles bundle-list rows should be reachable. Goal: upgrade `partial → works/broken` for bundle-open (local), map switch, MBTiles, OZF2. Identify the per-row XPath inside the bundle list and update both smoke docs and the promoted test.
3. **Fill out the assertions inside `smoke_bundle_and_maps.rs`.** The skeleton currently captures evidence but does not yet click and assert. As step 2 produces stable selectors, lift them into the test as constants (already stubbed) and add: click Maps…, assert bundle list non-empty, click an alternate row, assert tile-source-changed log line.
4. **Decide WPT scope.** If 7.9 (`smoke-waypoint-export-wpt.md`) verifies as `missing`, schedule the implementation explicitly — ADR-0022 makes it MVP-must, and this is the largest known interoperability gap.
5. **Schedule the on-map tools backlog item.** All three (distance, circle, projection) are confirmed missing; they need to be planned as a coherent toolbar, not three separate one-off features.

---

## Triage statistics — verified-only view

| Category | Count | Comment |
|----------|------:|---------|
| Smoke documents created | 40 | Each one is a runnable test plan. |
| Driven through to a verdict on this run | 5 | F8 (works) + 4 partials. |
| Confirmed missing on this run | 3 | All three on-map tools. |
| Hidden by design or fixture | 4 | Bundle URL, OSM fallback, ZIP import, large-track. |
| Pending (smoke ready, not yet driven) | 28 | Need a follow-up Appium pass. |
| **P0 blockers** | 0 | F8 closed the only known infra blocker. |
| **P1 — predicted critical gaps** | 7 | Verdicts pending the next driven run. |
| **P2 — partials & non-critical pending** | 23 | |
| **P3 — hidden / off-critical missing** | 10 | Includes the three confirmed-missing on-map tools. |

---

## References

- **Plan:** [`docs/superpowers/plans/2026-04-28-mvp-audit.md`](../superpowers/plans/2026-04-28-mvp-audit.md)
- **Verification protocol:** [`docs/agent-verification.md`](../agent-verification.md)
- **MVP scope:** ADR-0020
- **WPT priority:** ADR-0022
- **Desktop QA approach:** ADR-0024
- **Tooling-fixes findings:** [`docs/qa/2026-04-29-tooling-audit-findings.md`](2026-04-29-tooling-audit-findings.md)
- **Triage:** [`docs/qa/triage.md`](triage.md)
- **Feature status (audit-verified column):** [`docs/feature-status.md`](../feature-status.md)
- **Smoke template:** [`docs/qa/_template.md`](_template.md)
- **Promoted test:** [`tools/ozi-rs-mcp/tests/smoke_bundle_and_maps.rs`](../../tools/ozi-rs-mcp/tests/smoke_bundle_and_maps.rs)

---

**Audit run complete:** 2026-05-01.
**Honest framing:** the harness, the triage, and the F8 unblock are the deliverables of this run. The next run, driving the 28 pending smokes through Appium, is what turns the harness into actual MVP coverage.
