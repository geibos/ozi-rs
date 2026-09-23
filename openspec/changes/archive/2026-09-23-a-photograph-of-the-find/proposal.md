## Why

A find is photographed, and the photograph is what the next shift actually
looks at: the jacket in the grass, the footprint, the stretch of fence where
the trail stops. The mark says where; the note says what; the photograph is the
thing itself, and it lived in somebody's phone with no way to attach it to the
place it belongs.

OziExplorer carries a link on each waypoint for this. It is the last of the
four things `docs/backlog.md` recorded as present there and missing here.

**Paths, not bytes.** The backlog entry said this would need files inside the
`.ozp` and so a change to the format two headquarters exchange. It does not: a
`.ozp` is JSON that people read and diff, and a photograph inside it turns a
readable file into a megabyte of base64 that no editor opens. The file lives
beside the project — which is how everything else here works, a `.map` beside
its picture, a bundle in its folder — and sending the work means sending the
folder, which is what a crew already does.

A list rather than OziExplorer's single link, because a find is photographed
from three sides.

## What Changes

- A mark carries a list of files. Attaching one chooses a file already on the
  machine; nothing is copied anywhere.
- The list is replaced whole, as one undoable step: the undo delta needs the
  previous list either way, and a pair of add/remove commands can get out of
  step with itself.
- Blank entries and repeats are dropped — the same photograph attached twice is
  a mistake, not a decision.
- Each file can be shown in the file manager or detached.
- A project saved before this field existed loads with nothing attached.

The files do not travel in `.wpt` or GPX: neither format has a place for them,
and the note (field 11) is what does reach a headquarters running OziExplorer.

## Impact

- Affected specs: `waypoints`
- Affected code: `src-tauri/src/domain/waypoint.rs`,
  `src-tauri/src/domain/project.rs`, `src-tauri/src/application/commands.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`, `src/lib/api.ts`,
  `src/components/inspector/WaypointInspector.svelte`, `src/lib/i18n.ts`
