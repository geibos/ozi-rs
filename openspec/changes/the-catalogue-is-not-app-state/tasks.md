## 1. Backend

- [x] 1.1 `projects` off `AppStateDto`, with a test that a 500-project catalogue does not reach the snapshot and it stays under 4 KiB
- [x] 1.2 `app_state_dto` loses `cached_slugs`; `get_app_state` stops walking the bundles directory
- [x] 1.3 The accessor the catalogue used is gone

## 2. Frontend

- [x] 2.1 `syncProjectsFromAppState` and its call site removed
- [x] 2.2 `LizaProjectSummaryDto` hand-written in `types.ts`, with why it is not generated
- [x] 2.3 The test that guarded the old seeding path now pins the two sources that remain

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
