## Why

PLT is OziExplorer's own track format: the one a crew hands to whoever is still
running OziExplorer, and the one this application is replacing. Every piece of
the conversion is tested somewhere — the colour from its COLORREF, the segment
break flag, the altitude in feet, the OLE date — and nothing tested the whole
trip, which is what the receiver actually gets.

GPX had the same shape of gap and it hid a real loss: the colour never came
back. This checks the other format rather than assuming it is fine.

## What Changes

- A test exports a two-sitting track with a Cyrillic name, an elevation, one
  timestamp and one without, a colour and a line width, then imports the file
  and compares. It goes through the same RGB packing the export command uses,
  because a round trip that skips the caller is not the trip.
- Nothing else: it passes as written. The behaviour was already right; what
  was missing was the statement that it is.

## Capabilities

### Added Capabilities
- `track-export`: a PLT round trip is pinned.

## Impact

- **Backend**: one test in `infrastructure/export/plt.rs`.
- **Risk**: none; no production code changed.
