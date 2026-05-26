## ADDED Requirements

### Requirement: Session restore registers the active map layer so the workspace lands at calibrated bounds

When the application restores a persisted active map on startup (`Application::restore_session` in `src-tauri/src/application/mod.rs`), the restored `ActiveMapSelection` SHALL be pushed through the same active-map registration code path that a user-initiated `open_selected_map` invokes — i.e. the function that registers the tile-source layer and prepares the metadata that the frontend `MapView` reads to call `fitBounds(meta.bounds)`. The restored map SHALL NOT merely be assigned to `self.lizaalert.active_map` without registration.

Errors from the registration step SHALL be reported via `update_status` with `DiagnosticLevel::Error` and the active map SHALL be cleared, matching the existing failure mode of the click-open path. The session-restore flow SHALL NOT silently leave a half-restored active map (selection set, layer not registered) on the way out.

#### Scenario: Cold-start restores last map at fit-to-bounds

- **WHEN** the user has previously opened an OZI map AND the session file records it as the active map AND the user relaunches the application AND the map file still exists at its recorded path
- **THEN** the workspace `/project` route renders with the map's calibrated bounds visible (i.e. the same viewport the user would see immediately after a fresh click-open) AND no JSON parse / stringify error toast is shown AND no `data-testid="ipc-error"` toast is shown in dev builds

#### Scenario: Cold-start matches click-open viewport

- **WHEN** the user clicks an OZI map row in the bundle loader from a cold start (no session) AND records the resulting MapLibre viewport (`map.getBounds()`) AND then closes the app AND relaunches it
- **THEN** the viewport after relaunch matches the viewport from the click-open (within MapLibre's animation tolerance) AND not the default MapLibre `{ center: [0,0], zoom: 0 }`

#### Scenario: Restored map missing on disk falls back cleanly

- **WHEN** the session file records an active map whose `local_path` no longer exists on disk AND the application relaunches
- **THEN** the existing diagnostic "Session restore skipped missing active map: …" is emitted AND the workspace either falls back to the bundle-loader cold-start surface (no active map) AND no half-registered active map remains in `self.lizaalert.active_map`

#### Scenario: Active map registration error during restore is surfaced

- **WHEN** session restore validates the selection and the layer-registration step fails (e.g. tile-source registration returns an error)
- **THEN** the error is reported via `update_status` with `DiagnosticLevel::Error` AND `self.lizaalert.active_map` is reset to `None` AND the workspace falls back to the cold-start bundle loader
