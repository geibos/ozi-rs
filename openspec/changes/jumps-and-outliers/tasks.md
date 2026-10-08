## 1. Measuring

- [x] 1.1 Ground truth from the owner's eleven pairs: where he split, which
      in-segment points he deleted; legs measured at each
- [x] 1.2 Thresholds chosen against it, and the TypeScript detector
      re-scored on the same tracks: 26/32 breaks (one more already a GPX
      break), 15/20 deletions, 112 entries over eleven tracks

## 2. The command

- [x] 2.1 Tests: cut out a middle point; undo restores it; ends refused
- [x] 2.2 `CutOutTrackPoint` and its reverse, `apply_cut_out_track_point`,
      the Tauri command, bindings, commands reference

## 3. The detector and the table

- [x] 3.1 Tests for `track-jumps.ts`: spike, gap, neighbours not flagged,
      segments judged apart, no times
- [x] 3.2 Distance and speed under each row of the points table
- [x] 3.3 The list in the inspector, with its actions; component test

## 3a. Breaks (2026-10-08)

- [x] 3a.1 A pause of six hours or more is a break: the owner's 2026-10-08
      recording cut tails from 26 September and 2 October off two navigators;
      his eleven tracks of 2026-10-06 never paused for more than twenty
      minutes inside a search. Tests, the list entry, trimming either side

## 4. Gates

- [x] 4.1 `just ci` green (2026-10-06: 427 Rust, 741 vitest, matrix 64/64)
- [x] 4.2 Walked on the stand (`?track=dirty`), 2026-10-06: the list holds
      the outlier at point 20 (355 m · 319 km/h → 345 m · 311 km/h) and the
      jump at point 41; choosing the outlier selects it and centres the table
      on it; «Удалить и разделить» leaves segments of 19 and 40 points and the
      jump; «Разделить здесь» empties the list. The walk found the table not
      following the selection (fixed: centred, expanding a segment past its
      first thousand points). Undo is not walked: the stand keeps no history;
      one undo step is the Rust test's. `docs/progress/2026-10-06-jumps-and-outliers/`
- [x] 4.3 The matrix: `track-jumps` added, `track-inspector` updated. The
      first runs disagreed with themselves twice over, both times the matrix
      clicking too early: an entry in the inspector while it re-rendered
      (Playwright's retry scrolled the inspector column), and before the map
      had taken up its opening view (which then overrode the flight to the
      point). Clicks now wait for their target to hold still, and a `settle`
      step waits for the whole screen
- [ ] 4.4 (2026-10-08: the smoke journey on the owner's Лиса 2 reached the
      track in the packaged application and failed choosing an outlier by
      its text; entries now have an accessible name, journey not re-run)
      In the packaged application with one of the owner's real tracks:
      the list, «Удалить и разделить», one ⌘Z — with the batched smoke
