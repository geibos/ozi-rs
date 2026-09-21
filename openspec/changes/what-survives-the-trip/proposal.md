## Why

FTP upload of tracks is planned, and the file that goes up is a GPX this app
writes. Both halves of that were tested apart — the writer's XML, the reader's
parse — and nothing checked that a track handed to one comes back from the
other.

For a search record that matters in specifics: the name, including Cyrillic;
the break between two sittings, because a track joined across the gap claims a
straight line the crew never walked; every coordinate in order; and the times,
which are what make a track a timeline rather than a shape.

## What Changes

- A test takes a two-segment track with Cyrillic name and timestamps on one
  segment only, writes it, reads it back and compares. It passes: all of the
  above survives.
- A second test pins what does not. The writer emits the track's colour as
  `gpxx:DisplayColor` and the reader drops it — the `gpx` crate does not
  surface extensions at all, so reading one needs a second pass over the XML.
  A track that leaves and comes back is the default red.

Nothing about the track is lost by that: the colour lives in the `.ozp` project
file, and GPX is the interchange format, not the record. The test exists so the
gap is a fact rather than a surprise, and it fails when someone closes it.

The first version of that second test asserted against red, which is the
default colour — so it would have passed whether or not the colour survived,
and recorded a fact that was not one. It uses blue.

## The other two formats

Surveyed rather than assumed, since the point of this is to know:

- **PLT**, which is what OziExplorer itself reads, already round-trips — name
  in Cyrillic, colour and width included. Better covered than GPX was.
- **WPT** has no importer, deliberately: `product-scope` puts WPT export in the
  MVP and WPT import outside it. Its output is pinned line by line instead —
  exact v1.1 header, CP1251, CRLF, six decimal places.

## Impact

- Affected specs: `track-export`, `waypoints`
- Affected code: `src-tauri/src/infrastructure/export/gpx.rs` (tests only)
