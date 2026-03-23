---
source: docs.rs and source code
library: walkers
package: walkers
topic: sources::Attribution API migration
fetched: 2026-03-23T17:40:01Z
official_docs: https://docs.rs/walkers/0.52.0/walkers/sources/struct.Attribution.html
---

# walkers 0.52.0: `sources::Attribution`

## Version confirmed

- `walkers` current docs.rs release: `0.52.0`

## Current struct definition

From `src/sources/mod.rs`:

```rust
#[derive(Clone)]
pub struct Attribution {
    pub text: &'static str,
    pub url: &'static str,
    pub logo_light: Option<egui::ImageSource<'static>>,
    pub logo_dark: Option<egui::ImageSource<'static>>,
}
```

## Related API

`TileSource::attribution` still returns `Attribution`:

```rust
pub trait TileSource {
    fn tile_url(&self, tile_id: TileId) -> String;
    fn attribution(&self) -> Attribution;

    fn tile_size(&self) -> u32 { 256 }
    fn max_zoom(&self) -> u8 { 19 }
}
```

Built-in example from `OpenStreetMap`:

```rust
fn attribution(&self) -> Attribution {
    Attribution {
        text: "OpenStreetMap contributors",
        url: "https://www.openstreetmap.org/copyright",
        logo_light: None,
        logo_dark: None,
    }
}
```

## Minimal migration guidance

- Remove `AttributionType`; it is not present in `walkers 0.52.0`.
- Remove `logo_link` and `attribution_type`; those fields no longer exist.
- Construct `Attribution` with only `text`, `url`, `logo_light`, and `logo_dark`.
- `text` and `url` are `&'static str`, so use string literals (or another `'static` string), not owned `String` values.

## Before / after shape

Old code shape that no longer matches:

```rust
Attribution {
    text: SOME_TEXT,
    url: SOME_URL,
    logo_link: None,
    attribution_type: AttributionType::Text,
}
```

Updated code shape for `0.52.0`:

```rust
Attribution {
    text: "Local SQLite tiles",
    url: "",
    logo_light: None,
    logo_dark: None,
}
```

## Notes for local tile backends

- If your type implements `walkers::Tiles`, `fn attribution(&self) -> Attribution` uses the same struct.
- Logos are now split by theme via `logo_light` and `logo_dark`.
- If you do not provide logos, set both to `None`.

## Source links used

- https://docs.rs/walkers/0.52.0/walkers/sources/struct.Attribution.html
- https://docs.rs/walkers/0.52.0/walkers/sources/trait.TileSource.html
- https://docs.rs/crate/walkers/0.52.0/source/src/sources/mod.rs
- https://docs.rs/crate/walkers/0.52.0/source/src/sources/openstreetmap.rs
