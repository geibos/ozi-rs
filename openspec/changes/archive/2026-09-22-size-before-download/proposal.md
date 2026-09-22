## Why

Opening a map is a commitment to a download, and nothing on screen says how
big it is. In a штаб that runs off a phone tether the difference between a
sixteen-megabyte topo layer and a two-hundred-megabyte satellite layer decides
whether the map gets fetched at all — and the operator was choosing blind.

The size is already in the listing the app parses: the site prints it in the
cell next to each file. It cost nothing to read and was thrown away.

While reading them: three copies of a byte formatter lived in the loader, the
download popup and the cold-start status bar, all printing English units into a
Russian window and disagreeing about MB versus MiB.

## What Changes

- Map packages carry their size — from the listing for a remote map, from the
  file for a cached one — and the loader and the Maps tab show it.
- One shared byte formatter, in the app's language, replaces the three copies.
- The cold-start status bar's remaining English ("Downloading", "Cancel") is
  translated.

## Capabilities

### Modified Capabilities
- `map-bundles`: what the operator knows before starting a download.

## Impact

- **Backend**: `parse_entry_sizes` / `parse_human_size` in
  `infrastructure/lizaalert.rs`; `size_bytes` on `LizaMapPackage` and its DTO;
  generated bindings.
- **Frontend**: `src/lib/format-bytes.ts` (new), `BundleLoader.svelte`,
  `MapsTab.svelte`, `DownloadPopup.svelte`, `routes/+page.svelte`, i18n keys.
- **Risk**: low. An unknown size stays unknown rather than becoming zero.
