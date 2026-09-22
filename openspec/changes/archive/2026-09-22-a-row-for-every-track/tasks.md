## 1. Backend

- [x] 1.1 `list_track_summaries` over the layers, with a test that it includes what the map omits
- [x] 1.2 A test that the rows carry no geometry
- [x] 1.3 `list_tracks` command, registered, bindings regenerated

## 2. Frontend

- [x] 2.1 `listTracks` in `api.ts`, `trackFeaturesFromSummaries` in `track-features.ts`
- [x] 2.2 The Tracks tab reads the listing
- [x] 2.3 `trackFeaturesFromGeojson` removed with its tests; the row tests moved onto the listing

## 3. Fixtures and stand

- [x] 3.1 The sample project gains a track of one point
- [x] 3.2 The core writes `tracks-list.json`; the stand serves it rather than deriving rows from the geometry
- [x] 3.3 A test holds the two fixtures against each other

## 4. Gates

- [x] 4.1 `just ci` green
- [x] 4.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
