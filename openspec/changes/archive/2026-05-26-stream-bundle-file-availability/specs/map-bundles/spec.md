## ADDED Requirements

### Requirement: Per-map availability inside the active bundle streams into the UI live

The system SHALL surface a map within the active bundle as "available" (clickable, badged as cached) within the same event tick the backend completes the download of that map's file. The frontend SHALL NOT wait for the whole bundle to finish before flipping a freshly-downloaded map row from "downloading" to "cached".

The backend SHALL emit `state-changed` at the same point it emits `bundle-file-ready` so that the next `getAppState()` snapshot exposes the updated `downloaded` flag on `LizaMapPackageDto` for the just-completed map. The flag's underlying source (the per-`LizaMapPackage` `local_path`) MUST already be set before that emission.

#### Scenario: Single map finishes mid-download

- **WHEN** a multi-file bundle download is in progress AND the file backing one map (e.g. `10-Tracks/topo-A.sqlitedb`) finishes downloading while another file is still in flight
- **THEN** within the same tick the backend emits `bundle-file-ready` for that file it also emits `state-changed`; the next `AppStateDto.current_project.maps[i].downloaded` for that map is `true`; the corresponding map row in the bundle loader transitions from the blue `%` progress badge to the green `cached` badge and becomes clickable

#### Scenario: Clicking a just-finished map mid-download opens it

- **WHEN** a multi-file bundle download is in progress AND a map row whose file completed earlier in the same download now shows the `cached` badge AND the user clicks that row
- **THEN** the map opens in the workspace via the existing `openSelectedMap` path; the ongoing download of the remaining files continues uninterrupted; subsequent files completing in that bundle continue to flip their rows live

#### Scenario: All maps inside one bundle finish before non-map files

- **WHEN** a bundle contains two map files and one reference PDF AND both map files finish downloading before the PDF
- **THEN** both map rows flip to `cached` and become clickable as each finishes; the user can open either map without waiting for the PDF; once the PDF completes the bundle download is fully done with no further row state changes
