---
source: docs.rs, crates.io API, official specs, and reference implementation docs
library: Rust geodata import
package: rust-geodata-import
topic: ZIP-based geodata import support matrix for GPX, KML, Ozi text, and ozf2
fetched: 2026-03-28T08:10:00Z
official_docs: https://docs.rs/zip/latest/zip/
---

# Rust ZIP geodata import research

## 1) ZIP archive reading in stable Rust

### Recommended

- `zip` crate (`zip = 8.4.0` on docs.rs) is the practical default for stable Rust.
  - Docs state support for reading ZIP files, many common compression methods, AES/ZipCrypto, and `ZipArchive::new()` for reading.
  - It is based on PKWARE APPNOTE 6.3.9.
  - Maintenance signal is good: current 2026 release, documented, active repo.

### Recommendation

- Use `zip` for first-slice archive traversal and entry extraction.
- Keep feature usage conservative unless encrypted or unusual compression methods matter.

### References

- `zip` crate docs: https://docs.rs/zip/latest/zip/
- `zip` crate page: https://docs.rs/crate/zip/latest
- PKWARE APPNOTE.TXT v6.3.9 (linked from crate docs)

## 2) GPX parsing in stable Rust

### Recommended

- `gpx` crate (`gpx = 0.10.0`) is the clearest maintained Rust option.
  - Reads and writes GPX 1.0 and 1.1.
  - Uses `geo-types` primitives.
  - Good adoption signal on crates.io.

### Important limitation

- The crate docs explicitly say GPX extensions are **not yet supported**.
- For first-slice import of ordinary tracks/waypoints/routes, this is still a strong choice.

### Spec references

- Official GPX 1.1 schema and documentation from Topografix.

### References

- `gpx` crate docs: https://docs.rs/gpx/latest/gpx/
- `gpx` crate page: https://docs.rs/crate/gpx/latest
- crates.io API result for `gpx`
- GPX home/spec: https://www.topografix.com/gpx.asp
- GPX 1.1 schema: https://www.topografix.com/gpx/1/1/gpx.xsd

## 3) KML parsing options in Rust

### Best crate option

- `kml` crate (`kml = 0.13.0`) is the best obvious Rust option.
  - Reads and writes KML.
  - Focuses on conversion to `geo-types`.
  - Supports KMZ reading with its `zip` feature/default features.

### Risk notes

- Docs coverage is relatively low (~19.71% on docs.rs), so API discovery risk is higher than for `gpx`.
- KML itself is much broader than a track/waypoint importer usually needs; styles, folders, overlays, gx extensions, and arbitrary ExtendedData can complicate fidelity.

### Fallback / lower-level option

- `quick-xml` is a strong low-level XML parser if you want a constrained KML subset parser.
  - Better if you only need `Document` / `Folder` / `Placemark` / `Point` / `LineString`.

### Spec references

- Google KML Reference and OGC KML 2.2 references linked from it.

### References

- `kml` crate docs: https://docs.rs/kml/latest/kml/
- `kml` crate page: https://docs.rs/crate/kml/latest
- crates.io API result for `kml`
- `quick-xml` docs: https://docs.rs/quick-xml/latest/quick_xml/
- KML reference: https://developers.google.com/kml/documentation/kmlreference

## 4) Ozi `.map`, `.plt`, `.wpt`

### Rust crate situation

- I did **not** find a credible maintained Rust crate specifically for OziExplorer text formats.
- crates.io API search for `ozi` returned unrelated crates; search for `ozf` returned zero crates.
- That strongly suggests custom parsers are the realistic path.

### Spec/reference situation

- `.map`: OziExplorer publishes a map file format page, and GDAL has a built-in `MAP -- OziExplorer .MAP` raster driver with georeferencing support.
- `.plt` and `.wpt`: OziExplorer publishes text field definitions on its file format page.
- GPSBabel has mature reader/writer support for OziExplorer waypoint/track/route text formats, which is a useful implementation reference.

### Practical recommendation

- `.plt` and `.wpt`: custom Rust parsers are very feasible early; they are line-oriented text formats with published field structure.
- `.map`: custom parsing is feasible, but product value depends on what raster payloads you will actually support with it.
  - It is more calibration/projection metadata than standalone geodata.
  - If paired with unsupported raster payloads like `ozf2`, usefulness is reduced.

### References

- OziExplorer file formats: https://www.oziexplorer4.com/eng/help/fileformats.html
- OziExplorer map file format: https://www.oziexplorer4.com/eng/help/map_file_format.html
- GDAL MAP driver: https://gdal.org/en/latest/drivers/raster/map.html
- GPSBabel OziExplorer format: https://www.gpsbabel.org/htmldoc-development/fmt_ozi.html
- crates.io API search for `ozi`
- crates.io API search for `ozf`

## 5) `ozf2` decoding feasibility in Rust

### Current outlook

- `ozf2` decoding does **not** look like a good near-term Rust implementation target.

### Why

- I found no credible Rust crates for `ozf`/`ozf2`.
- OziExplorer and OSM references describe OZF2/OZFx as proprietary raster formats produced by `img2ozf`.
- GDAL has a built-in driver for `.map`, but I did not find a corresponding current GDAL OZF raster driver page.
- That suggests ecosystem support is much weaker than for GPX/KML/ZIP or even `.map` metadata.

### Main blockers

- Proprietary / reverse-engineered format status.
- Unclear spec availability.
- Potential licensing contamination risk if implementation work relies on non-permissive reverse-engineered code.
- Higher validation burden because raster decoding bugs are harder to detect than line-oriented text parsing bugs.

### Practical recommendation

- Treat `ozf2` as a feasibility-risk item.
- If it ever becomes necessary, first do a separate discovery slice on:
  - publicly documented spec availability;
  - legal/licensing status of reference implementations;
  - whether external conversion should be preferred over native decoding.

### References

- OSM OziExplorer page discussing proprietary ozfx2/ozfx3 + `img2ozf`: https://wiki.openstreetmap.org/wiki/OziExplorer
- crates.io API search for `ozf`

## 6) Concise support matrix

| Format | Classification | Why |
|---|---|---|
| ZIP container | good first-slice candidate | `zip` crate is maintained, stable, and directly suited to entry enumeration/extraction. |
| GPX | good first-slice candidate | `gpx` crate is maintained and supports GPX 1.0/1.1 well for common import needs. |
| KML | later candidate | `kml` crate exists and is viable, but KML surface area and API/doc risk are higher. |
| Ozi `.plt` | later candidate | No Rust crate, but format is simple and published; custom parser is realistic. |
| Ozi `.wpt` | later candidate | Same as `.plt`; line-oriented text format with published fields. |
| Ozi `.map` | later candidate | Feasible text parsing, but value depends on paired raster support and calibration handling. |
| `ozf2` raster payload | feasibility risk | No Rust crate found; proprietary/reverse-engineered format with spec/licensing uncertainty. |

## 7) Bottom-line recommendation for `ozi-rs`

### Best first slice

1. `zip` crate for archive handling.
2. `gpx` crate for GPX entries inside ZIPs.
3. Optional custom parsers next for Ozi `.plt` and `.wpt`.

### Defer

- KML until GPX/Ozi text import path is stable.
- `.map` until raster pairing strategy is decided.
- `ozf2` until a separate feasibility/legal investigation is done.
