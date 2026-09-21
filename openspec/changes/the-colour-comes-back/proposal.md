## Why

The GPX writer says what colour a track is — `gpxx:DisplayColor`, the Garmin
extension every navigator understands. The reader never heard it, because the
`gpx` crate does not surface extensions at all. So every track this application
exported and read back came in the default red.

Three navigators' tracks on one map, all red, is one smear instead of three
routes, and telling them apart is most of what a coordinator does with a day's
recordings. The gap was known and pinned by a test that asserted the loss, so
that it was a recorded fact rather than a surprise; the owner has said FTP will
carry track upload, which makes what survives a round trip matter more than it
did.

## What Changes

- Reading a GPX walks the document a second time for `gpxx:DisplayColor` and
  matches the colours to tracks by position. A track that declares none keeps
  its default, so a colourless track between two coloured ones does not take
  the next one's.
- One colour table serves both directions. Two would drift, and the drift
  would show as a track that changes colour on a round trip.
- A name the table does not hold — another program's extension, or a typo —
  leaves the track its default rather than guessing.
- A document the second pass cannot read yields no colours. By then the `gpx`
  crate has parsed the same bytes successfully, so a disagreement costs the
  colour, never the import.
- The pass uses `xml-rs`, the parser `gpx` itself uses. It was already in the
  dependency tree; the lockfile change is one line moving it from transitive
  to direct, and `cargo audit` is unchanged.

## Capabilities

### Modified Capabilities
- `track-export`: the round trip keeps the track's colour.

## Impact

- **Backend**: `infrastructure/import/gpx.rs` (second pass),
  `infrastructure/export/gpx.rs` (shared table, `garmin_color_to_rgba`),
  `Cargo.toml`.
- **Frontend**: none.
- **Risk**: low. The colour is applied only where one is declared and
  recognised; every other path is exactly as it was.
