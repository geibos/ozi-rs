## Why

The catalogue is walked in full at every launch — up to a thousand pages, a
page request each — and the walk holds the application's busy flag for its
whole length. While it runs, the only download button in the app is disabled.

On a field link that is minutes. A crew that launched the app to fetch one
bundle they already know the name of has nothing to do but wait for a listing
of thirteen thousand searches they are not going to read. The list they can see
is already usable: the cached one is applied first, and each page adds to it as
it arrives. What was missing was any way to say "that is enough".

## What Changes

- The catalogue walk takes a cancellation token and reports whether it was
  stopped, alongside what it read and how many pages it got through.
- A `cancel_project_listing` command stops the walk that is running.
- The loader shows a stop control beside the "refreshing" hint while a refresh
  is in flight.
- A stopped walk is not written over the cache. Its projects are the first few
  pages; saving them as the catalogue would leave a crew offline tomorrow with
  the newest few dozen searches and no sign the rest existed.
- The status line distinguishes a stopped refresh from a complete one, because
  "412 projects" and "412 projects so far" are different facts.

## Impact

- Affected specs: `lizaalert-integration`
- Affected code: `src-tauri/src/infrastructure/lizaalert.rs` (`CatalogueWalk`,
  `walk_project_listing`), `src-tauri/src/application/mod.rs`,
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`,
  `src/lib/bindings.ts`, `src/lib/api.ts`,
  `src/components/BundleLoader.svelte`, `src/lib/i18n.ts`,
  `src/test/stand/tauri-core.ts`
