## Why

CJ-6 — handing a file to a group — is the last thing that happens before a
crew walks out of the door, and four of the five ways out of the application
said nothing when they succeeded.

**An export that says nothing looks exactly like one that did not run.** The
day export (`export_all_tracks_gpx`) had a success toast from the start. The
single-file exports — one track to PLT, one layer to GPX, the marks to GPX,
the marks to WPT — wrote the file and returned in silence. The file appeared
in a directory the operator had chosen half a second earlier, in a picker that
closed itself, and nothing on screen changed. At four in the morning, sending
a crew out, "did it write?" is then a question that can only be answered by
leaving the application and going to look.

**And the folder is the next thing they want.** The file is being handed over —
attached to a message, copied to a stick, dropped in a chat. OziExplorer's
export dialog leaves the operator in the folder they wrote to; this left them
nowhere.

Found by walking CJ-6 on the stand on 2026-09-23.

Two things about the stand came out of the same walk, and are here because
they are the reason this was not found a month ago.

**The stand told the interface nothing had happened.** The application emits
`state-changed` after fifty-four commands; the stand emitted after two. Every
screen that waits for that event rather than reloading itself — the
unsaved-changes indicator above all — sat still on the stand and moved in the
packaged application. A stand that invents defects is worse than no stand: it
teaches whoever walked it to distrust what they see.

**And CJ-7 could not be walked at all.** The stand served a fixed
`project_dirty: false`, so the indicator could never turn; the window it
answered with registered the close guard and never fired it, and had no
`destroy()` for the guard to call when the operator confirms quitting. The
journey that is entirely about whether a night's work survives was the one
journey with no way to look at it.

## What Changes

- Every successful export tells the operator, by name, that the file was
  written, and offers to show it in the file manager.
- A new `reveal_path` command opens a file manager on any written file, the
  way `reveal_bundle` does for a bundle.
- The stand fires `state-changed` after the same commands the application
  does, keeps a model of whether the project has unsaved work, and can be
  asked to close its window.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/lib/actions/export-result.ts` (new), `src/lib/api.ts`,
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, the five components
  that export (`TracksTab`, `WaypointsTab`, `TrackInspector`,
  `WaypointInspector`, `CommandPalette`), `src/test/stand/*`
