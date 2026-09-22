## 1. Backend

- [x] 1.1 `export_day_to_gpx_file`, with tests written first
- [x] 1.2 `export_all_tracks_gpx` collects both and reports both counts
- [x] 1.3 A project of marks alone exports; only one with neither is refused
- [x] 1.4 The tracks-only file writer is gone rather than left unused

## 2. Frontend

- [x] 2.1 The toast says how many of each
- [x] 2.2 The button's tooltip stops promising tracks only

## 3. Stand

- [x] 3.1 `standAnswerDialogsWith` so a dialog flow can be looked at
- [x] 3.2 `export_all_tracks_gpx` answers with counts

## 4. Verification

- [x] 4.1 On the stand, with the dialog answered: «Выгружено: треков 3, точек 3»
- [x] 4.2 `just ci` green (332 Rust, 493 frontend)
- [x] 4.3 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
