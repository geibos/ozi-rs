## Why

Importing a folder of navigators makes one track layer per file, which is what
a day of searching looks like: twenty-odd layers. Handing that to the штаб
meant opening the export dialog once per layer — the same twenty-six-clicks
shape the visibility toggles had before bulk controls.

FTP upload is coming, and it will need "the day's tracks" as one thing anyway.

## What Changes

- One action exports every track in the project to a single GPX file, and says
  how many it wrote.
- An empty project is refused rather than writing a file that looks like a
  day's work and contains nothing.

## Capabilities

### Modified Capabilities
- `track-export`: exporting the whole project in one step.

## Impact

- **Backend**: `export_tracks_to_gpx_file` in the export layer,
  `export_all_tracks_gpx` in the application layer, one command.
- **Frontend**: a button in the Tracks tab header; i18n.
- **Risk**: low. Existing per-layer and per-track exports are untouched.
