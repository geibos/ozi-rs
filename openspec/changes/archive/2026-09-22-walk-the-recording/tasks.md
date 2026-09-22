## 1. Frontend

- [x] 1.1 The track's points flattened in order across its segments
- [x] 1.2 Previous / next, disabled at the ends, with the position beside them
- [x] 1.3 Stepping centres the map on the point, reusing the coordinate-carrying focus request
- [x] 1.4 From nothing selected, next goes to the first and previous to the last

## 2. The trap this hit on the way

- [x] 2.1 `let trackDetail: TrackDetail | null = $state(null)` failed type-checking the moment a `$derived` read it, with "Property 'segments' does not exist on type 'never'" — the same trap recorded in `docs/backlog.md` two slices earlier, in a different file
- [x] 2.2 Converted here and in the four other declarations that carried it latently
- [x] 2.3 A guard test, scoped to `$state(null)` — an array narrows to `never[]`, which is assignable and harmless, and a guard that flagged it would be one somebody turns off
- [x] 2.4 Verified red by putting one declaration back

## 3. Evidence

- [x] 3.1 Walked on the stand: nothing selected, then "1 из 5", "3 из 5", back to "2 из 5", crossing the segment boundary

## 4. Gates

- [x] 4.1 `just ci` green
- [x] 4.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
