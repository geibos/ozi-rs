## Why

The guard written yesterday reads `aria-label`, `title` and `placeholder`. The
one written this morning reads a notification's message. Neither reads the
words *between* the tags — and that is where the actions menu on every track
row was living, in English, including its destructive `Delete`, along with the
whole simplify dialog and two empty states.

That is the same lesson twice in two days: a guard on one shape says nothing
about another. The symbol picker's label table escaped the attribute guard the
same way.

## What Changes

- Thirteen strings move into the dictionary: the track row's actions menu
  (line width, simplify, delete), the simplify dialog end to end, the map's
  "drawing" badge, the inspector's empty state and the points table's.
- A guard fails on any text node in a component's markup carrying three Latin
  letters in a row. It reads only the markup — `<style>` and comments are
  blanked, `{…}` expressions are blanked — so it says nothing about how text is
  built, only that it is not typed in.
- `CRS` and `fps` are allowed by name: a projection row's heading and the F3
  developer overlay's unit, written the same in both languages.
- The drawing badge counts in `тчк` rather than `point/points`. Russian needs
  three plural forms for a count and the tracks list already solved it that
  way.
- The stand's `get_simplified_preview` answered `{points, removed}` while the
  DTO is `{original_count, simplified_count, segments}`, so opening the dialog
  threw inside `MapView` and showed two empty numbers. Fixed to the real shape,
  derived from the track fixture.

## Capabilities

### Modified Capabilities
- `ui-shell`: text between the tags comes from the dictionary too.

## Impact

- **Frontend**: thirteen i18n keys, five components,
  `src/test/no-untranslated-labels.test.ts`.
- **Stand**: `get_simplified_preview` returns the DTO's shape.
- **Backend**: none.
