## 1. Backend

- [x] 1.1 `set_all_tracks_visible(visible: bool)` in the application layer, applied across every track layer as one style mutation, with tests
- [x] 1.2 `show_only_track(layer_id, track_id)` in the application layer, with tests including the hidden-track case
- [x] 1.3 Expose both as specta commands and regenerate `bindings.ts`
- [x] 1.4 Name import-created layers after the source file, with a test

## 2. Frontend

- [x] 2.1 Show-all / hide-all controls in the Tracks tab header, localized
- [x] 2.2 "Only this one" action in the row menu, localized
- [x] 2.3 API wrappers over the generated bindings

## 3. Verification

- [x] 3.1 `just ci` green
- [x] 3.2 Screenshots before/after in `docs/progress/`
