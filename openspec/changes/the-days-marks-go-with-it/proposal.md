## Why

"Выгрузить всё…" writes every track in the project to one GPX and stops there.
A day's work is not only where people walked: it is the marks they made — the
штаб, the заброс, the point where something was found. Those live in the
waypoint layers and were left out of the handover, so a crew had to export
twice, from two tabs, and hand over two files.

GPX holds `<wpt>` and `<trk>` in the same document. There was never a reason
for two.

FTP upload is coming and it will ask for "the day" as one thing, which is the
same question this answers.

## What Changes

- The day's export writes the marks alongside the tracks, waypoints first as
  every writer of the format puts them.
- It says what it wrote: "Выгружено: треков 3, точек 3".
- A project of marks and no tracks now exports, because that is a real state —
  the штаб's own project before anyone has walked anywhere — and it has
  something worth handing over. Only a project with neither is refused.
- The stand can answer a file dialog with a path instead of always cancelling.
  Cancelling every time meant no flow behind a dialog — import, export,
  save-as, open a project — could be looked at on the stand at all.

## Capabilities

### Modified Capabilities
- `track-export`: the day's export carries the marks as well as the tracks.

## Impact

- **Backend**: `export_day_to_gpx_file` replaces `export_tracks_to_gpx_file`,
  `export_all_tracks_gpx` returns counts of both, one new DTO.
- **Frontend**: `api.ts`, `TracksTab.svelte`, three i18n strings.
- **Stand**: `tauri-dialog.ts` gains `standAnswerDialogsWith`;
  `export_all_tracks_gpx` answers with counts rather than "accepted".
- **Risk**: low. A receiver expecting only tracks gets a document that also
  holds marks, which every GPX consumer already handles.
