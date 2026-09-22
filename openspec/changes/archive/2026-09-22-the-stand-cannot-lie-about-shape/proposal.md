## Why

The stand's README says a command with no answer throws loudly, "because a
screen that renders because a mock quietly returned `undefined` is the failure
this whole exercise exists to stop". A mock that returns the *wrong shape* is
that same failure wearing a hat, and it happened twice in two days:

- `export_all_tracks_gpx` answered "accepted" where the caller reads two
  counts, so the Tracks tab threw on `.tracks`.
- `get_simplified_preview` answered `{points, removed}` where the DTO is
  `{original_count, simplified_count, segments}`, so opening the simplify
  dialog threw inside `MapView` and showed two empty numbers.

Both were found by opening a screen. The project's own rule is that the third
fix is over the shape, and here the shape is a type: the answers are typed
against the generated bindings, which is the only check that runs every time.

## What Changes

- The stand's handler table is typed as `StandAnswers`: for each command the
  bindings declare, the answer's type unwrapped from the generated `Result`,
  keyed by the wire's snake_case name.
- The tile commands are the stated exception — their binding says `number[]`
  and the transport hands the app an `ArrayBuffer`, as the real IPC does.
- Four answers that the compiler found were loose are now exact: the app state
  is an `AppStateDto` rather than `unknown`, the previewed project carries the
  whole project rather than a slug in an otherwise empty object, a missing
  track's fallback id is a number, and the rows an import adds are
  `TrackSummaryDto` rather than `Record<string, unknown>`.

## Capabilities

### Modified Capabilities
- `build-tooling`: the stand cannot answer with a shape the app does not
  expect.

## Impact

- **Stand**: `src/test/stand/tauri-core.ts`.
- **Product code**: none.
- **Risk**: none at runtime; the change is entirely in types, and the stand
  serves the same screens it did.
