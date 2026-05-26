## Why

The archived change `2026-05-26-redesign-library-sidebar` deleted the legacy `Sidebar.svelte` (which hosted `Import GPX`, `Import PLT`, `Create Track`, and the `Add Waypoint` mode toggle) but failed to land equivalent affordances anywhere in the new `LibraryRail`. A grep for `importGpx|importPlt|createEmptyTrack` against `src/components/LibraryRail.svelte` and `src/components/library/*.svelte` returns zero matches; the `api.ts` functions still exist but nothing in the UI calls them, and `addWaypointMode` has no toggle either. This is a regression that blocks the core workflow — users currently cannot load tracks into the app or add waypoints. It must be fixed before the next release.

## What Changes

- The Tracks tab header (`src/components/library/TracksTab.svelte`) SHALL host, next to the existing track-layer `Select`, an icon-button row exposing three actions: `Import GPX…` (opens the native file dialog, then calls `importGpx(path)`), `Import PLT…` (opens the native file dialog, then calls `importPlt(path)`), and `Create Track` (toggles drawing mode by flipping `drawingModeActive` and seeding `drawingTrackLayerId` from the currently active track layer, identical to the legacy `Sidebar.svelte` logic).
- The `Create Track` button SHALL switch its label / variant while `$drawingModeActive` is true, showing `Done (N points)` derived from the drawing track's current point count (same surface the old Sidebar showed inside its drawing-mode panel) and ending drawing on click.
- The Waypoints tab header (`src/components/library/WaypointsTab.svelte`) SHALL host, next to the existing waypoint-layer `Select`, a single toggle button `Add Waypoint` that flips `addWaypointMode` with the same wiring the old Sidebar used. The button SHALL show a pressed / active visual state while `$addWaypointMode` is true.
- All three Tracks-tab actions and the Waypoints-tab toggle SHALL use Lucide icons at `strokeWidth: 1.5` (`Upload` for `Import GPX…`, `Upload` for `Import PLT…` distinguished by tooltip and label, `Pencil` for `Create Track`, `MapPin` for `Add Waypoint`). Hover SHALL surface a shadcn `Tooltip` carrying the action label, since the icon row is space-constrained.
- The Tracks-tab import buttons SHALL be disabled while `$drawingModeActive` is true (mirrors the existing `disabled` on the track-layer `Select`); the Waypoints-tab toggle is independent of drawing mode.
- No new backend IPC SHALL be introduced; this change reuses `importGpx`, `importPlt`, and `createEmptyTrack` from `src/lib/api.ts`, and the existing stores `drawingModeActive`, `drawingTrackLayerId`, `addWaypointMode`, `activeTrackLayerId`, `activeWaypointLayerId` from `src/lib/stores.ts`.
- A structural test SHALL be added at `src/test/library-track-import.test.ts` asserting that the Tracks tab's source references `importGpx`, `importPlt`, and `createEmptyTrack`, and that the Waypoints tab's source references `addWaypointMode`. This test catches a future regression of the same shape.

## Capabilities

### Modified Capabilities

- `ui-shell`: the existing Library requirement is extended with a Tracks-tab affordances clause (Import GPX, Import PLT, Create Track) and a Waypoints-tab affordance clause (Add Waypoint mode toggle). The row patterns, layer selectors, and existing tabs are unchanged.

## Impact

- **Frontend**: `src/components/library/TracksTab.svelte` and `src/components/library/WaypointsTab.svelte` gain action affordances in their tab headers; no other component is touched. `src/test/library-track-import.test.ts` is new.
- **Backend**: no change.
- **Spec evidence**: structural test in `src/test/library-track-import.test.ts` confirms the affordances exist by source-grep; manual desktop verification per `docs/agent-verification.md` confirms the buttons load tracks and toggle drawing / waypoint modes against a real project. Playwright is not acceptable per ADR-0024.
- **Risk**: low. Scope is two component files, no IPC, no schema. The only subtle wiring is the `Done (N points)` label that has to read the drawing track's live point count — the old Sidebar already had this surface and the same store derivation can be reused.

## Out of scope

- **Cmd-K palette mode chips** — the planned top-context-bar mode chips and Cmd-K palette from `redesign-inspector-pane` are unrelated; this change does NOT block their arrival. Once they land, the Tracks-tab `Create Track` button and the Waypoints-tab `Add Waypoint` toggle can be reconsidered, but for the regression window they MUST live in the Library.
- **New IPC, new file formats, new track sources.** This change only re-exposes the existing import surface.
- **Re-styling of `LibraryRow` or the tab content body.** Only the tab headers are touched.

## Dependencies

- The archived change `2026-05-26-redesign-library-sidebar` SHALL be considered the regression source: it removed the affordances without migrating them. This change does NOT modify the archived spec deltas; it adds new requirements that supersede the implicit "Library has no mode chips" stance taken by that change.
- No ordering dependency on any active change folder. The `bundle-loader-as-overlay`, `consolidate-state-event-flow`, and other open changes are independent and can land in any order relative to this fix.
