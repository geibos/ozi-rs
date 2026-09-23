## 1. The names on the map

- [x] 1.1 Anchor on the longest segment; `segmentsOf` keeps the parts apart
- [x] 1.2 Rank by kilometres walked, not points logged
- [x] 1.3 Declutter by text box, not by anchor distance
- [x] 1.4 The `NaN` comparator that made the sort a no-op
- [x] 1.5 A test for each, red before the fix

## 2. Elsewhere

- [x] 2.1 `Rename*Layer` on a missing layer is an error, with a redo test
- [x] 2.2 The `.ozp` version is read on its own, with a test on an unparseable future file
- [x] 2.3 A `.wpt` in another datum warns, through the existing datum check
- [x] 2.4 Area geometry in `geo.ts`, tested at 60° north where a longitude degree is half a latitude one
- [x] 2.5 Owner, mid-turn: length and area are separate tools — an area quoted for an open path is a meaningless number in a place people read numbers
- [x] 2.6 The area outline is drawn closed, so the shape agrees with the number
- [x] 2.7 The radius ring reports the ground it covers beside its radius

## 3. Claims corrected rather than defended

- [x] 3.1 The restore tests said "the map was restored"; they prove it was reached
- [x] 3.2 The `chr(209)` export reasoning records what nobody has checked

## 4. Gates

- [x] 4.1 `just ci` green
- [x] 4.2 Seen on the map 2026-09-23 (`docs/progress/2026-09-23-reviewer-round-2/distance-and-area.png`): a four-point sector reads «5.79 км · 3.37 км²», and «20260708_Veter2» sits on a drawn stretch rather than in the gap it used to occupy
