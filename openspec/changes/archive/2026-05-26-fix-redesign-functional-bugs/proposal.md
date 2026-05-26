## Why

User smoke after the redesign trilogy surfaced five functional regressions:

1. JSON parse/stringify errors in a toast on (a) cold-start session restore and (b) project selection in the Library Maps tab.
2. Persisted last-map opens at canvas top-left at min-zoom instead of fit-to-bounds.
3. Bundle download progress is invisible in the Library Maps tab — per-row state never flips while files land.
4. Double-click "Maps…" freezes the UI ~5s while a second `load_projects` IPC serialises behind the first.
5. The Cmd-K trigger renders like a text input; the user typed into it and got no response.

Each one blocks a documented user path, and the IPC error toast (1) carried only `[object Object]` text. This change adds a dev-only structured error surface alongside the targeted fixes so the next regression is visible in seconds, not in a smoke session.

## What Changes

- A dev-only error toast SHALL render the full structured error text whenever an IPC call rejects, carrying `data-testid="ipc-error"` so smoke can assert on it. The toast SHALL be additive — existing `toast.error("Failed to …", { description })` call sites stay, the dev toast augments them with the raw error payload.
- The session-restore path on cold-start SHALL drive the active map through the same `register_active_map_layer` flow that a user-initiated `open_selected_map` takes, so the frontend `MapView` receives a viewport hint and `fitBounds` runs against the calibrated OZI bounds. The route-level redirect to `/project` SHALL not race the metadata fetch.
- The Library Maps tab (`src/components/library/MapsTab.svelte`) SHALL consume `downloadingMaps` and `downloadProgress` from `src/lib/stores.ts` and render an in-progress indicator (percentage or progress badge) on each row whose bundle is currently downloading. The "cached" badge remains driven by `m.downloaded`.
- The status bar in the workspace SHALL render the active bundle's `bundleProgress` text whenever a download is in flight, mirroring what the old full-route bundle loader showed at `src/routes/+page.svelte:91-138`. This restores the global progress affordance the redesign accidentally dropped.
- The "Maps…" button in the Library Rail (`MapsTab.svelte:48-51`) and the "Refresh" path in the bundle loader SHALL short-circuit when `$busy` is already true: no second `load_projects` IPC, no UI freeze. The button SHALL also visibly reflect the busy state.
- The Cmd-K trigger in `WorkspaceShell.svelte:101-109` SHALL be styled and labelled so it cannot be mistaken for a text input — no caret, no focus-ring that mimics inputs, no placeholder-shaped label. It SHALL read as a button labelled "Open command palette" with the `⌘K` chord shown as the key affordance, and on activation it SHALL open the dialog whose internal `cmdk` input takes focus.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `ui-shell`: dev IPC-error toast surface; Cmd-K trigger styling discipline; status-bar bundle-progress restoration; Library Maps tab consumes download progress and serialises re-entry.
- `lizaalert-integration`: Library Maps tab rows reflect in-progress bundle download state, not only `downloaded: true/false`.
- `project-persistence`: session-restore on cold-start exposes the restored active map through the same active-map registration flow as a user-initiated open, so viewport restore matches a click-open.
- `map-bundles`: bundle-download progress is observable from every surface that lists the maps of the currently downloading project, not only from inside `BundleLoader`.

## Impact

- **Frontend**: error toast plumbing in `src/lib/api.ts` (or a shared wrapper) + `WorkspaceShell.svelte` for the global host slot; `MapsTab.svelte` reads `downloadingMaps` / `downloadProgress` and renders the progress badge; `WorkspaceShell.svelte` status-bar slot renders `bundleProgress`; `MapsTab.svelte` open-loader button gates on `$busy`; `WorkspaceShell.svelte` Cmd-K trigger restyle.
- **Backend**: session-restore SHALL invoke `register_active_map_layer` for the restored selection (or an equivalent that prepares the same viewport-hint state). `src-tauri/src/application/mod.rs::restore_session` (≈ lines 1104-1185) is modified to push the restored map through the layer-registration path; no new IPC commands, no new events.
- **Spec evidence**: smoke that the cold-start active map renders at calibrated zoom; that the Maps tab shows live download progress; that the status bar mirrors the loader's progress text during downloads; that double-clicking Maps is a no-op when busy; that the Cmd-K trigger does not behave like a text input; that an IPC failure shows the structured error toast with `data-testid="ipc-error"`.
- **Risk**: low. All fixes target paths that already work in adjacent flows — viewport restore mirrors the click-open path; progress wiring mirrors what `BundleLoader` already does; the IPC error toast is additive. The session-restore backend change is the most invasive — it shares code with `register_active_map_layer` which is already covered by smoke when the user clicks a map.

## Out of scope

- Redesigning the bundle-loader Sheet — the existing extracted `BundleLoader` is unchanged; this proposal only re-wires the workspace surfaces that observe its state.
- Listener-count refactors. The `consolidate-state-event-flow` change is the single owner of `listen()` calls; this proposal MUST NOT add new event subscriptions outside `src/routes/+layout.svelte`.
- New Tauri commands. Every fix uses an existing IPC or store; the backend change to `restore_session` reuses the existing `register_active_map_layer` helper.
- Production telemetry. The IPC-error toast is dev-only (gated on `import.meta.env.DEV`); a real telemetry pipeline is a separate proposal.
- Map preview thumbnails (deferred, tracked under `map-preview-thumbnails`).

## Dependencies

This change sits on top of the redesign trilogy (already merged) plus `consolidate-state-event-flow`, `bundle-loader-as-overlay`, `stream-bundle-file-availability`, `cache-project-catalog-locally`, `raise-download-concurrency`. No new dependencies. The bug fixes touch capabilities owned by `ui-shell`, `lizaalert-integration`, `project-persistence`, and `map-bundles`, so this is filed as a multi-capability spec delta.
