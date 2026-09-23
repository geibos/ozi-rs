## 1. One dispatch

- [x] 1.1 `import-paths.ts`: routing by extension, failures collected not thrown
- [x] 1.2 The Tracks tab picker uses it, and takes its filter list from it
- [x] 1.3 Behavioural tests replace the two source-text assertions

## 2. The drop

- [x] 2.1 The window's drag-drop hook imports what lands on it
- [x] 2.2 A webview without the hook does not fail the launch — the picker is still there
- [x] 2.3 The stand answers the hook and can play a drop

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 Walked on the stand 2026-09-23: dropping `ЛИСА17.gpx` and `ШТАБ.wpt` on the window put the track in the list and took the map from two markers to four, with «Импортировано файлов: 2 из 2». `docs/progress/2026-09-23-drop-and-version/dropped-files.png`
