## Why

The download progress panel and the toaster both live in the bottom-right
corner, and svelte-sonner's viewport carries `z-index: 999999999` against the
panel's `60`, so a toast always draws over the panel.

Measured on the stand at 1024×640: the panel occupied 692..1012 × 550..628 and
an error toast covered 644..1000 × 542..616. The overlap hid the panel's title,
its Cancel button and the "files / bytes" row — everything except the tail of
the per-file bars — for as long as the toast was up.

This is not a rare pairing. A bundle download is exactly when toasts appear: a
file that fails after its retries, a preview that could not be read, a refusal
while the app is busy. The operator is told a file failed and, at the same
moment, loses sight of how far the rest of the bundle got.

## What Changes

- While the download panel is on screen, the toaster steps above it instead of
  over it. The lift is the panel's measured height plus its own margin and a
  gap, so it follows a panel that grows with the number of files.
- With no download on screen, toasts keep the corner they have always had.

## Capabilities

### Added Capabilities
- `map-bundles`: the progress panel stays readable while a toast is up.

## Impact

- **Frontend**: `src/lib/toast-offset.ts` (new), `src/lib/stores.ts`
  (`downloadPopupHeight`), `src/components/DownloadPopup.svelte` (measures
  itself), `src/routes/+layout.svelte` (passes the offset).
- **Backend**: none.
- **Risk**: low. Nothing moves unless a download is on screen, and the offset is
  clamped so a toast never sits closer to the edge than it does today.
