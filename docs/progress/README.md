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
| After | [maps tab](2026-09-20-visible-fixes/after-maps-tab.png), [tracks tab](2026-09-20-visible-fixes/after-tracks-tab.png), [row detail](2026-09-20-visible-fixes/after-tracks-rows-detail.png) |
| Automated gates | `just ci` green (248 Rust, 281 frontend), `cargo audit` clean, npm audit gate clean, GitHub Actions green |
| Customer-journey smoke | still owed |

Confirmed on screen (2026-09-21): no straight lines across the map; every row
carries its visibility eye, a round colour swatch, a locate button and the
actions menu; durations read `26d 5h` and `19d 7h`; the Maps tab lists the
active local map instead of claiming the project has none.

Two defects surfaced while verifying and are fixed in the same slice: the row
list went empty because the tab still filtered for `LineString` geometry, and
the icon utilities never applied because the legacy element defaults sat
outside a cascade layer and therefore beat them.

Still visibly wrong, not yet addressed: the orange `Format: YYYYMMDD_Callsign`
line repeats under most rows and dominates the list; the raster's no-data area
is filled solid black by the source map rather than left transparent.

The CJ smoke is still owed. The grant returned on 2026-09-21, but every
rebuild re-prompts for Documents access because the debug bundle is ad-hoc
signed, so the harness needs a stable signing identity before smoke runs can
be routine.
