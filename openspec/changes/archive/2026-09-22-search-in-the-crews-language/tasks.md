## 1. Frontend

- [x] 1.1 `transliteratedPattern` in `src/lib/translit.ts`, with tests written first
- [x] 1.2 `filterProjects` uses it for a Cyrillic query and keeps substring matching otherwise
- [x] 1.3 `paletteProjects` uses it too, and still stops at its limit

## 2. Verification

- [x] 2.1 On the stand: `Шувалово` → `2026 09 20 Schuvalovo` (1 of 3), `Лаврово` → `2026 07 08 Lavrovo`, `Мурманск` → 0 of 3, `Sagra` → `2026 07 14 Sagra`
- [x] 2.2 `just ci` green
- [x] 2.3 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
