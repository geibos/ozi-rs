## 1. Statistics

- [x] 1.1 `track-motion.ts`: time on the move over a window, ascent and descent past a threshold
- [x] 1.2 Tests: rest stop, slow combing, standing still, no timestamps, flat noise, a hill, no elevations, a different threshold

## 2. Settings

- [x] 2.1 `settings.ts`: the thresholds as a persisted store, defaults 25 m / 2 min / 5 m, a value that is not a positive number ignored
- [x] 2.2 `Settings.svelte`: appearance (theme with the Catppuccin pack, language) and track statistics
- [x] 2.3 Reached from a gear in the top bar, the palette and ⌘,
- [x] 2.4 `ThemePicker.svelte` restyled through Tailwind and mounted on the settings screen
- [x] 2.5 Tests: defaults, persistence, a bad value, the screen writes what is typed

## 3. The inspector

- [x] 3.1 Time on the move beside the span; ascent and descent; nothing for a track without times or heights

## 4. Gates

- [x] 4.1 `just ci` green (2026-10-02: 407 Rust, 714 vitest)
- [x] 4.2 Walked on the stand 2026-10-02: the gear opens the screen with theme, language and the three thresholds; Esc closes it and ⌘, opens it again; the inspector shows «Длительность 5ч 1мин», «В движении 3мин», «Набор / сброс +0 м / −0 м» for the fixture, and the climb went to «+3 м» when the threshold was set to 1 m, persisted as `ozi:motion-settings`. `docs/progress/2026-10-02-settings-and-time-on-the-move/`
- [ ] 4.3 `just smoke` green on a bundle built from this tree — 2026-10-02: on a bundle built from `fe06e6a`, `smoke_cj5_draw_track` and `smoke_report_capture_moment` green; `smoke_cj3_layer_management` failed twice on a machine in use (its tab click landed on an inactive window) and was not run a third time. Owed: one run with nobody at the keyboard.
