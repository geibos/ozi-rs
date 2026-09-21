## Why

The catalogue names every search in latin transliteration —
`2026-07-08_Lavrovo`, `2026-09-20_Schuvalovo`, `2026-07-14_Sagra` — and the
filter box matched the query as a literal substring. A crew that types
`Лаврово`, which is the name they were given over the radio and the name of the
place they are standing in, got an empty list.

That box is the only way into a bundle, and the list behind it is about
thirteen thousand rows. "Type the name in English" is not an instruction a
field crew should have to remember, and the transliteration is not one they can
guess: the same catalogue writes `ш` as `sch` in one search and `sh` in the
next.

## What Changes

- A Russian query matches the latin name. Each Cyrillic letter matches any of
  its plausible latin spellings, so `Шувалово` finds `Schuvalovo` and
  `Shuvalovo` alike, and `Жуково` finds both `Zhukovo` and `Jukovo`.
- A Cyrillic entry, should the catalogue ever carry one, still matches its own
  spelling.
- A latin query keeps the plain substring match it had — same results, and no
  pattern built for the common case.
- A space in the query stands for whatever separates words in a slug (`_`, `-`
  or a space), so `Лаврово 2026` finds `Lavrovo-2026`.

## Capabilities

### Added Capabilities
- `lizaalert-integration`: the catalogue filter accepts the crew's own
  language.

## Impact

- **Frontend**: `src/lib/translit.ts` (new), `src/lib/project-list.ts`.
- **Backend**: none.
- **Risk**: low. A query with no Cyrillic takes exactly the path it took
  before; the pattern is built once per call, not once per row.
