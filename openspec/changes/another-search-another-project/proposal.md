## Why

A project is one search — the owner said so on 2026-10-01, and «Новый поиск»
in the palette has started one since 2026-09-23. But the palette is not where a
crew moves to the next search. They open the catalogue, pick the next search
and open its map, and the previous search's routes stay in the project and are
drawn over a district they have nothing to do with. Saving then writes two
operations into one file.

The owner's answer to "what should choosing another search do": start a new
project, asking about saving first.

## What Changes

- Opening a map of a search other than the one the current map belongs to,
  while the project holds work — tracks, unsaved changes, or a file on disk —
  starts a new project before the map opens.
- Unsaved work is asked about first, with the three answers the close guard
  gives: save and go on, go on without saving, or stay. Staying opens nothing.
  A save that fails or is cancelled is a stay.
- Work already saved to disk needs no question: the file holds it.
- A toast names the search the new project belongs to.
- Opening another map of the same search, or the first map with an empty
  project, does none of this.

## Impact

- Affected specs: `project-persistence`
- Affected code: `src/lib/actions/open-map.ts`, `src/lib/actions/project.ts`,
  `src/lib/search-switch.ts` (new), `src/lib/stores.ts`,
  `src/components/CloseGuard.svelte`, `src/lib/i18n.ts`
