## ADDED Requirements

### Requirement: Bundle-download progress region has a stable layout

The bundle-download progress region in the bundle loader SHALL have an outer layout whose dimensions do not change as progress, file-ready, and download events arrive. Content slots inside the region (status message, current-file label, indeterminate bar, byte counters, progress bar, ready-files list, action buttons) MAY appear and disappear, but their containers SHALL reserve their footprint so that neighbouring UI does not reflow.

The total height of the progress region SHALL be a single CSS value reused both as the height of the region itself and as the bottom inset of the main bundle-loader grid.

#### Scenario: Progress events do not reflow the page

- **WHEN** a bundle download is in progress AND progress, file-ready, and download events arrive at 5+ events per second
- **THEN** the bottom edge of the progress region remains at a constant vertical position relative to the viewport AND the project list / map list above it does not shift

#### Scenario: Idle and busy states share the same outer height

- **WHEN** the application is idle (no download) AND when a download is in progress
- **THEN** the progress region occupies the same outer height in both states; toggles inside the region only change which content is visible, not the region's footprint

#### Scenario: Ready-files list grows inside its reserved slot

- **WHEN** files complete during a download and are appended to the ready-files list
- **THEN** the ready-files list scrolls inside its reserved slot rather than expanding the surrounding region
