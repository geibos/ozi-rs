## Why

Since 2026-09-23 the workspace route treats a project file with no map as a
workspace — a `.ozp` from another headquarters opens without a map, because
this machine may never have downloaded that ground, and throwing the operator
back to the catalogue left the project loaded and invisible. Only one of the
two routes learned that. The launcher's start-up redirect still forwards to the
workspace only when a map is active, so after a restart with such a project
the operator lands in the catalogue with their work hidden behind it.

The workspace route also decided before the application state had arrived:
on start the state is empty for a moment, its guard sent the operator to the
launcher, and the launcher sent them back only if there was a map. The
screenshot matrix found it, trying to photograph the Maps tab's empty state.

## What Changes

- One rule for both routes: the workspace is worth opening when a map is
  active or a project file is open.
- The launcher's start-up redirect uses it, so a restored project with no map
  opens in the workspace.
- The workspace route waits for the application state before deciding, and
  ignores the empty moment before it arrives.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/lib/workspace-route.ts` (new), `src/routes/+page.svelte`,
  `src/routes/project/+page.svelte`
