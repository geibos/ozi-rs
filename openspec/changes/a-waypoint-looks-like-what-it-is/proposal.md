## Why

Every waypoint on the map was the same yellow dot. The symbol a crew chooses —
flag, camp, danger, water, meeting point — was stored, listed in the Waypoints
tab, carried into GPX, and the one place it did not appear was the place they
look at.

A search area collects the task point, what was found, where the danger is and
every group's marks. Ten identical dots do not distinguish any of that.

## What Changes

- The symbol table moves to `src/lib/waypoint-symbols.ts`, so the picker a crew
  chooses from and the marker on the map read the same list. There were two
  places that had to agree and only one of them knew anything.
- A marker draws its symbol's glyph, keeping the disc behind it: a glyph alone
  over aerial imagery is unreadable, and a marker has to be findable before it
  can be identified. 22px rather than 14 to fit one.
- The applied-marker record carries the symbol, so the reconciler redraws when
  a symbol changes rather than only when a waypoint moves.
- A symbol this build does not know — from a GPX another tool wrote — falls
  back to the default pin. The waypoint is still there and must still be
  findable.
- The picker's ten labels were English literals inside its table; they come
  from the dictionaries now.

## Impact

- Affected specs: `waypoints`
- Affected code: `src/lib/waypoint-symbols.ts` (new), `src/lib/i18n.ts`,
  `src/components/SymbolPicker.svelte`, `src/components/MapView.svelte`
