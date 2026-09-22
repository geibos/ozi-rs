## 1. Frontend

- [x] 1.1 The guard over markup text, written first, failing with the list it found
- [x] 1.2 Thirteen strings translated in both dictionaries
- [x] 1.3 `CRS` and `fps` allowed by name, with the reason

## 2. Stand

- [x] 2.1 `get_simplified_preview` answers with the DTO's shape

## 3. Verification

- [x] 3.1 On the stand: the row menu reads «Показать только этот / Экспорт GPX / Экспорт PLT / Толщина линии / Упростить… / Удалить», and the dialog «Упрощение трека · Допуск: 10 м · Предпросмотр · Было: 5 → станет: 3 · Отмена · Упростить», with no console errors
- [x] 3.2 `just ci` green (334 Rust, 500 frontend)
- [ ] 3.3 `just smoke` green (blocked: the Mac2 driver cannot enable automation mode — see `docs/STATE.md`)
