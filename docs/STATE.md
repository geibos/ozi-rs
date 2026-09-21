# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

Last merged slice: **catalogue repair** (2026-09-21), after row density, dev signing and 0.2 visible fixes. `main` is pushed and
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

**Track layers.** Importing creates one layer per source file — the owner's
project carries 28 of them, with the active layer left empty, so the rail can
look empty while the map is full. Layer ids also collide because they are
allocated as `len() + 1`. Decide how the rail should present multiple layers
(group by layer, or follow the import), then fix the id allocation as part of
the same slice.

After that, **0.3 — correctness fixes** (`revive-ui-cycle` tasks 1b.1 … 1b.8): persist the
bundles root across restarts, make Esc discard an in-progress draw instead of
undoing it onto the redo stack, allocate map layer ids as max+1, restore what
`qa_observe` captures, add HTTP timeouts, surface export errors to the caller,
drop the unused `lucide-svelte` dependency and stop classifying `.kml` archive
entries as supported. Each one gets its test first.

Blocked before that, and blocking verification of everything after it: **task
0.4**, restoring the screen grants (below).

The catalogue works again: the site changed its listing markup in September,
which silently emptied the project list and the per-project map list. Both are
fixed and verified against the live site (13439 projects, maps listed for a
freshly published project).

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
