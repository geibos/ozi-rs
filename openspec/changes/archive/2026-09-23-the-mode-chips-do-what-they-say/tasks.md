# Tasks

- [x] 1.1 `src/lib/actions/modes.ts`: `workspaceMode` derived from the flags
      that have always held it, and `setInteractionMode(mode)` that leaves the
      old one and enters the new — including the track-creation routine for
      drawing.
- [x] 1.2 The chips in `WorkspaceShell.svelte` set the mode, carry
      `aria-pressed`, and show the active one.
- [x] 1.3 The Tracks tab's drawing toggle asks for the mode instead of doing
      it, so the two cannot drift.
- [x] 1.4 Asking for drawing with no active layer says so.
- [x] 1.5 Two stand lies fixed: `create_empty_track` answers with one empty
      segment as the real command does, and `get_track_detail` serves a created
      track under its own id. Without either, drawing mode could not be entered
      on the stand at all.
- [x] 1.6 Walked on the stand: draw → three `insert_track_point` → Esc leaves;
      measure, edit and view switch; pressing the active mode leaves it.
- [x] 1.7 `just ci` green.
