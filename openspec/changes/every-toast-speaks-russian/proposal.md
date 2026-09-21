## Why

Third time. The library rows' tooltips were English inside a Russian window;
the bundle progress arrived as an English sentence from the backend; the folder
import's summary did too. Each was found by looking at one screen, and each fix
was a sweep of that screen.

The project's own rule says the third fix is a test over the shape. Written as
one — a toast's message must come from the dictionary — it found sixteen more
on its first run, none of them in a place anybody had looked: rename a
waypoint, hide a track, export a GPX, delete a track, simplify, reveal in the
file manager. All of them are the sentence a crew reads at the moment something
went wrong, which is the worst moment to be handed a language they do not
read.

`no-untranslated-labels` did not catch them because it watches `aria-label`,
`title` and `placeholder`. A toast is none of those.

## What Changes

- Sixteen messages move into the dictionary, in both languages.
- A guard fails on any `toast.*()` whose first argument is a typed-in string.
  The second argument is untouched: `description: String(error)` is the
  backend's own words about a failure, which is evidence rather than interface
  text.
- The guard reads the same narrow rule as `no-untranslated-labels` — a Latin
  letter typed in — so a message assembled entirely from translations passes,
  and it strips comments first, because `api.ts` documents this very rule in a
  comment that quotes a bad call.

## Capabilities

### Modified Capabilities
- `ui-shell`: every message the operator is shown comes from the dictionary.

## Impact

- **Frontend**: seventeen i18n keys, six components,
  `src/test/no-english-toasts.test.ts` (new).
- **Backend**: none.
- **Risk**: low. The wording changed language, not meaning.
