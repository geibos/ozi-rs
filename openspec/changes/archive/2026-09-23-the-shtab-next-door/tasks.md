## 1. Reading

- [x] 1.1 `import/wpt.rs`: signature check, four header lines, the four fields that matter
- [x] 1.2 Placeholder rows, blank lines and unparseable rows skipped, tested
- [x] 1.3 Windows-1251 through the PLT decoding chain, tested
- [x] 1.4 Short rows (trailing fields omitted) are valid, tested

## 2. The escape that had to go

- [x] 2.1 Round-trip test out and back through WPT — the thing that could not exist before
- [x] 2.2 Both exporters keep the space; the reasoning is in the code, not only here
- [x] 2.3 A name carrying `С` is not read as a name carrying a comma

## 3. Reaching the operator

- [x] 3.1 `import_wpt_file_into_project`: a layer per file, named after it
- [x] 3.2 `import_wpt` command, registered, bindings regenerated
- [x] 3.3 The import dialog accepts `.wpt`; the filter label says so in both languages
- [x] 3.4 The stand answers `import_wpt` by putting marks on the screen

## 4. Gates

- [x] 4.1 `just ci` green
- [x] 4.2 Walked on the stand 2026-09-23: importing a `.wpt` put «ШТАБ-2» and «Рубеж» in the Waypoints tab and took the map from two markers to four. The first attempt put the marks into the fixture's existing layer, which looked right in the list and drew nothing — the marker reconciler gates on a fingerprint over the layer set, and an import into an existing layer does not move it. The stand creates a layer per file now, as the backend does. `docs/progress/2026-09-23-wpt-import/wpt-imported.png`
- [x] 4.3 `just smoke` green (2026-09-23, both journeys against a bundle built the same hour; the layer journey added to the gate that day walks create → active → delete over real IPC)
