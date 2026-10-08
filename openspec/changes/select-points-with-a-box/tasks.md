## 1. The edit

- [x] 1.1 Tests: delete the chosen, keep only the chosen, one undo each;
      removing every point refused
- [x] 1.2 `apply_remove_track_points` over the crop plumbing,
      `remove_track_points`, bindings, commands reference

## 2. The tool

- [x] 2.1 Tests for `box-select.ts`: the box either way, points inside it,
      Shift adds, the tool takes the drag from the other tools
- [x] 2.2 `BoxSelect.svelte` mounted by the map; «Рамка», palette entry,
      Delete and Esc; component test against a stand-in map

## 3. Gates

- [ ] 3.1 `just ci` green
- [x] 3.2 Walked on the stand 2026-10-08: a box over the left half chose 2
      points, «Удалить выбранные» removed them (5 → 3 points, toast), Esc
      put the tool away. The walk's first drag started on the zoom control and
      chose nothing — the tool is not at fault, the walk was. The bar's count
      wrapped onto two lines; kept on one. `docs/progress/2026-10-08-box-select/`
- [ ] 3.3 In the packaged application on a real track — with the batched
      smoke
