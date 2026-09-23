## Why

The strip at the top of the map names the four things the application can be
doing — Просмотр, Рисование, Правка, Измерение — and it was inert. Not greyed
out in a way that reads as "later": four buttons carrying the four mode names,
marked `aria-disabled`, doing nothing at all when pressed. The group's own
label said so, to anyone reading the source: `Mode (inert placeholder)`.

Every one of those modes existed and worked. Drawing was reachable from the
Tracks tab, editing from the segments table, measuring from the command
palette. The control that says which mode you are in — the most visible one in
the workspace, sitting over the map — was a placeholder from a redesign that
never came back to it. CJ-5 has recorded it as «mode-чипы Draw/Edit/Measure —
муляжи» since July.

Wiring them is not four `onclick` handlers, though. Entering drawing mode is
not a flag: a track has to be created in the active layer, its first segment
found, the counters reset. That routine lived inside the Tracks tab, and
copying it into the shell is exactly how the import surfaces came to accept
different file types from each other. So the mode became a function that every
surface asks for, which is the smallest honest version of the `interactionMode`
the rebuild plan describes.

## What Changes

- The mode chips set the mode, show which one is current, and pressing the
  current one leaves it.
- `$lib/actions/modes.ts` owns what entering and leaving a mode means; the
  Tracks tab's drawing toggle now asks it rather than doing it.
- Asking for drawing with no track layer says so instead of lighting a chip
  over a map that ignores the clicks.

Two things the stand had wrong came out of walking it, and are fixed here
because without them the journey cannot be walked at all: `create_empty_track`
answered with a track that had no segments, where the real command gives it
one, and `get_track_detail` served a created track's detail only under the
fixture track's id.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/lib/actions/modes.ts` (new),
  `src/components/WorkspaceShell.svelte`,
  `src/components/library/TracksTab.svelte`, `src/lib/i18n.ts`,
  `src/test/stand/tauri-core.ts`
