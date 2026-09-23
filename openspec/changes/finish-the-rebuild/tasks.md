# Tasks

## 1. The screenshot matrix

- [ ] 1.1 `src/test/stand/screens.ts`: a registry of screens — bundle loader,
      workspace, library tabs, both inspectors, command palette — each
      declaring its fixture and transport for the five states.
- [ ] 1.2 `src/test/stand/shots.ts`: render screen × state × locale × theme at a
      fixed viewport with bundled fonts and animations off; `--compare` against
      the baseline with a per-pixel tolerance and diff images; `--update`
      rewrites it; `--slice` writes into `docs/progress/<date>-<slice>/`.
- [ ] 1.3 `just shots`, and `just ci` gains `just shots --compare`.
- [ ] 1.4 Commit the baseline of the current interface as the recorded "before".
- [ ] 1.5 CI runs the fixtures and the matrix, uploading the images and diffs;
      documented in `docs/ci.md`.

## 2. The view-model layer

- [ ] 2.1 `src/lib/vm/`: `workspace`, `library`, `catalog`, `selection`, `map`,
      building on `$lib/actions/modes.ts`, which already owns the modes.
      Part of it is deciding who owns the map's geometry types: `get_tracks_geojson`
      returns `serde_json::Value` and `api.ts` casts it to `GeoJSON.FeatureCollection`,
      and a specta mirror would duplicate that type and still need a cast.
- [ ] 2.2 The legacy flags derived from the mode rather than beside it, marked
      deprecated, then deleted when the last reader moves.
- [ ] 2.3 `tracks-layer.ts` becomes `attach(map, vm) → detach()`, with a
      fake-map test for the lifecycle.
- [ ] 2.4 Fixture-driven tests for every module.
- [ ] 2.5 Migrate the workspace status bar first; screenshots before and after.
- [ ] 2.6 An ESLint rule keeping migrated components off `appState` and the
      legacy flags.
- [ ] 2.7 `projects` leaves `AppStateDto` once `vm/catalog` is its only
      consumer; `syncProjectsFromAppState` and the stale-while-revalidate guard
      go with it.

## 3. A journey per Customer Journey

- [ ] 3.1 A file dialog the Mac2 driver can answer, which unblocks CJ-3's
      import half and CJ-6 entirely.
- [ ] 3.2 A fixture `.ozp` loaded at launch, which unblocks CJ-4 and CJ-8.
- [ ] 3.3 A fake catalogue server for CJ-1, and a staged cached bundle with the
      link down for CJ-2.
- [ ] 3.4 CJ-7's close guard: dismissing a native quit dialog ends the Mac2
      session, so this needs a different approach or stays a stand-only walk.
- [ ] 3.5 Crop `appium_screenshot` to the application window through
      `CGWindowListCopyWindowInfo`, with the crop maths unit-tested.
