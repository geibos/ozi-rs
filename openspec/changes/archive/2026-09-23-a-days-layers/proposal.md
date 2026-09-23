## Why

A day of recordings arrives as files, and every file becomes a layer: the
importer creates `Imported tracks: <path>` and, when the GPX carries `<wpt>`
elements, `Imported waypoints: <path>` beside it. Twenty crews is twenty
layers, named after paths nobody chose, and there is no command in the IPC
registry that creates, renames or removes one. The domain has had
`AddTrackLayer` and `RemoveTrackLayer` since the first commit; nothing ever
exposed them. So the operator can toggle a layer's tracks and nothing else:
not tidy the names, not throw away the folder they imported by mistake, not
start an empty layer to draw a plan into.

Opening that up first needs a defect fixed. `RemoveTrackLayer` reverses to
`AddTrackLayer { id, name }`, which creates an **empty** layer. The forward
command carries the whole layer, tracks included; the reverse throws them
away. Deleting a layer of forty tracks and pressing Ctrl+Z would hand back an
empty layer and lose the day. It has never fired because nothing could reach
it — which is the only reason this is a fix and not an incident.

Renaming has no command at all.

## What Changes

- Removing a layer is undoable **with its contents**. `RestoreTrackLayer` and
  `RestoreWaypointLayer` put the layer back as it was, and are what a removal
  reverses to.
- `RenameTrackLayer` and `RenameWaypointLayer` join the command set, undoable
  like every other edit.
- Six commands reach the frontend: create, rename and delete, for track layers
  and for waypoint layers. Creating takes a name and answers with the new id,
  so the caller can make it active without a round trip through the state.
- Deleting the layer that is active moves the active layer to another one
  rather than leaving the drawing tools pointed at nothing.
- Both library tabs grow the controls: a layer's row menu renames and deletes
  it, and a "new layer" entry beside the active-layer select creates one.

## Impact

- Affected specs: `layers`, `undo-redo`
- Affected code: `src-tauri/src/application/commands.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`, `src/lib/bindings.ts`, `src/lib/api.ts`,
  `src/lib/i18n.ts`, `src/components/library/TracksTab.svelte`,
  `src/components/library/WaypointsTab.svelte`
