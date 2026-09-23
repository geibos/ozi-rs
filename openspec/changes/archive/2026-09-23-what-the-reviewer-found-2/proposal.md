## Why

An outside reviewer (`gpt-6-astra`) read the six days of work in
`31d92cb..HEAD`. Eight findings survived checking, and three of them were in
the code written to put track names on the map — one of which was visible in
the screenshot taken to prove that feature worked, and read as "the middle of
the track" rather than as the defect it is.

Separately, the measuring tool was still half of what OziExplorer's is: the
original calls it "Distance & Area", and the area is the half a coordinator
writes down. A sector is handed to a crew as "прочесать 2.4 км²", and that
number decides how many people it takes.

## What Changes

Track names, all three found by the reviewer:

- A name goes half way along the track's **longest segment**, not half way
  along the flattened track. A track is split where the recording stopped, and
  half the flattened length can fall in the gap between two stretches a
  kilometre apart — where there is no line to label.
- Which name survives a crowd is decided by **kilometres walked**, not points
  logged. Ranking by point count let a navigator logging once a second at a
  rest stop beat a long route logged once a minute.
- Names are kept apart by their **text boxes**, not their centres. Two long
  names sixty pixels apart cleared a forty-eight pixel centre test and
  overlapped anyway.
- And one the reviewer did not name: the comparator read
  `Number(undefined)` for an unselected track, which is `NaN`, so the sort did
  nothing at all whenever no track was selected. The "selected first" rule
  passed its own tests by luck.

Elsewhere:

- Renaming a layer that does not exist is an error. It answered `Ok`, so the
  command stack cleared the redo history and counted a mutation for something
  that never happened.
- The `.ozp` version is read on its own, before the rest is parsed. Reading
  the whole file first meant a future format that moved a field was reported
  as "format error" rather than as coming from a newer build — the diagnosis
  failing in exactly the case it exists for.
- A `.wpt` that declares a datum other than WGS 84 says so. Transforming
  between datums is a non-goal; taking the coordinates in silence puts the
  other headquarters' marks 100–150 m from where they meant them.
- The measuring tool reports the enclosed area from the third point on.
- Two claims are corrected rather than defended: the restore tests said they
  proved the map was restored when they proved the restore reached it, and the
  `chr(209)` export reasoning rested on something nobody has checked.

## Impact

- Affected specs: `track-display`, `product-scope`, `undo-redo`,
  `project-persistence`, `track-import`
- Affected code: `src/lib/track-labels.ts`, `src/lib/geo.ts`,
  `src/components/MapView.svelte`, `src/lib/i18n.ts`,
  `src-tauri/src/application/commands.rs`, `src-tauri/src/application/mod.rs`,
  `src-tauri/src/application/import.rs`,
  `src-tauri/src/infrastructure/import/wpt.rs`,
  `src-tauri/src/infrastructure/export/wpt.rs`,
  `src-tauri/src/infrastructure/persistence.rs`
