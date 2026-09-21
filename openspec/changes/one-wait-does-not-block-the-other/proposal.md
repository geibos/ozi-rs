## Why

One `busy` flag guarded three unrelated things: the catalogue walk, a bundle
download, and opening a bundle from disk. The walk runs at every launch, is up
to a thousand pages, and holds the flag for all of it — so on a field link the
only download button in the application was disabled for minutes after every
launch, and the operator's remedy was to press Stop on a refresh they did want.

The two do unrelated work. The walk reads a remote HTML listing; a download
fetches files into the bundles root and writes each through a `.part` file
before renaming it, so a walk that scans the root while a download runs cannot
see a half-written map.

The previous slice on this made the refusal *visible* — the button says why it
is disabled — which was right, but the honest next question is why it is
disabled at all.

## What Changes

- `busy` becomes `listing_busy` and `bundle_busy`. A walk blocks another walk;
  a bundle operation blocks another bundle operation; neither blocks the other.
- Each flag is released by the completion of its own operation, so finishing
  the walk no longer clears a running download's progress line.
- The refusal and the disabled button say "another bundle is still opening",
  which is now the only thing they can mean.

## Capabilities

### Modified Capabilities
- `lizaalert-integration`: a catalogue refresh no longer blocks a download.

## Impact

- **Backend**: `LizaAlertState`, `begin_load_projects`, `begin_load_project`,
  `begin_open_local_bundle`, `apply_projects_loaded`, `apply_project_loaded`,
  `AppStateDto`.
- **Frontend**: `stores.ts` (`listingBusy`, `bundleBusy`, `busy` as either),
  `BundleLoader.svelte`, `+page.svelte`, i18n.
- **Risk**: medium — it removes a mutual exclusion. What it protected is
  covered above: the two operations touch different things, and the download's
  `.part`-then-rename means the walk cannot read a partial file.
