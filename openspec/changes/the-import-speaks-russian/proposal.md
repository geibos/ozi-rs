## Why

Importing a folder is how a day's recordings arrive — the field archive is a
directory of per-date subfolders — and the command answered with an English
sentence that the Tracks tab put straight into a toast. A Russian crew finished
the most common import there is and read "Imported 12 tracks and 3 waypoints
from 4 files".

The project has already decided this question once. `progress-in-the-crews-
language` moved the bundle progress to a key and its arguments, and its note
said the remaining English lived in status messages "which no surface currently
renders — convert them the same way if one starts to". This one was rendered
the whole time.

It was invisible because the stand could not walk any flow behind a file
dialog: every dialog answered "cancelled". That is fixed too.

## What Changes

- `import_tracks_directory` returns counts — files, tracks, waypoints, and the
  names of the files it could not read — instead of a sentence.
- The Tracks tab words it: «Импортировано: треков 3, точек 2, из файлов 4».
- A folder where one navigator's file could not be read imported the rest, so
  it is a success with a caveat rather than a failure: the names appear as the
  toast's description.
- The skipped files are named, not pathed. A toast has no room for a directory
  tree and the file name is what an operator recognises.
- The stand answers the import commands, so the flow can be looked at.

## Capabilities

### Modified Capabilities
- `track-import`: a folder import reports in the operator's language.

## Impact

- **Backend**: `import_tracks_directory` returns `ImportReportDto`;
  `application::import` is `pub` so the report type is nameable.
- **Frontend**: `api.ts`, `TracksTab.svelte`, two i18n keys.
- **Stand**: answers for `import_gpx`, `import_plt`,
  `import_tracks_directory` and `load_project_file`, with imported rows
  appearing in the list.
- **Risk**: low. One command's return type; its only caller is the Tracks tab.
