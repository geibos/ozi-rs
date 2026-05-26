## 1. Locate and verify the regression surface

- [x] 1.1 Confirm with `grep -n "importGpx\|importPlt\|createEmptyTrack" src/components/LibraryRail.svelte src/components/library/*.svelte` that the three API symbols are absent from the new Library tree (expected: zero matches before this change lands)
- [x] 1.2 Confirm with `grep -n "addWaypointMode" src/components/library/*.svelte` that the waypoint mode store is unwired in the Library (expected: zero matches before this change)
- [x] 1.3 Open the archived `Sidebar.svelte` via `git show <pre-removal-commit>:src/components/Sidebar.svelte` (find via `git log --diff-filter=D --name-only` for `src/components/Sidebar.svelte`) to capture the exact wiring of `Create Track` (drawing mode entry/exit, `drawingTrackLayerId` seeding, `Done (N points)` label) and `Add Waypoint` (toggle of `addWaypointMode`) so the new buttons port the same logic without regressing edge cases

## 2. Add Tracks-tab affordances

- [x] 2.1 In `src/components/library/TracksTab.svelte`, import `importGpx`, `importPlt`, `createEmptyTrack` from `$lib/api`; import `drawingModeActive`, `drawingTrackLayerId`, `activeTrackLayerId` from `$lib/stores` (the latter is already imported); import the Lucide icons `Upload`, `Pencil`, `Check` from `@lucide/svelte` / `lucide-svelte` matching the project's existing import path
- [x] 2.2 Add `handleImportGpx`, `handleImportPlt`, and `handleCreateTrackToggle` async functions next to the existing `handleExportGpx` / `handleExportPlt` handlers; follow the same shape (await `open({ ... })` from `@tauri-apps/plugin-dialog`, then await the API call, wrap in try/catch with `toast.error`)
- [x] 2.3 In the tab header (`<header class="border-border border-b px-2 py-1.5">` block), add a flex row beside the existing `Label + Select` group containing three shadcn `Button` instances with `variant="ghost"` and `size="icon-sm"` (or the closest existing icon-button size in the project's `button` primitive — confirm via `src/lib/components/ui/button/`), each wrapped in a shadcn `Tooltip` with the labels `"Import GPX"`, `"Import PLT"`, and `"Create track"` (or `"Finish track"` while drawing is active)
- [x] 2.4 Wire `disabled={$drawingModeActive || $activeTrackLayerId === null}` on the import buttons; wire `disabled={$activeTrackLayerId === null}` on the Create Track button; ensure the existing `disabled={$drawingModeActive}` on the layer `Select` remains intact
- [x] 2.5 Hide the entire icon-button row inside the same `{#if trackLayers.length > 0}` guard that wraps the `Label + Select`
- [x] 2.6 Implement the drawing-mode visual switch: when `$drawingModeActive === true`, swap the `Pencil` icon for `Check` on the Create Track button and surface a `Done (N points)` text label adjacent to the icon. Source the point count from the same store the legacy Sidebar consumed — locate it via the `git show` recovery in task 1.3 and import it from `$lib/stores`
- [x] 2.7 In `handleCreateTrackToggle`, on entry (when `$drawingModeActive === false`) call `drawingTrackLayerId.set($activeTrackLayerId)`, then `await createEmptyTrack($activeTrackLayerId)`, then `drawingModeActive.set(true)`. On exit (when `$drawingModeActive === true`) call `drawingModeActive.set(false)`. Wrap the entry path in try/catch with `toast.error`

## 3. Add Waypoints-tab affordance

- [x] 3.1 In `src/components/library/WaypointsTab.svelte`, import `addWaypointMode` from `$lib/stores`; import the Lucide `MapPin` icon
- [x] 3.2 In the tab header, add a shadcn icon `Button` (or `Toggle` primitive if the project already uses it; confirm via `src/lib/components/ui/`) next to the existing `Label + Select` group; wrap in a shadcn `Tooltip` with label `"Add waypoint"`
- [x] 3.3 Wire the click handler to flip `addWaypointMode` via `addWaypointMode.update((v) => !v)`; bind the pressed visual state to `$addWaypointMode` (via `variant="default"` when true, `variant="ghost"` when false, or via a `data-state` attribute the chosen primitive understands)
- [x] 3.4 Wire `disabled={$activeWaypointLayerId === null}` on the button
- [x] 3.5 Hide the button inside the existing `{#if waypointLayers.length > 0}` guard

## 4. Structural regression test

- [x] 4.1 Create `src/test/library-track-import.test.ts` using the same Vitest harness the project's other `src/test/*.test.ts` files use (consult `vitest.config.ts` or `vite.config.ts` to confirm the test glob covers `src/test/**/*.test.ts`)
- [x] 4.2 Read the source of `src/components/library/TracksTab.svelte` via `fs.readFileSync` and assert that the resulting string contains the substrings `importGpx`, `importPlt`, `createEmptyTrack`, and `drawingModeActive` — each as its own `expect(source).toContain(...)` assertion so a failure points at the missing symbol
- [x] 4.3 Read the source of `src/components/library/WaypointsTab.svelte` and assert that the resulting string contains the substring `addWaypointMode`
- [x] 4.4 Run `just test` (or the project's Vitest invocation) and confirm the new test file passes alongside the existing suite

## 5. Verification

- [x] 5.1 Run `just lint` and `just clippy` — there should be no new warnings introduced by these UI changes (clippy will be a no-op since `src-tauri/` is untouched; lint covers the Svelte/TS side)
- [x] 5.2 Run `just check` to confirm the Svelte / TS type-check still passes against the modified component sources
- [x] 5.3 Run `just test` to confirm the structural test passes and no existing tests regressed
- [x] 5.4 Run `just ci` to gate the whole package together (this re-runs clippy + check + lint + test)
- [ ] 5.5 Follow `docs/agent-verification.md` to launch the desktop build, open the seeded test project, and manually confirm: Import GPX surfaces the file dialog and loads a track; Import PLT does likewise; Create Track enters drawing mode and the icon/label switch to `Check + Done (N points)`; clicking again exits drawing mode; Add Waypoint toggles the pressed state and clicking the map subsequently creates a waypoint. Playwright is NOT acceptable evidence per ADR-0024 — DEFERRED: requires interactive desktop QA session, to be performed by the reviewer before merge

## 6. OpenSpec validation

- [x] 6.1 Run `openspec validate fix-library-track-import --strict` and resolve any complaints (missing scenarios, malformed deltas, unreferenced capabilities) before opening the PR
- [x] 6.2 Confirm via `openspec list` that the change appears in the active list and is not accidentally archived
