## Why

The detachment's cartographic standard (п. 10–13) has every foot patrol's
track filtered in OziExplorer "by index 4", and Ветра, Борты and БПЛА at the
same index when they are filtered at all. Nobody could say what index 4 is:
not the standard, not the wiki ("ВСЕГДА фильтруем через 4"), not Ozi's help,
which says only that a higher index keeps fewer points. An operator moving
from Ozi had no way to get the same result from our «Упростить», whose
default was 10 m.

The owner supplied eleven tracks from a real search on 2026-10-06, each as
the original GPX and the PLT after filtering at index 4 and some manual
cleaning. Measured against them:

- Ozi's filter only selects points: every point in each PLT is a point of
  its GPX, unmoved.
- Of 32 287 points Ozi dropped, all but 20 lie within about 2.5 m of the line
  between the points it kept; the 20 are the owner's hand-deleted outliers,
  19 to 683 m off. The cut-off is a tolerance, in metres.
- Douglas–Peucker reproduces the selection: at 1.95 m our implementation
  keeps 6671 points where Ozi kept 6672, and the two agree on 97–98 % of the
  points (a point counts as matched within two positions or three metres,
  since dense phone tracks have interchangeable neighbours). Sequential
  algorithms and plain distance thinning agree far worse.

Doing that measurement found a defect: our perpendicular distance found the
foot of the perpendicular in raw degrees. At latitude 60, where a degree of
longitude is half a degree of latitude, it overstated distances by up to
17 %, and simplification kept 5 % more points than the tolerance allowed.

## What Changes

- The perpendicular distance is found on a local plane with longitude scaled
  by the cosine of the latitude.
- The simplify dialog opens at 2 m and says that 2 m is what OziExplorer's
  filter at index 4 does; a button puts the slider back there.
- The track menu closes when «Упростить…» is chosen; it stayed open on top
  of the panel's buttons.
- Only index 4 is established. How Ozi maps other indices to metres is not,
  and nothing here claims it.

## Impact

- Affected specs: `track-editing`
- Affected code: `src-tauri/src/domain/track.rs`, `src/lib/simplify.ts`
  (new), `src/lib/stores.ts`, `src/components/library/TracksTab.svelte`,
  `src/components/inspector/TrackInspector.svelte`, `src/lib/i18n.ts`
- Evidence: `docs/field-notes/2026-10-06-track-processing-in-ozi.md`; the
  ignored test `ozi_filter_agreement` re-runs the comparison against an
  exported set of pairs kept out of the repository.
