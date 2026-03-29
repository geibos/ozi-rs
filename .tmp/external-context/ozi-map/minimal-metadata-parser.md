---
source: Official OziExplorer docs + GDAL docs/source + GPSBabel docs
library: OziExplorer .map
package: ozi-map
topic: minimal metadata parser and importer guidance
fetched: 2026-03-28T09:43:06Z
official_docs: https://www.oziexplorer4.com/eng/help/map_file_format.html
---

# OziExplorer `.map` minimal metadata parser guidance

## Authoritative references

- OziExplorer map file format: https://www.oziexplorer4.com/eng/help/map_file_format.html
- GDAL MAP driver docs: https://gdal.org/en/stable/drivers/raster/map.html
- GDAL MAP driver source: https://raw.githubusercontent.com/OSGeo/gdal/master/frmts/map/mapdataset.cpp
- GPSBabel Ozi docs (useful for text/encoding conventions, not `.map` georeferencing): https://www.gpsbabel.org/htmldoc-development/fmt_ozi.html

## Minimal structure to parse

The file is line-oriented and order-sensitive.

1. `OziExplorer Map Data File Version ...`
2. map title
3. referenced raster filename
4. map code / TIFF scale factor placeholder (`1 ,Map Code,` often present)
5. datum line, e.g. `WGS 84,, 0.0000, 0.0000,WGS 84`
6. `Reserved 1`
7. `Reserved 2`
8. magnetic variation line
9. projection line, e.g. `Map Projection,Mercator,...`
10-39. `Point01` .. `Point30` calibration lines (always 30 lines; many may be blank)
40. `Projection Setup,...`
41+. optional sections such as map features/comments, attached files, moving-map border points, grid setup, open position, image width/height

For a first Rust importer, you can ignore features/comments/TF/grid/MOP/IWH, but preserve unknown lines if you plan later round-trip support.

## Fields worth extracting for MVP metadata

### Header / identity

- format/version string from line 1
- title from line 2
- raster reference from line 3

### Raster filename handling

GDAL's current behavior is a good reference:

- treat line 3 as the image path
- reject path traversal sequences
- if relative, resolve relative to the `.map` file directory
- if absolute but missing, retry with just the basename in the `.map` file directory
- expect Windows paths (`D:\\maps\\foo.ozf2`) even on non-Windows hosts
- comparisons should be case-insensitive when trying local fallback resolution

### Datum / projection

From Ozi docs, for normal maps only the **first datum field** is typically meaningful. Minimal parser should extract:

- datum name: first field of datum line
- projection name: second field of `Map Projection,...`
- projection setup numbers from `Projection Setup,...`

Useful `Projection Setup` fields for minimal metadata:

1. latitude origin
2. longitude origin
3. scale factor / K factor
4. false easting
5. false northing
6. latitude 1
7. latitude 2
8. height (only for Vertical Near-Sided Perspective)

### Calibration points

Each `PointNN` line contains:

- pixel `x,y`
- coordinate mode (`deg` commonly)
- latitude degrees/minutes + hemisphere
- longitude degrees/minutes + hemisphere

Minimal importer should:

- parse all 30 point slots
- keep only populated points
- convert hemispheres `S/W` to negative signs
- tolerate blank calibration points
- record both pixel and geographic coordinates

Example:

`Point01,xy, 494, 235,in, deg, 24, 0,S, 148, 0,E, grid, , , ,S`

=> pixel `(494,235)`, lat `-24.0`, lon `148.0`

## Recommended minimal Rust data model

```rust
struct OziMapMetadata {
    version: String,
    title: String,
    raster_ref_raw: String,
    raster_ref_resolved: Option<PathBuf>,
    datum_name: String,
    projection_name: String,
    projection_setup: ProjectionSetup,
    calibration_points: Vec<CalibrationPoint>,
    moving_map_border: Vec<BorderPoint>,
    image_size_hint: Option<(u32, u32)>,
}
```

Where `CalibrationPoint` stores pixel x/y and optional lat/lon, and `BorderPoint` stores `MMPXY`/`MMPLL` pairs if present.

## Common pitfalls

1. **Do not treat `.map` as free-form CSV.** It is line-typed and positional.
2. **Do not assume only used calibration points exist.** There are always 30 `PointNN` rows.
3. **Do not require all optional sections.** Many files stop after `Projection Setup`; others include moving-map sections.
4. **Do not trust line 3 blindly.** GDAL explicitly guards against path traversal and retries basename fallback.
5. **Do not assume geotransform is always derivable from simple affine math.** GDAL may expose either a geotransform or GCPs depending on calibration.
6. **Do not ignore moving-map border lines if you need coverage extent.** `MMPNUM`, `MMPXY`, and `MMPLL` can define a neatline/cutline distinct from full image bounds.
7. **Do not over-interpret datum shift tail fields** on the datum line for MVP; Ozi docs say usually only the first field matters for normal maps.
8. **Expect Windows-1252-ish text in the ecosystem.** GPSBabel documents `windows-1252` as default for Ozi text formats; keep importer text handling permissive.
9. **Expect Windows path syntax and legacy filenames.** Backslashes, drive letters, and mixed-case basenames are common.

## Actionable implementation strategy

1. Validate extension/header contains `OziExplorer Map Data File`.
2. Read all lines as text, preserving order.
3. Parse fixed header lines by index.
4. Parse 30 `PointNN` lines into optional calibration points.
5. Parse `Projection Setup` into typed numeric fields.
6. Optionally parse `MMPNUM`/`MMPXY`/`MMPLL` for extent metadata.
7. Resolve raster filename relative to `.map` location with secure fallback behavior.
8. Defer full projection math to a later slice; first importer can expose raw parsed metadata plus geographic calibration points.

## Useful behavior to mirror from GDAL

- identify by `.MAP` extension plus header text
- open referenced raster after resolving filename
- use moving-map border points to build neatline when border differs from rectangular image bounds
- distinguish between affine geotransform and GCP-based georeferencing

## Best MVP scope

For `ozi-rs`, the safest first slice is:

- parse `.map` text metadata
- resolve raster reference securely
- capture datum/projection strings and projection setup values
- capture populated calibration points
- capture optional border/extents from moving-map lines
- leave full projection reconstruction / image georeferencing as a later infrastructure slice
