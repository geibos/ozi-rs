---
source: Official OziExplorer pages, GDAL docs/RFC, docs.rs/lib.rs, and public implementation references
library: OZF2 / OZFX3
package: ozf2
topic: OZF2/OZFX3 raster decoding feasibility for an open-source Rust desktop app
fetched: 2026-03-28T09:43:06Z
official_docs: https://www.oziexplorer4.com/img2ozf/img2ozf.html
---

# OZF2 / OZFX3 decoding feasibility

## Bottom line

- **There is no credible current Rust crate** for OZF2/OZFX3 decoding that showed up in current package indexes: docs.rs search for `ozf2` returned **no results**, and lib.rs search for `ozf2` returned **Nothing found**.
- **The format family is proprietary and not openly specified by OziExplorer.** Public interoperability relies on **reverse-engineered** implementations.
- **OZF2/OZFX3 decoding is feasible in practice for older files**, because there are public reference implementations, but it is **not a low-risk foundation** for a clean open-source Rust project.
- **OZF4 is a further blocker**: OziExplorer now uses a newer format and GDAL's adopted 2025 RFC says **OZI now uses a new format v4 that has not been reverse engineered**.

## What current authoritative sources say

### Official OziExplorer status

From OziExplorer's current Img2ozf page:

- Img2ozf now converts images to **OZF4** for PC OziExplorer, OziExplorerCE, and OziExplorer for Android.
- The page also says older behavior existed where **Ozfx3 files produced from normal map images will be used exactly the same as the Ozf2 files produced by the old img2ozf**.

Implication: OZF2/OZFX3 are legacy-ish but still historically relevant; official tooling has moved on to **OZF4**.

Source: https://www.oziexplorer4.com/img2ozf/img2ozf.html

### GDAL status

GDAL RFC 108 (adopted, implemented for GDAL 3.11) states:

- **"OZI OZF2/OZFX3: OZI now uses a new format v4 that has not been reverse engineered."**

The current GDAL raster driver index also lists **OZI** among the drivers **removed in GDAL 3.11**.

Implication: even the main open geospatial interoperability library decided this path is not strategic to keep maintaining.

Sources:

- https://gdal.org/en/stable/development/rfc/rfc108_driver_removal_3_11.html
- https://gdal.org/en/stable/drivers/raster/index.html

## Licensing / spec / reverse-engineering status

- I found **official user-facing format docs for `.map`, `.plt`, `.wpt`, etc.**, but **not an official open spec for OZF2/OZFX3 raster internals**.
- Public decoding knowledge comes from reverse-engineering discussions and code.
- A long-standing public reverse-engineering thread on Google Groups (`ozex.dev`) documents magic values, tile structure assumptions, encryption/decryption behavior, zoom table details, and export/locking flags.
- GDAL's old OZI driver source is MIT-style licensed GDAL code, but the **understanding embodied in it is reverse-engineered**, not vendor-specified.

Practical reading:

- **Legally safer than copying proprietary vendor code** is to rely on independently published open-source implementations or isolate optional interoperability behind a separate component.
- But **product risk remains** because the vendor format is proprietary, the spec is unofficial, and newer variants are not publicly understood.

Key references:

- Google Groups reverse-engineering discussion: https://groups.google.com/g/ozexdev/c/aMOu10waOyM
- Historical GDAL OZI driver source: https://svn.osgeo.org/gdal/tags/2.2.0/gdal/frmts/ozi/ozidataset.cpp

## Known implementation references

### Strongest reference: historical GDAL OZI driver

The old GDAL OZI driver is the most credible public implementation reference I found. It shows:

- palette-based 8-bit raster handling;
- tiled blocks;
- zlib-compressed tile payloads;
- decryption logic for OZFX3-style files;
- overview/zoom level handling.

Reference: https://svn.osgeo.org/gdal/tags/2.2.0/gdal/frmts/ozi/ozidataset.cpp

### Other public references

- `vss-devel/tilers-tools`: Python tooling; README says `ozf_decoder.py` converts `ozf2` and `ozfx3` to TIFF and can help when GDAL lacks OZF support.
  - https://github.com/vss-devel/tilers-tools
- `zhmurov/ozf2tiff`: tiny C++ converter; README says it is based on code reposted from a forum and lightly fixed.
  - https://github.com/zhmurov/ozf2tiff

These are useful as research material, but they are **not equivalent to a maintained Rust-native dependency**.

## Rust ecosystem status

- **docs.rs search**: no `ozf2` results.
- **lib.rs search**: no `ozf2` results.
- I did not find a current Rust crate that looks maintained, widely used, and suitable as a dependency for production raster loading.

References:

- https://docs.rs/releases/search?query=ozf2
- https://lib.rs/search?q=ozf2

## Safe recommendation for an open-source Rust desktop app

### Recommended project policy

1. **Do not make native OZF2/OZFX3 decoding an MVP dependency.**
2. Treat OZF2/OZFX3 as **legacy, proprietary, reverse-engineered import support**.
3. If you support it at all, make it:
   - **read-only**;
   - **explicitly optional** behind a feature flag or adapter boundary;
   - **clearly documented as best-effort**;
   - **isolated in infrastructure**, not in the core domain/application model.
4. Prefer first-class support for open or better-specified raster inputs where possible.
5. **Do not invest in OZF4 support unless a new credible open implementation/spec appears**.

### Best practical path if you still need OZF2/OZFX3

- Use the historical GDAL OZI code and other public references only as **research input** for a clean-room Rust decoder.
- Or, safer operationally, support a **conversion workflow**: convert legacy OZF2/OZFX3 outside the app into a more standard raster format, then import that result.
- Keep all claims conservative: support only the older reverse-engineered variants you can regression-test with fixture files.

### Recommended wording for repo/docs

> OZF2/OZFX3 are proprietary OziExplorer raster formats with community reverse-engineered decoding for older variants, but no current maintained Rust crate and no reliable path for OZF4. For an open-source Rust app, support should be optional, read-only, and clearly best-effort; open or better-specified raster formats should be preferred.
