## Why

The launch screen's status bar is a fixed 80px of four rows, three of which are
empty whenever nothing is downloading — which is whenever the application has
just been launched. One of the empty rows is an outlined progress track with no
fill, which reads as a broken widget rather than as a reserved space.

The reservation exists so the bar does not jump when a download starts. Paying
for that with permanent dead space on the first screen a crew sees is the worse
half of the trade, and the jump it avoids is a single growth at the moment the
operator asked for a download.

## What Changes

- The bar is one line — the status line — until there is a download to report
  on, then it grows to its four rows and shrinks back when the download ends.
- The growth is a height transition, so it reads as a panel opening rather than
  as a jump.
- The empty progress track is gone when there is nothing to show. A download
  that has not reported bytes yet keeps the row, so the bar does not resize
  twice in a second.

## Capabilities

### Modified Capabilities
- `map-bundles`: the launch screen's status bar is sized to what it has to
  say.

## Impact

- **Frontend**: `src/routes/+page.svelte` (markup and styles).
- **Backend**: none.
- **Risk**: low; the bar shows exactly what it showed, in less space when it
  has nothing to show.
