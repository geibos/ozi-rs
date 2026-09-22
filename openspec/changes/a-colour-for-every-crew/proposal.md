## Why

A day's recordings arrive as a folder of GPX from phones and navigators, and
almost none of them say what colour the track is: `gpxx:DisplayColor` is a
Garmin extension, written by Garmin's own software and by little else. Every
one of those imported as the domain's default red, so twenty crews' routes drew
one red smear on the map.

Telling the crews apart is most of what a coordinator does with a day. The slice
that taught GPX to read the colour back fixed the case where a colour was
declared; this is the common case, where none is.

## What Changes

- An imported track that declares no colour is given one from a twelve-colour
  palette, by its position among the project's tracks — so a folder import
  continues the sequence across files, and the same day imported twice looks
  the same twice.
- A track that declares a colour keeps it. GPX says so through the Garmin
  extension, PLT through its COLORREF.
- "It is the default red" is not used as the test for "declared nothing" — a
  file may genuinely declare red. The importers report what the file said, and
  the application decides.
- The palette avoids greens and pale shades, which a topographic basemap is
  made of, and puts the most distant hues first, since most days need only the
  first few.

## Capabilities

### Modified Capabilities
- `track-import`: imported tracks are told apart by colour.

## Impact

- **Backend**: `TRACK_PALETTE` and the assignment in `application/import.rs`;
  `ArchivedGpxImport` and `PltImport` report whether a colour was declared.
- **Frontend**: none.
- **Stand**: its import answers use the same palette, or it would hide the
  thing the palette exists to show.
- **Risk**: low, and reversible per track — the colour is editable in the row
  and the inspector, and it is saved in the `.ozp`.
