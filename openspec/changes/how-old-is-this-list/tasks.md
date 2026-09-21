## 1. Frontend

- [x] 1.1 `formatCatalogueAge` in `src/lib/catalogue-age.ts`, with tests written first
- [x] 1.2 `catalogueWrittenAt` seeded from the cache and updated on every write
- [x] 1.3 The loader shows it beside the count and inside the offline notice

## 2. Verification

- [x] 2.1 On the stand: «Только скачанные · 3 из 3 · от 21.09, 09:12», and offline «Нет связи — сохранённый список от 21.09, 09:12»; with no timestamp the age disappears and the notice falls back to its old wording
- [x] 2.2 `just ci` green
- [ ] 2.3 `just smoke` green (blocked: the Mac2 driver cannot enable automation mode — see `docs/STATE.md`)
