## Why

Adding the waypoint colour turned up a class rather than an instance: the whole
`.ozp` format rests on `#[serde(default)]` being in the right places. A field
added to any persisted struct without one turns **every project a crew has ever
saved** into a load error, and the only place that shows up is a load — nothing
in the build says a word.

The format has no version field and no migration path (tracked as CJ-8), so
"the older file still loads" is not a nicety; it is the whole compatibility
story.

What existed guarded the project shell: a legacy file with no layers at all.
Nothing guarded what is inside them — tracks, segments, points, styles,
waypoints.

## What Changes

- A test loads the smallest project an early build could have written *with
  contents*: a track of one segment and two points, and a waypoint, carrying
  only the fields that have always existed. It asserts the defaults that
  absence is supposed to mean — a track with no style recorded is visible, a
  waypoint with no symbol has none and is visible.
- It passes today, which is the finding: the format is genuinely tolerant. Now
  it stays that way.

Verified by removing one `#[serde(default)]` — from `Track.style` — and
watching the test name the field it could no longer read.

## Impact

- Affected specs: `project-persistence`
- Affected code: `src-tauri/src/infrastructure/persistence.rs` (tests only)
