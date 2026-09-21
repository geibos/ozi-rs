## Why

"Открыть бандл (скачать)" fetches the project's whole directory tree. A bundle
carries print sheets and Android tile packs this app cannot open, and on a
phone tether they are most of the transfer — the owner's July note asks for a
type filter for exactly that reason.

What is worth carrying is a field judgement: a crew that also runs OziExplorer
on a phone wants the Android pack; one working only here does not. So the app
should not decide. It should show what the bundle holds and let the operator
clear what they do not need.

## What Changes

- A previewed bundle reports its top level — the entries the site lists, with
  the sizes it states — and the loader shows them as a checklist.
- Everything is checked. Nothing is skipped unless the operator clears it, so
  the behaviour is unchanged until they choose otherwise.
- The download leaves the cleared entries on the server.

## Capabilities

### Modified Capabilities
- `map-bundles`: what a bundle download fetches.

## Impact

- **Backend**: `BundleEntry` on `LizaProject`, filled from the listing online
  and from disk offline; `skip_top_level` through `BundleDownloadConfig` and
  the walk; `load_project` takes the list.
- **Frontend**: the checklist in `BundleLoader.svelte`; i18n; the wrapper.
- **Risk**: low. An empty skip list is the previous behaviour, and the choice
  resets when a different project is previewed.
