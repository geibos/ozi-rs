## Why

Clicking a project in the loader previews it: the backend fetches that
bundle's map list in a thread of its own. Nothing ordered two of those.

A crew scanning the catalogue clicks several projects in a row — that is how
you find the right one. Each click spawned a thread, and whichever answered
last won, unconditionally. So the map list could swap, seconds later, to a
project the operator had already moved past, under the row they were actually
looking at. A failure from an abandoned preview reported itself in the status
bar the same way, naming a project nobody had asked about any more.

Two smaller faults sat on the same path:

- A preview deliberately does not take the busy flag, so that a click during
  the catalogue walk is not silently swallowed. But it cleared the flag when it
  finished — releasing something it never took. A preview landing mid-download
  let a second download start.
- The loader decided the preview had arrived by comparing **display names**.
  A name is not identity; the catalogue holds thirteen thousand of them and a
  repeat search name is ordinary. It also ran a fifteen-second timer that
  cleared the spinner outright while the request was still in flight, which
  told the operator the list had arrived when nothing had.

## What Changes

- The newest preview is the only one that may land. A result whose slug is no
  longer the selected project is dropped, success or failure alike.
- A finished preview leaves the busy flag exactly as it found it.
- The loader matches the arriving project by slug, not by display name.
- The fifteen-second timer no longer ends the wait; it says the wait is running
  long. What bounds the request is the backend's HTTP read timeout, which
  surfaces a real failure.

## Impact

- Affected specs: `map-bundles`
- Affected code: `src-tauri/src/application/mod.rs` (new `apply_preview_loaded`),
  `src-tauri/src/commands/mod.rs` (`preview_project`),
  `src/components/BundleLoader.svelte`, `src/lib/i18n.ts`,
  `src/test/stand/tauri-core.ts`
