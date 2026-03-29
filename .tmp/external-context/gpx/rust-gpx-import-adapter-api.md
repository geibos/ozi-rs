---
source: docs.rs and crate source
library: gpx
package: gpx
topic: small Rust GPX import adapter API details
fetched: 2026-03-28T08:20:00Z
official_docs: https://docs.rs/gpx/latest/gpx/
---

# gpx crate import adapter notes (`gpx` 0.10.0)

## Parse from in-memory reader / byte slice

- Primary entry point: `gpx::read<R: std::io::Read>(reader: R) -> Result<Gpx, gpx::errors::GpxError>`
  - Docs: `https://docs.rs/gpx/latest/gpx/fn.read.html`
- It accepts any `Read`, so a byte slice works directly via `bytes.as_ref()` or `std::io::Cursor<&[u8]>`.
- The docs example also shows `BufReader::new("<gpx></gpx>".as_bytes())`.

Minimal shape:

```rust
use gpx::{read, Gpx};
use std::io::Cursor;

let bytes: &[u8] = ...;
let gpx: Gpx = read(Cursor::new(bytes))?;
```

## Key output types

- `Gpx`
  - `waypoints: Vec<Waypoint>`
  - `tracks: Vec<Track>`
  - `routes: Vec<Route>`
  - Docs: `https://docs.rs/gpx/latest/gpx/struct.Gpx.html`
- `Track`
  - `name: Option<String>`
  - `segments: Vec<TrackSegment>`
  - Docs: `https://docs.rs/gpx/latest/gpx/struct.Track.html`
- `TrackSegment`
  - `points: Vec<Waypoint>`
  - Important: track points are represented as `Waypoint`, not a separate track-point type.
  - Docs: `https://docs.rs/gpx/latest/gpx/struct.TrackSegment.html`
- `Waypoint`
  - used for both top-level waypoints and track-segment points
  - common fields for a minimal importer: `name`, `elevation`, `time`, `comment`, `description`
  - Docs: `https://docs.rs/gpx/latest/gpx/struct.Waypoint.html`

## Coordinate representation and access

- Coordinates are stored in `Waypoint` as a private `geo_types::Point<f64>`.
- Access through:
  - `Waypoint::point(&self) -> geo_types::Point<f64>`
  - `Waypoint::point_mut(&mut self) -> &mut geo_types::Point<f64>` (present in crate source)
- Construction uses `Waypoint::new(Point<f64>)`.
- In crate examples/source, `Point::new(-121.97, 37.24)` is used, implying `x = longitude`, `y = latitude`.
- For a deterministic importer, map as:
  - `let p = waypoint.point();`
  - `let lon = p.x();`
  - `let lat = p.y();`
- Docs/source:
  - `https://docs.rs/gpx/latest/gpx/struct.Waypoint.html`
  - source link from that page: `src/gpx/types.rs` (`point`, `new`)

## Notable limitations / gotchas

- GPX extensions are not supported by the crate README/current status.
  - Good for a small deterministic importer if you intentionally ignore extensions.
- Only GPX 1.0 and 1.1 are accepted.
  - `GpxVersion` docs: `https://docs.rs/gpx/latest/gpx/enum.GpxVersion.html`
- Track points are `Waypoint`s.
  - If your domain has separate `Waypoint` vs `TrackPoint` types, your adapter must distinguish by context.
- Many fields are optional (`Option<...>`), including `name`, `elevation`, `time`.
  - A minimal importer should avoid inventing defaults except where your domain requires them.
- `Waypoint` keeps its point field private.
  - You must use `point()` instead of accessing fields directly.
- Docs example text is slightly misleading about coordinate labels.
  - The source example prints `latitude: point.x(), longitude: point.y()`, but the construction example and normal `geo_types::Point` conventions indicate `x=lon`, `y=lat`.
  - Treat `x` as longitude and `y` as latitude.
- Error type is `gpx::errors::GpxError` and is `#[non_exhaustive]`.
  - Add a wildcard arm if pattern matching.
  - Docs: `https://docs.rs/gpx/latest/gpx/errors/enum.GpxError.html`
- Parsing can fail on invalid structure, missing required attributes, unsupported version, or out-of-bounds lon/lat.
  - Notable variants include `UnknownVersionError`, `LonLatOutOfBoundsError`, `XmlParseError`, `TrackSegmentError`.

## Tiny pseudo-mapping example

```rust
use gpx::{read, Gpx};
use std::io::Cursor;

struct ImportedTrack {
    name: Option<String>,
    segments: Vec<ImportedSegment>,
}

struct ImportedSegment {
    points: Vec<ImportedTrackPoint>,
}

struct ImportedTrackPoint {
    lon: f64,
    lat: f64,
    elevation_m: Option<f64>,
}

struct ImportedWaypoint {
    name: Option<String>,
    lon: f64,
    lat: f64,
}

fn import_gpx(bytes: &[u8]) -> Result<(Vec<ImportedTrack>, Vec<ImportedWaypoint>), gpx::errors::GpxError> {
    let gpx: Gpx = read(Cursor::new(bytes))?;

    let tracks = gpx.tracks.into_iter().map(|trk| ImportedTrack {
        name: trk.name,
        segments: trk.segments.into_iter().map(|seg| ImportedSegment {
            points: seg.points.into_iter().map(|wpt| {
                let p = wpt.point();
                ImportedTrackPoint {
                    lon: p.x(),
                    lat: p.y(),
                    elevation_m: wpt.elevation,
                }
            }).collect(),
        }).collect(),
    }).collect();

    let waypoints = gpx.waypoints.into_iter().map(|wpt| {
        let p = wpt.point();
        ImportedWaypoint {
            name: wpt.name,
            lon: p.x(),
            lat: p.y(),
        }
    }).collect();

    Ok((tracks, waypoints))
}
```

## Practical importer advice for ozi-rs

- Parse bytes with `Cursor<&[u8]>`.
- Preserve track/segment boundaries exactly.
- Convert `Waypoint` to your domain `TrackPoint` only when it comes from `TrackSegment.points`.
- Convert top-level `Gpx.waypoints` to your domain `Waypoint` collection.
- Keep absent `name`, `time`, and `elevation` as `None` unless your app requires explicit normalization.
