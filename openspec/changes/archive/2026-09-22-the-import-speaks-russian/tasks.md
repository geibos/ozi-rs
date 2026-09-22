## 1. Backend

- [x] 1.1 `ImportReportDto` with counts and skipped names, with tests written first
- [x] 1.2 `import_tracks_directory` returns it instead of an English sentence

## 2. Frontend

- [x] 2.1 The Tracks tab words the result from the counts
- [x] 2.2 A partly-read folder is a success with a caveat, not a failure

## 3. Stand

- [x] 3.1 Answers for the import commands, with the imported rows appearing
- [x] 3.2 The directory answer reports one unreadable file, so the caveat branch is visible

## 4. Verification

- [x] 4.1 On the stand: «Импортировано: треков 3, точек 2, из файлов 4 — Не прочитано 1: ЛИСА17.plt». The caveat branch is the one measured; the clean branch builds the same summary string and differs only in which toast it calls
- [x] 4.2 `just ci` green (334 Rust, 494 frontend)
- [x] 4.3 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
