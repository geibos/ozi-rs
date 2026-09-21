## 1. Backend

- [x] 1.1 A complete walk replaces the project list; a stopped one does not — a test each
- [x] 1.2 `catalogue-refresh-started` after the cached chunk, before the walk
- [x] 1.3 `catalogue-refresh-finished` carrying whether the walk completed

## 2. Frontend

- [x] 2.1 The store collects the slugs a walk sends between the boundaries
- [x] 2.2 A complete walk prunes to them; a stopped walk prunes nothing; no walk in progress prunes nothing
- [x] 2.3 A chunk that arrived before the window does not count — five tests

## 3. The stand

- [x] 3.1 `catalogue.json` written by the core, since the catalogue is its own stream now
- [x] 3.2 The stand plays cached chunk, started, walk chunk, finished
- [x] 3.3 Walked on the stand: three rows, "3 из 3", not emptied by the complete walk

## 4. Gates

- [x] 4.1 `just ci` green
- [ ] 4.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
