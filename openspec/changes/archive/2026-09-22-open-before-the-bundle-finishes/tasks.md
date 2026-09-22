## 1. Prove the capability

- [x] 1.1 Rust test: a map whose file lands mid-download opens from disk rather than starting a download

## 2. Say so

- [x] 2.1 `readyMapName` matches a landed file against the project's maps, with tests
- [x] 2.2 The layout announces the first openable map per download, localized

## 3. Verification

- [x] 3.1 `just ci` green
- [x] 3.2 Seen on screen (2026-09-22, stand, `docs/progress/2026-09-22-verification/map-ready-announcement.png`): the announcement reads «2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb готова / Можно открывать — остальное докачается само.» 4.3 s into the download, while the 185 MiB satellite layer is still pending. The stand replays the events the backend emits, so this proves the screen given those events; that the backend emits them when a file lands is task 1.1's Rust test
