## Context

The Library redesign migrated the Tracks/Waypoints lists from floating panels into a persistent rail, but the agent shipping that change overlooked the entry-point affordances — `Import GPX`, `Import PLT`, `Create Track`, `Add Waypoint` — that the old `Sidebar.svelte` carried. The result: the new UI is read-only for tracks unless the user already has tracks loaded from before. This design document records the placement and wiring decisions for restoring those entry points.

## Decisions

### Decision 1 — Buttons live in the Tracks/Waypoints tab headers, not in a separate toolbar

The Tracks tab header already hosts the active-track-layer `Select`; the Waypoints tab header already hosts the active-waypoint-layer `Select`. Both selectors are scoped per tab. The corresponding actions (import / create / add-waypoint) are also per-tab, so colocating them with the selector keeps the mental model tight: "this tab's actions live in this tab's header". An alternative — a single global toolbar above the tabs — was rejected because the Maps tab has no symmetric action (the Maps Sheet is opened from elsewhere) and a global toolbar would have to switch its contents per active tab, which is harder to read than per-tab headers.

### Decision 2 — Icon-only buttons with Lucide icons at `strokeWidth: 1.5`, label via `Tooltip`

The tab header is narrow (the library rail is roughly 240-280px wide depending on shell layout). Three text buttons + a layer selector would not fit cleanly without truncation or wrapping. Icon buttons at `size="icon-sm"` (or the closest shadcn variant) keep the row compact; a shadcn `Tooltip` on hover surfaces the human label.

Icon assignments:

- `Import GPX…` — `Upload` icon, tooltip "Import GPX".
- `Import PLT…` — `Upload` icon, tooltip "Import PLT". The duplication of icon shape is intentional: both are file-import actions; the differentiation lives in the tooltip and in the file dialog's extension filter. An alternative (using `FileText` for one and `Upload` for the other) was considered and rejected because it implies the actions are categorically different — they aren't, they only differ in extension.
- `Create Track` — `Pencil` icon, tooltip "Create track". Switches to a `Check` / `Square` icon while drawing is active; see Decision 4.
- `Add Waypoint` — `MapPin` icon, tooltip "Add waypoint". Visually pressed (`variant="default"` instead of `variant="ghost"`, or `data-state="on"`) while the mode is active.

All icons use `strokeWidth={1.5}` to match the existing icon weight in the rail (`LibraryRow`, the dropdown trigger). The default `strokeWidth={2}` reads too heavy next to text.

### Decision 3 — File dialog opens via `@tauri-apps/plugin-dialog`, not a custom popover

The existing `exportGpx` / `exportPlt` flows in `TracksTab.svelte` already use `open({ save: true, ... })` from `@tauri-apps/plugin-dialog`. The matching import flow uses the same plugin with `multiple: false, directory: false` and an extension filter. There is no need for a custom popover or a Sheet — the native dialog matches Tauri convention and matches how `Sidebar.svelte` originally implemented import. The button's click handler is:

```ts
async function handleImportGpx() {
  const path = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "GPX", extensions: ["gpx"] }],
  });
  if (path) {
    try {
      await importGpx(path as string);
    } catch (err) {
      toast.error("Failed to import GPX", { description: String(err) });
    }
  }
}
```

A symmetric `handleImportPlt` differs only in the extension filter and the `importPlt` call. The `try / catch + toast.error` shape matches every other handler in `TracksTab.svelte`.

### Decision 4 — Drawing-mode flow: button toggles state and label

The legacy `Sidebar.svelte` exposed a `Create Track` button that, while active, switched to a `Done (N points)` affordance. The same surface SHALL be reproduced in the new tab header. State flow:

- When the button is clicked AND `$drawingModeActive === false`: the handler sets `drawingTrackLayerId` to `$activeTrackLayerId` (the currently selected track layer; the action is disabled if no layer is selected), calls `createEmptyTrack(layerId)` to materialise the receiving track in the domain, and flips `drawingModeActive` to `true`. The button's icon switches to `Check`; its label switches to `Done (N points)` where `N` is derived from the drawing track's live point count (the same store derivation `Sidebar.svelte` used — typically reading from the in-flight drawing-track state). The Tracks-tab import buttons are disabled while drawing is active (`$drawingModeActive`), matching the existing `disabled` on the layer `Select`.
- When the button is clicked AND `$drawingModeActive === true`: the handler flips `drawingModeActive` back to `false`, leaves `drawingTrackLayerId` set so the just-finished track is still identifiable for subsequent inspector wiring, and the icon / label revert.

The point-count source is the same store the legacy Sidebar read. If that store does not exist on the new branch, the implementation task is responsible for surfacing it from the existing drawing-state plumbing — this design does not invent a new store.

### Decision 5 — Waypoint add-mode flow: simple toggle, no extra IPC

The Waypoints-tab `Add Waypoint` button is a pure toggle on the `addWaypointMode` store. When `$addWaypointMode === true`, the button shows the pressed visual state. Map click handling for adding a waypoint at the clicked coordinate lives in `MapView.svelte` and reads `addWaypointMode` directly — this change does not touch that path. No `createEmptyWaypoint` call is needed from the button (the map click creates the waypoint via the existing handler).

### Decision 6 — Disabled states and empty layer lists

- Both import buttons SHALL be disabled when `$activeTrackLayerId === null` (no track layer exists or is selected to receive the import). The legacy Sidebar did not strictly enforce this — it would fall back to creating a default layer — but the new UI's invariant is "import targets the active layer", and surfacing a disabled state is clearer than auto-creating layers behind the user's back.
- The `Create Track` button SHALL be disabled when `$activeTrackLayerId === null` for the same reason.
- The `Add Waypoint` toggle SHALL be disabled when `$activeWaypointLayerId === null`.
- All four buttons SHALL be hidden (not just disabled) when the corresponding `trackLayers` / `waypointLayers` array is empty — matching the existing `{#if trackLayers.length > 0}` guard around the `Select` in the tab header. The header becomes empty rather than showing dead controls; the empty-state body of the tab already explains there is nothing to show.

### Decision 7 — Structural test, not behavioural test

A behavioural test of "click import, see track appear" requires the full Tauri runtime (file dialog + IPC). Per `AGENTS.md` and ADR-0024, Playwright is not acceptable evidence. The pragmatic regression guard is therefore a structural test at `src/test/library-track-import.test.ts` that reads the source of `TracksTab.svelte` and `WaypointsTab.svelte` and asserts the API symbols and store symbols appear in the file. This is cheap, fast, deterministic in Vitest, and would have caught the original regression. Real behavioural verification continues to follow `docs/agent-verification.md` against a desktop build.
