## Why

Slice 0.3 gave the transfers timeouts, so a stalled connection fails instead of
hanging for ever. What it did not give them is a second try — and the failure
of one file fails the whole bundle, after however many files had already come
down.

That is the wrong trade for the link a bundle is fetched over. A search bundle
is hundreds of megabytes across a handful of files, pulled over a tethered
phone at the edge of coverage; a dropped connection there is ordinary, not
exceptional. Losing the whole download to one of them, and starting again, is
how a crew ends up without a map.

## What Changes

- A file is fetched up to three times before the bundle gives up on it, with a
  short wait between attempts — short because the operator is standing there.
- Cancellation is never retried. It is not a transport failure, and retrying it
  would ignore the operator and keep the link busy after they asked it to stop.
- A retry says so, through the keyed progress channel, because a retry that
  looks like a stall is the same as a stall to the person watching the bar.
- A retry resumes: it asks for the rest with a `Range` request rather than
  starting the file again. Starting again is no use on the transfer that
  actually needs a retry — a connection that drops near the end of a 185 MiB
  map costs that map twice, and on a link that drops it may never land.
- The partial file is therefore kept between attempts, and removed once the
  file is given up on. A server that ignores the range answers 200 rather than
  206; then what is on disk is worthless and the file starts again, which is
  the only safe reading of that answer.

## Impact

- Affected specs: `map-bundles`
- Affected code: `src-tauri/src/infrastructure/lizaalert.rs`, `src/lib/i18n.ts`
