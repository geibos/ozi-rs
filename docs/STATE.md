# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

Last merged slice: **Russian by default** (2026-09-21), after track search, catalogue repair, row density, dev signing and 0.2 visible fixes. `main` is pushed and
`origin/main` is level with it. Automated gates are green: `just ci` runs
rustfmt, clippy, type-checks, 248 Rust tests and 278 frontend tests;
`cargo audit` is clean; the npm audit gate passes with one documented waiver.
GitHub Actions is green on `main` as of c08e05d — all seven jobs, including the
Windows NSIS bundle and the smoke builds on all three platforms.

The project is working through `openspec/changes/revive-ui-cycle`, which
rebuilds the UI development cycle rather than the UI: agents that change
screens cannot currently see them, most frontend tests assert on source text,
and there is one end-to-end scenario. `openspec/changes/codify-architecture-decisions`
is written and validating but not yet archived.

## Next slice

Working goal, set by the owner on 2026-09-21: ozi-rs has to be good enough to
replace OziExplorer in the field, and working with tracks and bundles has to be
fast, comfortable and good-looking. The queue below is ordered by how much of
that it buys, and is meant to be re-read and re-ordered each session rather
than followed blindly.

1. **Bulk visibility for tracks.** Show all / hide all, and show only the
   selected one. With 26 tracks over one basemap, isolating one is currently 26
   clicks.
2. **Layer names from imports.** One layer per imported file, each named
   `Imported tracks: /Users/.../20260708_Veter2.gpx`. The selector is a column
   of paths; the file name alone would do.
3. **Waypoints tab parity.** It is localized now but still has no search and a
   different row rhythm from the tracks tab.
4. **Bundle flow.** Opening a project still means going through the cold-start
   route; the maps list and the download popup have not been looked at since
   the catalogue was repaired.

Then the correctness slice 0.3 (`revive-ui-cycle` tasks 1b.1 … 1b.8): bundles
root persistence, Esc discarding a draw, `qa_observe`, HTTP timeouts, export
errors surfaced.

## Known broken

- **Slice 0.2 still owes its customer-journey smoke.** Screenshots are done
  and in the gallery; the smoke run has not been driven yet.
- **Appium stopped accepting sessions** late on 2026-09-21 (`Resource
  temporarily unavailable`, os error 35) and did not recover from a server
  restart. Screenshots still work through `screencapture -l`, so verification
  continued without it, but driven interaction is currently unavailable.
- **`.mcp.json` still runs a prebuilt `ozi-rs-mcp`** from May, so the July
  fixes to it are absent from agent sessions (`revive-ui-cycle` task 0.3).
- **MapLibre carries a critical advisory** (`GHSA-jrc7-96c5-q579`) whose fix is
  two major versions ahead. Waived with a reason in
  `scripts/npm-audit-gate.mjs`; the upgrade needs its own slice.
- **Human-facing docs still lie in places** — feature-status and roadmap mark
  sort, crop and split/join as absent although they exist; the command
  reference is missing several commands. Listed in the
  `codify-architecture-decisions` design under "Findings", fixed by its task 2.6.

Localization is done for the shell, the rails and the three library tabs. What
is still English: the Track Inspector's field labels, the command palette's own
entries, and the bundle loader. Those are the next translation pass.
