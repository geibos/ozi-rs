## 1. Frontend

- [x] 1.1 `waypoint-symbols.ts` holds the table and the glyph lookup, with tests including the unknown-symbol fallback
- [x] 1.2 The picker reads it; its ten labels come from the dictionaries
- [x] 1.3 The map marker draws the glyph over the disc, and carries the symbol so a change redraws it

## 2. Evidence

- [x] 2.1 Walked on the stand: the flagged waypoint draws 🏁, the one without a symbol 📍, both 22×22

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
