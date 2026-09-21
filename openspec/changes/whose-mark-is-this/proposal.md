## Why

ADR-0020 declares waypoint colour in scope; waypoints had none. Tracks have had
per-track colour from the beginning.

The symbol says *what* a mark is — the task point, what was found, where the
danger is. The colour says *whose* it is: group A's marks against group B's, on
one map, in a field HQ, at night. Symbols alone cannot carry that, and a search
collects marks from every group working it.

## What Changes

- `Waypoint` carries an optional RGBA colour, `#[serde(default)]` so a `.ozp`
  written before this loads with every waypoint uncoloured — which is what it
  meant.
- `None` is not a colour. An uncoloured waypoint follows whatever the map draws
  waypoints with, so changing that default later moves every uncoloured
  waypoint with it. Clearing returns a waypoint to the default rather than to a
  colour that happens to look like it.
- `SetWaypointColor` is an undoable command, the same shape as
  `SetWaypointSymbol`, with the old colour in the delta.
- The Waypoint Inspector offers a swatch beside the symbol picker, and a
  "default colour" control that appears only when there is a colour to clear.
- The map marker takes its background from it.

Found beside this and fixed: the inspector's symbol failure was an English
literal in a `toast.error`. The silent-failure guard let it through because it
does toast; the label guard does not look inside `toast` calls.

## Impact

- Affected specs: `waypoints`
- Affected code: `src-tauri/src/domain/waypoint.rs`,
  `src-tauri/src/domain/project.rs`, `src-tauri/src/application/commands.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`, `src-tauri/src/fixtures.rs`, `src/lib/bindings.ts`,
  `src/lib/api.ts`, `src/lib/i18n.ts`,
  `src/components/inspector/WaypointInspector.svelte`,
  `src/components/MapView.svelte`, `src/test/stand/tauri-core.ts`
