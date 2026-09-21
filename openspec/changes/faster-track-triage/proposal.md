## Why

Triage on a search means looking at one track at a time: twenty-odd tracks are
imported from the groups' navigators onto one basemap, and the operator needs to
see each one alone to judge whether it is clean, mis-recorded or duplicated.
Today the only control is a per-row visibility toggle, so isolating one track
means hiding twenty-five others by hand and then showing them again. That is the
single slowest interaction in the tool.

The layer selector makes it worse: importing a folder creates one layer per file
named `Imported tracks: /Users/.../20260708_Veter2.gpx`, so the dropdown is a
column of identical path prefixes with the distinguishing part cut off.

## What Changes

- The Tracks tab gains show-all and hide-all controls, and a per-row "only this
  one" action that hides every other track in one step.
- Those operations are single backend commands rather than a loop of per-track
  toggles, so twenty-six rows cost one round trip and one state update.
- A layer created by an import is named after the source file rather than its
  full path.

## Capabilities

### Modified Capabilities
- `track-display`: bulk visibility operations and the isolate action.
- `track-import`: the naming rule for import-created layers.

## Impact

- **Backend**: two commands in `src-tauri/src/commands/mod.rs` with their
  application-layer counterparts; `application/import.rs` layer naming.
- **Frontend**: header controls and a row action in
  `src/components/library/TracksTab.svelte`; generated bindings; i18n keys.
- **Risk**: low. Visibility is a non-undoable style mutation already, so bulk
  changes do not touch the command stack.
