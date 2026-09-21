## Why

The catalogue is thirteen thousand rows and the list is virtualized, so only
the rows on screen exist in the DOM. There was no tab order over it and there
never can be one: a crew on a laptop in a field HQ had to find their search
with the trackpad, and a screen reader saw a handful of buttons out of thirteen
thousand.

Typing part of the name already narrows the list. What was missing was the rest
of that gesture — move down into the results and open one without reaching for
the pointer.

## What Changes

- The list is a listbox that holds focus itself and points at the current row
  with `aria-activedescendant`, which is the pattern a virtualized list needs:
  moving DOM focus row by row would fight the windowing, and the rows a
  position may land on are mostly not rendered.
- The position is an index into the filtered array, not into what is rendered.
  Arrow keys move it, Page Up/Down by a screenful, Home and End to the ends;
  it does not walk off either end, and the list scrolls to keep it in view.
- Enter opens the row it is on. From the search box, Down walks into the list
  and Enter opens the first match, so the whole gesture is: type, arrow, Enter.
- Narrowing the list clears the position, because it no longer means anything.
- The position carries its own outline: it is not DOM focus, and it is not the
  selected row either.

## Impact

- Affected specs: `lizaalert-integration`
- Affected code: `src/components/BundleLoader.svelte`, `src/lib/i18n.ts`
