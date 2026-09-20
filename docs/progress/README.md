# Progress gallery

One entry per merged slice, newest first. Each entry links before/after
screenshots so the state of the interface can be read in one page instead of
being reconstructed from commits.

Screenshots come from the screenshot matrix (`just shots`) once that lands;
until then they are cropped captures of the real window taken through the
native QA harness (`docs/native-qa-mcp.md`).

---

## 2026-09-20 — visible fixes (slice 0.2)

Four defects the owner saw the first time the built app was opened.

- Straight red lines no longer run across the map. Track geometry is emitted
  as one `MultiLineString` part per segment, so the gap between segments is a
  gap, and `split` / `join` are visible on the map for the first time.
- Row buttons render their icons again. The legacy `button` reset in
  `app.css` was applying `padding: 4px 10px` and a border to 24px icon
  triggers, leaving no content box for the glyph and stretching the round
  colour swatch into a rectangle.
- The Maps tab lists the active map even when no LizaAlert project is loaded,
  so a locally opened OZI map no longer sits behind "No maps in this project".
- Durations of a day or more read as `26d 5h` instead of `629h 22m`, with a
  tooltip naming what the figure measures.

Evidence: [`2026-09-20-visible-fixes/`](2026-09-20-visible-fixes/)

| | |
|---|---|
| Before | [maps tab](2026-09-20-visible-fixes/before-maps-tab.png), [tracks tab](2026-09-20-visible-fixes/before-tracks-tab.png) |
| After | **not captured — blocked**, see below |
| Automated gates | `just ci` green (248 Rust, 278 frontend), `cargo audit` clean, npm audit gate clean |
| Customer-journey smoke | not run — blocked by the same cause |

**Why the after-shots are missing.** The native QA harness cannot see the
screen on this machine right now. `xcodebuild` builds WebDriverAgent fine, but
the Mac2 driver host process dies immediately on launch, and a plain
`screencapture` of the whole screen comes back 97.6% black with no windows
listed by the window server — both are the signature of a revoked Screen
Recording / Accessibility grant, most likely dropped when Xcode updated to
26.2. Restoring it is a manual step the owner has to take (System Settings →
Privacy & Security → Screen Recording and Accessibility → allow the terminal
and Xcode Helper), tracked as task 0.4 of `revive-ui-cycle`.

The after-shots and the CJ smoke for this slice are owed as soon as that grant
is back; the code changes themselves are covered by unit tests
(`tracks_geojson_*` in `src-tauri/src/commands/mod.rs`, `maps-list.test.ts`,
`track-duration-format.test.ts`).
