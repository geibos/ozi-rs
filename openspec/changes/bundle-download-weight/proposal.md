## Why

"Открыть бандл (скачать)" fetches the project's whole directory tree — every
subdirectory, including the print maps and the Android packages this app cannot
open. The progress panel counted files ("3 of 47") and said nothing about
bytes, so the operator could not tell whether the download would finish on a
phone tether or run for an hour.

The scan already reads every directory listing to build the file list, and
those listings state each file's size. The total was there to be had.

The panel had a second problem: the bundle loader opens as a sheet with a
`z-50` overlay while the panel sits at `z-40`, so opening the loader to queue
the next map hid the download it had just started.

## What Changes

- The scan sums the sizes of the files it is about to fetch — skipping those
  already on disk, so a resumed download does not re-announce the whole bundle
  — and reports it with the progress.
- Byte progress accumulates across the concurrent workers, so the panel shows
  fetched-of-total rather than only a file count.
- The progress panel renders that total and sits above the loader's overlay.

## Capabilities

### Modified Capabilities
- `map-bundles`: what a running bundle download reports.

## Impact

- **Backend**: `RemoteFileDownload` carries a size; `download_bundle_concurrent`
  reports totals and accumulates fetched bytes.
- **Frontend**: `DownloadPopup.svelte`.
- **Risk**: low. A listing without sizes reports no total, as before.
