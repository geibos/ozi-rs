## Why

A survey of the bundle flow found that the parts a field crew depends on say
nothing when they fail, and say the wrong thing when they succeed.

- The only download button in the app is refused while the catalogue walk holds
  the busy flag. The refusal was indistinguishable from success: no error, no
  status, nothing. The catalogue now walks up to a thousand pages, so that
  window is minutes long on a field link.
- Every failure in the loader was swallowed by an empty `catch`, and the error
  reporter is disabled outside dev builds. A failed preview looked like an empty
  project; a failed download looked like nothing at all.
- The progress panel is shown while a download id is set and the app is busy.
  Nothing cleared the id when a download finished, so the panel reappeared —
  with the finished download's rows — the next time anything made the app busy.
- Opening a map from the Maps tab left the "open the loader" flag set, so the
  loader reopened over the map that had just been opened.
- The catalogue repair taught the remote walk to find `.sqlitedb` files in any
  subdirectory, but the local mirror still read one fixed folder. A bundle
  stored anywhere else read as "not downloaded" and was fetched a second time.
  The same mismatch applied to the coordinates file, which the online side
  matches by pattern and the cache demanded by exact name.

## What Changes

- Refusing to open a bundle returns a reason, and the loader shows it. The
  download button is disabled and says why while the app is busy.
- The loader reports preview, download and local-bundle failures as toasts.
- A finished download releases the progress panel.
- Opening a map clears the loader flag.
- Cached `.sqlitedb` maps are found anywhere inside the bundle directory, and
  the coordinates file is matched by pattern, as online.

## Capabilities

### Modified Capabilities
- `map-bundles`: refusal reporting, progress-panel lifetime, and cached-bundle
  detection.

## Impact

- **Backend**: `begin_load_project` returns a typed refusal;
  `read_cached_sqlite_map_packages` and `project_coordinates_path` in
  `infrastructure/lizaalert.rs`.
- **Frontend**: `BundleLoader.svelte`, `MapsTab.svelte` (also localized),
  `stores.ts`, i18n keys.
- **Risk**: low. The cache change widens a search; the refusal change turns a
  silent no-op into an error the UI already knows how to display.
