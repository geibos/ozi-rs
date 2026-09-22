## 1. Frontend

- [x] 1.1 `highlightTrack` and the casing layer, with tests written first
- [x] 1.2 `MapView` drives it from the selection, reading the store before the guard
- [x] 1.3 The four other effects with the same shape are turned around
- [x] 1.4 A guard fails on an effect that returns before reading anything reactive, comments stripped

## 2. Verification

- [x] 2.1 On the stand, twelve imported routes: clicking «20260921_СОКОЛ7 1» puts a white casing under the violet line and leaves the colour on top
- [x] 2.2 The guard bites: restoring the old order fails it, naming `MapView.svelte:274`
- [x] 2.3 `just ci` green (338 Rust, 508 frontend)
- [x] 2.4 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
