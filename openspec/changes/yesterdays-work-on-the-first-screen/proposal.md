## Why

A saved `.ozp` could be opened from exactly one place: the command palette. The
launch screen offers "Открыть локальный бандл…" and "Папка для бандлов…" and
nothing else, so a crew arriving in the morning with yesterday's work saw a
catalogue of thirteen thousand LizaAlert searches and no way back to their own
project — unless they knew ⌘K existed.

The same reasoning already moved the language switch out of the palette and
into the status bar: "a Russian-speaking crew had to know the palette exists to
get a Russian interface".

The word "проект" also means two different things in this application, and both
buttons said the same thing. The Maps tab's "Открыть проект…" opens the
LizaAlert catalogue; the palette's "Открыть проект…" opens a `.ozp` file. The
loader's own list is headed "Каталог поисков", so the catalogue already has a
better word for itself.

## What Changes

- The launch screen offers "Открыть сохранённый проект…" and, under it, the
  three most recent projects as one click each.
- The Maps tab's button says "Выбрать поиск…", which is what it does.
- The dialog, the remembering, the framing and the dropping of a stale path
  live in one `openProjectFile` action. The palette's two handlers become
  calls to it.

## Capabilities

### Modified Capabilities
- `project-persistence`: opening a saved project is reachable without the
  command palette.

## Impact

- **Frontend**: `$lib/actions/project.ts` (`openProjectFile`),
  `BundleLoader.svelte`, `CommandPalette.svelte`, i18n (one key added, one
  reworded).
- **Backend**: none.
- **Risk**: low. The palette keeps both of its entries and now shares their
  implementation.
