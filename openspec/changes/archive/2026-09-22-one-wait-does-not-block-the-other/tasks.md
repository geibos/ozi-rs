## 1. Backend

- [x] 1.1 Split `busy` into `listing_busy` and `bundle_busy`, with tests written first
- [x] 1.2 Each flag released by its own completion
- [x] 1.3 `AppStateDto` carries both; bindings regenerated

## 2. Frontend

- [x] 2.1 `listingBusy` / `bundleBusy` stores, `busy` kept as "either"
- [x] 2.2 The refresh control waits on the walk, the download control on the bundle
- [x] 2.3 The disabled download button states the bundle reason, not the listing one

## 3. Verification

- [x] 3.1 On the stand, driving both flags: walking → refresh disabled, download offered; downloading → refresh free, download disabled with the bundle reason; idle → both free
- [x] 3.2 `just ci` green
- [x] 3.3 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
