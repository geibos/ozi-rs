# waypoints Spec Delta

## REMOVED Requirements

### Requirement: System exports waypoints to PLT

**Reason**: The OziExplorer `.plt` format is a track (polyline) format and does not support waypoints. The previous requirement conflated the two formats. It is replaced by a correctly-scoped requirement targeting OziExplorer's actual waypoint format (`.wpt` v1.1). No PLT-waypoint code ever shipped, so there is no migration impact.

**Migration**: None. Replaced by "System exports waypoints to OziExplorer WPT" below.

## ADDED Requirements

### Requirement: System exports waypoints to OziExplorer WPT

The system SHALL provide an "Export waypoints (WPT)" action that writes all waypoints in a chosen layer to a user-selected `.wpt` file in the OziExplorer WPT format (version 1.1 header, latitude/longitude in WGS84 decimal degrees, optional symbol code, optional elevation and timestamp).

#### Scenario: Export waypoints with symbols to WPT

- **WHEN** the user invokes "Export waypoints (WPT)" on a layer with three waypoints, two of which have symbols
- **THEN** the resulting `.wpt` file contains exactly three rows in OziExplorer 1.1 WPT format with symbol codes preserved for the two waypoints and a default symbol for the third

#### Scenario: WPT export dialog suggests `<bundle>/<layer>.wpt`

- **WHEN** the user invokes WPT export with a bundle active and a waypoint layer named `points`
- **THEN** the file picker pre-fills with `<bundle>/points.wpt`

#### Scenario: WPT export with no active bundle suggests filename only

- **WHEN** the user invokes WPT export with no active bundle
- **THEN** the file picker suggests `<layer>.wpt` only, without a directory
