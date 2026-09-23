## Why

CJ-8 is two headquarters passing a `.ozp` back and forth, and the file carried
nothing that said which build wrote it. The audit called this a
forward-compatibility risk and it is the sharp kind: an older build opening a
newer file reads the parts it understands, silently drops the rest, and writes
that loss back over the other штаб's work the moment the operator saves.

The whole compatibility story so far has been "an older file still reads",
which serde gives for free and a test pins. The other direction had nothing.

## What Changes

- A saved `.ozp` carries `format_version`. The envelope lives in the
  persistence layer, not on `Project`: a format version is a fact about a
  file, not about a search.
- A file with no version is every project written before this, and opens
  unchanged — `flatten` keeps the JSON exactly the shape it had.
- A file whose version is newer than this build understands is refused, with a
  message that says why. Refusing is the only honest answer until there is a
  migration; degrading silently is what loses the other headquarters' work.

## Impact

- Affected specs: `project-persistence`
- Affected code: `src-tauri/src/infrastructure/persistence.rs`
