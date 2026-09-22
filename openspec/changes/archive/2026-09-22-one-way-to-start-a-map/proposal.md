## Why

`open_selected_map` opens a map from disk when it is there and starts a
download when it is not, returning the download id in the second case so the
caller can put the progress panel on screen with its cancel. Two of the three
callers used it. The command palette dropped it.

From the palette, then, asking for a map that was not on disk did this: the
palette closed, a download began, nothing appeared, nothing could be cancelled,
and the map never opened — because the palette's own navigation is gated on an
`activeMap` that a running download has not set yet. Reachable today through
the palette's recent-files list, whose entries name a map that may since have
been removed, moved, or downloaded onto a different machine.

## What Changes

- One helper, `openMapShowingDownload`, holds the rule: open the map, and if a
  download started instead, put its progress on screen and say the map is not
  open yet. All three callers use it — the palette, the bundle loader and the
  Library Maps tab.
- The stand plays a single-map download, which it never did: its
  `open_selected_map` always answered "already on disk", so this path could not
  be looked at.

## Capabilities

### Added Capabilities
- `map-bundles`: a map fetched in order to open it is visible and cancellable
  from wherever it was asked for.

## Impact

- **Frontend**: `src/lib/actions/open-map.ts` (new), `CommandPalette.svelte`,
  `BundleLoader.svelte`, `library/MapsTab.svelte`, `src/test/stand/`.
- **Backend**: none — it already returned the id.
- **Risk**: low. Two callers keep the behaviour they had; the third gains it.
