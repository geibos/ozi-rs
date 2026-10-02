## 1. Deciding

- [x] 1.1 `search-switch.ts`: whether a map opens another search, and whether the project holds work
- [x] 1.2 Tests on both, from the state the frontend actually has

## 2. Asking and starting

- [x] 2.1 The close-guard dialog asks for either purpose, with labels for each
- [x] 2.2 Saving reports whether it happened, so a cancelled save is a stay
- [x] 2.3 `makeWayForSearch`: ask when dirty, save or discard or stay, start the new project, toast the search's name
- [x] 2.4 `openMapShowingDownload` opens nothing when the operator stays
- [x] 2.5 Tests: save-and-continue, discard, stay, cancelled save, saved project, same search, empty project

## 3. Gates

- [x] 3.1 `just ci` green (2026-10-02: 407 Rust, 694 vitest)
- [x] 3.2 Walked on the stand 2026-10-02: Lavrovo project, Sagra chosen, its map opened. Saved project: no question, tracks gone, toast «Новый проект для поиска «2026 07 14 Sagra»». After an edit: the question in the next-search wording; «Отмена» stayed in the catalogue with nothing opened; «Продолжить без сохранения» opened the map over an empty Tracks tab. The stand first had to be made to tell searches apart — it moved the previewed slug and kept the fixture's name — and the dialog had to be widened: at 312px its three answers ran past its background. `docs/progress/2026-10-02-another-search/`
- [ ] 3.3 `just smoke` green on a bundle built from this tree — 2026-10-02: on a bundle built from `fe06e6a`, `smoke_cj5_draw_track` and `smoke_report_capture_moment` green; `smoke_cj3_layer_management` failed twice on a machine in use (its tab click landed on an inactive window) and was not run a third time. Owed: one run with nobody at the keyboard.
