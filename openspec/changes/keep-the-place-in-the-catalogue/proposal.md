## Why

On the workspace route the bundle loader lives inside a Sheet, and a Sheet
unmounts its content. Everything the component held went with it: the search
text, the "only downloaded" toggle, the selected project, the contents the
operator had unchecked, and the place in a thirteen-thousand-row list.

Closing the loader to look at the map is the ordinary thing to do — it is why
the loader is a Sheet rather than a screen. Every time, the operator came back
to an empty search box at the top of the catalogue and had to find the search
again by name. The names are latin transliterations, so finding it again is not
a matter of typing what the search is called.

## What Changes

- The loader's browsing position — filter, "only downloaded", selected project,
  cleared bundle contents and scroll offset — is held outside the component and
  restored when it mounts again.
- It is session state, not persisted: a crew opening the app tomorrow starts on
  today's search, not on yesterday's.
- The scroll window still resets when the filter changes, but no longer on the
  component's first run, where it would have discarded the position just
  restored.

## Impact

- Affected specs: `map-bundles`
- Affected code: `src/lib/stores.ts` (new `bundleLoaderView`),
  `src/components/BundleLoader.svelte`, `src/test/stubs/app/`
