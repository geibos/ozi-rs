## Why

After the cleaning, every track of a search is renamed, coloured and saved by
the detachment's cartographic standard, and the marks are styled and saved
the same way. The owner's recordings of 2026-10-06 and 2026-10-08 show it
done by hand, track after track, in OziExplorer: work out the date from the
first point, transliterate the callsign, pick red or blue for the search day,
Save, type the file name to match the track name, Replace — and for marks,
pick colours the standard names, delete the address a phone wrote into each,
gather the day's marks into one file.

ozi-rs held names to a looser rule than the standard (Cyrillic and `-` after
the date passed), offered no standard colours, saved PLT one dialog per
track, wrote every mark to WPT in OziExplorer's default yellow whatever its
colour, and suggested a WPT file name the standard does not allow.

## What Changes

- Names: the warning holds a track name to п. 14–15 (`ГГГГММДД_Позывной`,
  latin letters, digits, `_`, `-`). Where a standard name can be worked out
  — the day of the first point, the group's word and number transliterated
  (`Лиса 19 Мина` → `20261008_Lisa19_Mina`) — the warning offers it and a
  click renames; «Имена по стандарту» renames every such track.
- Colours: the track colour popover offers the standard's colours (п. 20–21),
  says that black is for tasks (п. 23), and paints every visible track the
  chosen colour — a search day at once.
- Saving: «Сохранить в 10-Tracks» writes each visible track to its own PLT
  named after it in the search's `10-Tracks` (п. 24–25), asking before
  replacing files already there and before writing names not by the
  standard; two tracks with one name are refused.
- Marks: a mark's colour goes to WPT as its label background and comes back
  on import; the WPT dialog suggests `10-Tracks/Waypoints_ГГГГММДД.wpt`
  (п. 25, 29); every visible mark can be saved into one WPT (п. 31); a
  layer's notes can be cleared in one undo step; the inspector offers the
  standard's mark colours (п. 28) and flags a Russian «С» and characters п. 26
  does not allow.
- The track summary carries the first point's time (`start_time`).

## Impact

- Affected specs: `track-display`, `track-export`, `waypoints`
- Affected code: `src-tauri/src/application/{mod,commands}.rs`,
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`,
  `src-tauri/src/infrastructure/{export,import}/wpt.rs`, `src/lib/api.ts`,
  `src/lib/standard-name.ts` (new, replaces `track-names.ts`),
  `src/lib/standard-colours.ts` (new), `src/lib/track-features.ts`,
  `src/components/library/{TracksTab,WaypointsTab}.svelte`,
  `src/components/inspector/WaypointInspector.svelte`, `src/lib/i18n.ts`
- Evidence: `docs/field-notes/2026-10-06-track-processing-in-ozi.md`,
  `docs/field-notes/2026-10-08-forest-search-in-ozi.md`
