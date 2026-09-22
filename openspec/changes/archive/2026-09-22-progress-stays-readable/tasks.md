## 1. Frontend

- [x] 1.1 `toastOffset(panelHeight)` in `src/lib/toast-offset.ts`, with tests written first
- [x] 1.2 `DownloadPopup` measures itself into a `downloadPopupHeight` store, zeroed when no download is on screen
- [x] 1.3 The layout passes the derived offset to `Toaster`

## 2. Verification

- [x] 2.1 Measured on the stand: panel 692..1012 × 522..628, toast lifted to 437..511, gap 11 px, no overlap; toast returns to the corner when the download finishes
- [x] 2.2 `just ci` green
- [x] 2.3 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
