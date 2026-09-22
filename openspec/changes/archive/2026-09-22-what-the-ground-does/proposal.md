## Why

The track inspector carried a card headed "Высота" whose entire content was
"График высоты — будет в следующем изменении". `PointDetailDto` has carried
`elevation` since the first import path was written, so the data was there the
whole time and the card was a promise shown to a crew instead of an answer.

What the ground does along a route is not decoration for a search: a leg that
climbs out of a river valley takes an hour where the same distance on the flat
takes twenty minutes, and a crew reading somebody else's recording has no other
way to learn that from this application.

## What Changes

- The card draws the elevation against distance along the track, with the
  range in metres beside the heading.
- A recording that carries no elevation, or only one reading, says so instead
  of drawing a line through nothing.
- The profile is built from the detail the statistics card already loads, so
  the chart costs no round trip.
- Ascent and descent totals are deliberately absent: summing every rise in a
  GPS track adds up its own noise, and the threshold that fixes that is a
  decision about the data, recorded in `docs/backlog.md` beside the
  moving-time threshold.

## Capabilities

### Added Capabilities
- `track-display`: the track's elevation is shown against distance.

## Impact

- **Frontend**: `src/lib/elevation-profile.ts` (new),
  `inspector/TrackInspector.svelte`, three i18n keys (one removed).
- **Backend**: none — the elevation was already in the DTO.
- **Risk**: low. The card showed nothing usable before.
