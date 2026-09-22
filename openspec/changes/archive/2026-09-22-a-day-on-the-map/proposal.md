## Why

The palette handed to imported tracks was designed by reasoning: avoid greens
and pale shades, because a topographic basemap is made of them. Nothing had
looked at twelve of those colours drawn over that basemap, because the stand
served imported tracks to the list and not to the map — and the map is where a
day of recordings is actually read.

A stand that shows a list but no routes cannot answer the only question a
palette raises.

## What Changes

- Imported tracks reach the map: the stand gives each one a route, and adds
  the rows to `AppStateDto.tracks` as well as to `list_tracks`. The second part
  is why they were invisible — `MapView` redraws off a fingerprint taken from
  the state, so an import that left the state alone appeared in the list and
  nowhere else.

## Capabilities

### Modified Capabilities
- `build-tooling`: the stand draws what an import produced.

## Impact

- **Stand**: `src/test/stand/tauri-core.ts`.
- **Product code**: none.
- **What it showed**: twelve routes over the topographic basemap are
  distinguishable from each other and from the ground. The two browns are the
  closest pair and sit far apart in the sequence; cyan over the river hatching
  is the weakest, and still reads at line width 3.
