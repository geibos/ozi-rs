## Why

CJ-4 — taking a crew's dirty recording and making a route out of it — is where
a coordinator spends most of their time, and it had never been walked. The
first attempt found three things in twenty minutes.

**Simplify opened with no numbers under the slider.** There are two ways in,
the row's ⋯ menu and the Track Inspector, and only the first asked for a
preview. From the inspector — which is the natural place, it sits under the
track's statistics — the operator got "Допуск: 10 м" and nothing else until
they happened to move the slider. They could simplify a track having never
been told what it would cost, which is the one thing a live preview exists to
prevent.

**Simplify then did not invalidate what is drawn from the track.** Five of the
six geometry operations bump `tracksGeometryVersion`; simplify lives in a
different component from the other five and did not. So the statistics dropped
from five points to three while the segment table beside them still listed
five, and the map still drew the points that were gone. That reads as "nothing
happened", which is worse than an error.

**And the stand could not walk any of it.** Twelve commands had no answer
there — `create_empty_track`, both crops, sort, split, join, simplify, all
three point edits — which is the whole of CJ-4's editing spine. The stand
refuses an unanswered command loudly, which is right, but nobody finds out
until they walk the screen that sends it. That is the fourth such gap in two
days.

## What Changes

- The preview belongs to the dialog, not to whoever opened it. An effect in
  the component that renders it fetches one whenever the dialog is open with
  nothing to show — so both entrances behave the same, and a tolerance change
  simply clears the preview rather than scheduling a second fetch.
- Simplify bumps the geometry version, and a test fails on any component that
  calls a shape-changing command without doing so.
- The stand answers all twelve, each by mutating a session copy of the
  fixture's track detail, so the list, the inspector and the map move
  together. A test fails when a generated command has no answer there.

## Impact

- Affected specs: `track-editing`, `build-tooling`
- Affected code: `src/components/library/TracksTab.svelte`,
  `src/test/stand/tauri-core.ts`
