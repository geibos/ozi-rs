## 1. Frontend

- [x] 1.1 `openMapShowingDownload` in `src/lib/actions/open-map.ts`, with tests written first
- [x] 1.2 All three callers go through it
- [x] 1.3 The stand plays a single-map download for a map it reports as not downloaded

## 2. Verification

- [x] 2.1 On the stand: opening a not-downloaded map from the palette's recent files shows the progress panel for the download's lifetime (18 sampled frames over ~2.7 s) and clears it on `download-finished`
- [x] 2.2 `just ci` green
- [ ] 2.3 `just smoke` green (blocked: the Mac2 driver cannot enable automation mode — see `docs/STATE.md`)
