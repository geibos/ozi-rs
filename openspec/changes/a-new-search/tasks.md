## 1. Backend

- [x] 1.1 `new_project`: empty document, default layers, no path, empty history
- [x] 1.2 Test: the old tracks are gone and undo cannot bring them back
- [x] 1.3 Test: the active map survives
- [x] 1.4 Command registered, bindings regenerated

## 2. Reaching the operator

- [x] 2.1 `api.ts` wrapper and a project action that asks before discarding
- [x] 2.2 Command palette entry, both dictionaries
- [x] 2.3 Active layer ids follow the new project's defaults

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 Walked on the stand 2026-09-23: the Tracks tab went from three tracks and two markers to none, the active layer fell back to the default, the toast read «Начат новый поиск» — and the Maps tab still listed `2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb`, which is the requirement that the ground survives the work. The stand answered `new_project` by clearing only this session's overlays at first, so the fixture's own tracks stayed in the list while the toast said otherwise; it empties the document now, as the backend does. `docs/progress/2026-09-23-new-search/after-new-search.png`
