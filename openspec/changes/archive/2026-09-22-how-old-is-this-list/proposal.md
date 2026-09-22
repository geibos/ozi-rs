## Why

The catalogue cache has written an ISO timestamp since it was introduced, and
`loadCatalogCache` threw it away on read. Nothing ever showed it.

That timestamp matters in exactly the state the cache exists for. Offline, the
loader says «Нет связи — список сохранённый», which is honest and incomplete: a
list saved this morning and a list saved three weeks ago look identical, and a
search published yesterday is missing from both. Without the date, a crew
cannot tell "this search does not exist" from "my list predates it" — and the
first reading is the one that sends people looking for another way in.

## What Changes

- The loader shows when the list on screen was written, beside the match
  count, and inside the offline notice where it matters most.
- The timestamp is read from the cache at startup and updated whenever the
  cache is written.
- It is absolute — day, month and time — rather than "3 часа назад": relative
  Russian needs plural rules to be written correctly, and a clock time is what
  a crew compares against when a coordinator says a search went up.
- A timestamp that is not a date prints nothing rather than "Invalid Date".

## Capabilities

### Modified Capabilities
- `lizaalert-integration`: the cache's write timestamp is shown, not only
  stored.

## Impact

- **Frontend**: `src/lib/catalogue-age.ts` (new), `stores.ts`
  (`catalogueWrittenAt`, `loadCatalogWrittenAt`), `BundleLoader.svelte`, three
  i18n keys.
- **Backend**: none.
- **Risk**: low; it reads a field that was already being written.
