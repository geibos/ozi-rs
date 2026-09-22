## Why

A bundle is fetched whole. The 16 MiB topo layer lands in seconds; the 185 MiB
satellite layer takes minutes. The backend has always made a map openable the
moment its file lands — `note_bundle_file_ready` sets the map's local path, and
opening it then reads from disk instead of downloading.

Nothing ever said so. The crew watched a file counter and waited for the whole
bundle before starting, which is the owner's July note: "a bundle cannot be
opened while the rest still downloads".

## What Changes

- The first map of a bundle download that becomes openable is announced, once
  per download, with the invitation to open it while the rest continues.
- A test pins the capability itself: a map whose file lands mid-download opens
  from disk rather than starting a second download.

## Capabilities

### Modified Capabilities
- `map-bundles`: what the operator is told about a bundle in progress.

## Impact

- **Backend**: none — a test only.
- **Frontend**: `src/lib/ready-maps.ts` (new), the layout's event wiring, i18n.
- **Risk**: none to the download path; the announcement is a toast.
