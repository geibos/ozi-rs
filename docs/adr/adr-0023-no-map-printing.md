# ADR-0023: Map Printing Is Not in Scope

- Status: accepted
- Date: 2026-04-28

## Context

`README.md` and the project roadmap previously listed "print map with tracks and
waypoints to PDF or image" as remaining work. During MVP scope brainstorming the
user confirmed map printing is not planned: SAR volunteers do not currently rely
on printed maps inside their workflow, and the operational format is digital
delivery to handheld navigators (GPX/PLT/WPT — see ADR-0022).

The decision to not implement printing is deliberate, not deferred. Without an
explicit ADR, future agents would re-discover the README mention and propose the
work again.

## Decision

Map printing (to PDF, image, or any other rendered output) is not part of MVP and
is not planned post-MVP. Remove it from `README.md` "Remaining Work" and
`docs/roadmap.md`.

If future user requirements re-introduce the need, a successor ADR may revisit
this decision. At that point the ADR must include:

- the user workflow that requires print output,
- the chosen output format(s) and resolution constraints,
- the implementation surface (browser print, server-side rasteriser, native print
  pipeline, or PDF library), and
- a justification for why digital handoff to navigators no longer suffices.

## Consequences

### Positive

- The roadmap and README converge on the actual product direction.
- Future agents do not re-propose printing as "obviously needed" — the decision
  is recorded and citable.
- Implementation work avoids the integration cost of a print pipeline (page
  layout, scale bars, attribution, multi-page tiling, raster vs. vector output).

### Negative

- Volunteers who occasionally need a paper map for briefings must use external
  tools (screenshot, browser print) until this ADR is revisited.

## Related

- ADR-0020 — MVP scope (records this exclusion)
