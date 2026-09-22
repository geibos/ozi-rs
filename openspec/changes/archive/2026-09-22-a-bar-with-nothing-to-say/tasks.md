## 1. Frontend

- [x] 1.1 The bar collapses to its status line when no download is on screen
- [x] 1.2 The placeholder progress track is rendered only while a download is
- [x] 1.3 A height transition, so the growth reads as opening

## 2. Verification

- [x] 2.1 Measured on the stand: 32px idle; 80px with the row reserved the moment a download starts; 80px with «Скачано 2 из 4 файлов · 2/4 · 4.8 МиБ / 19.1 МиБ» while it runs; 32px again when it finishes
- [x] 2.2 `just ci` green (334 Rust, 500 frontend)
- [x] 2.3 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
