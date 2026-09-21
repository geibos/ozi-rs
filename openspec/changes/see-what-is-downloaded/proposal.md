## Why

The project list is about thirteen thousand identical rows. The backend knows
which of those bundles are already on disk — `is_project_cached` has existed all
along — but the summary sent to the frontend carries only a slug and a name. So
the one question a crew asks in a штаб with no signal, "which of these can I
still open?", has no answer on screen.

The filter makes it worse: it matches the project name only, and names are latin
transliterations of the slug, so a date typed with dashes finds nothing. The
count beside the box shows the catalogue total rather than the number of
matches, which makes filtering look like it did nothing.

## What Changes

- The project summary carries whether the bundle is on disk, computed once per
  catalogue read rather than per row.
- Downloaded projects are marked in the list, and a "downloaded only" toggle
  narrows the catalogue to what can be opened without a signal.
- The filter matches the slug as well as the name, and the count reports
  matches out of the total.

## Capabilities

### Modified Capabilities
- `map-bundles`: what the project list tells the operator about availability.

## Impact

- **Backend**: `cached_project_slugs` in `infrastructure/lizaalert.rs`;
  `LizaProjectSummaryDto` gains `cached`; generated bindings.
- **Frontend**: `src/lib/project-list.ts` (new), `BundleLoader.svelte`, the
  localStorage catalogue cache, i18n keys.
- **Risk**: low. A stale or missing flag defaults to "not downloaded", which is
  the safe answer — the backend re-sends fresh flags on every catalogue read.
