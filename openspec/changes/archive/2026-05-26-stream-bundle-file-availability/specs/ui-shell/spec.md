## ADDED Requirements

### Requirement: Bundle-loader status bar omits the "Open bundle now" affordance and the ready-files list

The bundle-loader status bar SHALL NOT include an "Open bundle now" button. The bundle-loader status bar SHALL NOT include a ready-files list enumerating individual files that have finished downloading during the active bundle download. Both surfaces were workarounds for delayed per-map availability and SHALL be replaced by the Maps column reflecting per-file readiness directly (see the `map-bundles` capability).

The status bar SHALL retain its other slots from change A: status-line, current-file label, progress bar, byte counters, and action slot. The remaining action slot SHALL continue to host the `Cancel` button while a download is in flight, and SHALL be empty otherwise.

The bar's outer height SHALL shrink by the height previously reserved for the ready-files row. Within a single session, the bar's outer height SHALL still NOT jitter as progress events arrive — the stable-layout guarantee for the remaining content from change A SHALL be preserved.

#### Scenario: No "Open bundle now" button at any point in a download

- **WHEN** the user starts a bundle download AND files start completing one by one
- **THEN** no "Open bundle now" button appears in the status bar at any point — neither before, during, nor after individual files complete; the user's affordance for opening a finished map is the Maps column row itself, which has flipped to `cached`

#### Scenario: No ready-files list at any point in a download

- **WHEN** a bundle download is in progress AND files are completing
- **THEN** no list of recently-completed files is rendered in the status bar; per-file completion is communicated solely through the corresponding Maps-column row's badge transition

#### Scenario: Cancel button still shown while a download is in flight

- **WHEN** a bundle download is in flight AND `activeDownloadId` is non-null
- **THEN** the action slot in the status bar contains a single `Cancel` button; no other action button is present in that slot

#### Scenario: Stable-layout invariant preserved for remaining slots

- **WHEN** a bundle download is in progress AND `download-progress` / `bundle-progress` / `state-changed` events arrive at 5+ per second
- **THEN** the bottom edge of the status bar remains at a constant vertical position relative to the viewport for the duration of the download; toggles inside the remaining slots only change which content is visible, not the bar's outer footprint
