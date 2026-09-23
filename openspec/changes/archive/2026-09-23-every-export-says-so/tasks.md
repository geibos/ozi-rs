# Tasks

## 1. Saying that a file was written

- [x] 1.1 `src/lib/actions/export-result.ts`: one `reportExported(path)` that
      names the file, shows the path and offers «Показать», so the ten call
      sites cannot drift apart.
- [x] 1.2 `reveal_path` command in `src-tauri/src/commands/mod.rs`, registered
      in `lib.rs`, wrapped as `revealPath` in `src/lib/api.ts`.
- [x] 1.3 Wire `reportExported` into all ten export call sites across
      `TracksTab`, `WaypointsTab`, `TrackInspector`, `WaypointInspector` and
      `CommandPalette`.
- [x] 1.4 `src/test/every-export-says-so.test.ts`: a guard that every component
      calling an export command also reports it.
- [x] 1.5 Walk CJ-6 on the stand: the toast reads
      `20260708_Veter2.plt записан / /searches/10-Tracks/20260708_Veter2.plt /
      Показать`, and «Показать» produces exactly one `reveal_path` call.

## 2. A stand that behaves as the application behaves

- [x] 2.1 `EMITS_STATE_CHANGED` read out of `src-tauri/src/commands/mod.rs`
      rather than restated: fifty-four commands, not two.
- [x] 2.2 `src/test/stand-emits-what-the-app-emits.test.ts` keeps the two lists
      in step.
- [x] 2.3 An unsaved-work model on the stand: everything changes the project
      unless it is a read, an export, a catalogue call or a save.
- [x] 2.4 `src/test/stand-dirty-model.test.ts` pins the partition, so a new
      command cannot stay unclassified.
- [x] 2.5 `standRequestClose()` and `destroy()` on the stand's window, and
      `__standWindow` / `__standDialog` on `window` so a walk reaches the copy
      of the module the application loaded rather than a second one.
- [x] 2.6 Walk CJ-7 on the stand: clean → edit → «Есть несохранённые
      изменения» → Cmd+S → «Сохранено»; a close with unsaved work holds the
      window and asks, cancelling keeps it open, confirming destroys it.

## 3. Gates

- [x] 3.1 `just ci` green: 375 Rust, 584 frontend.
- [x] 3.2 `openspec validate every-export-says-so --strict`.
