## 1. Frontend

- [x] 1.1 The guard, written first, failing with the list it found
- [x] 1.2 Sixteen messages translated in both dictionaries
- [x] 1.3 The guard ignores comments and messages built from translations, and has tests for both

## 2. Verification

- [x] 2.1 On the stand: the keys resolve in Russian, and «Переместить на карте» shows «Нажмите на карту, чтобы перенести точку / Точку можно и перетащить на карте»
- [x] 2.2 `just ci` green (334 Rust, 499 frontend)
- [ ] 2.3 `just smoke` green (blocked: the Mac2 driver cannot enable automation mode — see `docs/STATE.md`)
