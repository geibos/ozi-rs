## 1. Backend

- [x] 1.1 `apply_preview_loaded(slug, result)` drops a result that is no longer the selected project, with a test
- [x] 1.2 The same method leaves `busy` untouched, with a test
- [x] 1.3 A stale preview failure is not reported, with a test
- [x] 1.4 `preview_project` calls it with the slug it requested

## 2. Frontend

- [x] 2.1 The pending hint is keyed on the slug, not the display name
- [x] 2.2 The timer marks the wait slow instead of clearing it; new key in both dictionaries
- [x] 2.3 The stand answers `preview_project` by moving the previewed slug, so the round trip closes

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
