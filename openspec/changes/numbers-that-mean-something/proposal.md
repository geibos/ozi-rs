## Why

Two places show a number that says nothing.

A track of one point read `0.0 км · 0мин · 1 тчк` — two measurements of nothing
in front of the one measurement there is. Such tracks only became visible at
all a few slices ago, when the list stopped being derived from the map's
geometry, so nobody had seen this.

While the catalogue refreshes, the hint says only that it is refreshing. The
refresh can now be stopped, and the decision to stop is made on exactly one
fact the hint was withholding: how much is already there. A crew that came for
one search does not need the other twelve thousand, and cannot tell whether
theirs has arrived.

## What Changes

- A track with fewer than two points states its point count and nothing else.
- English says "1 pt" and "2 pts"; the Russian abbreviation does not inflect.
- The refreshing hint carries the number of projects loaded so far, once there
  is one.

## Impact

- Affected specs: `track-display`, `lizaalert-integration`
- Affected code: `src/lib/track-stats.ts`,
  `src/components/BundleLoader.svelte`, `src/lib/i18n.ts`
