## 1. Measuring

- [x] 1.1 Eleven pairs from a real search (2026-10-06): subset check, the
      dropped-point deviation, Douglas–Peucker against sequential filters;
      written up in the field notes
- [x] 1.2 `ozi_filter_agreement`, an ignored test that re-runs the comparison
      on our own implementation from an exported set of pairs

## 2. The distance

- [x] 2.1 Test: 100 m off a diagonal at latitude 60 measures 100 m (was 116.6)
- [x] 2.2 The foot of the perpendicular on a cos-latitude plane

## 3. The panel

- [x] 3.1 One constant for the index-4 tolerance, used by both ways into the panel
- [x] 3.2 The hint and the button, in both dictionaries
- [x] 3.3 Component test: the panel opens at 2 m, the button returns to it

## 4. Gates

- [x] 4.1 `just ci` green (2026-10-06: 424 Rust, 726 vitest, 60 shots; the panel is not in the baseline)
- [x] 4.2 Walked on the stand 2026-10-06: opens at «Допуск: 2 м» with the hint; at 10 m «Индекс 4» appears and puts it back to 2 m. The walk found the track menu staying open after «Упростить…», on top of the panel — a `preventDefault` left over from when the panel lived inside the menu; removed. `docs/progress/2026-10-06-filter-like-ozi/`
