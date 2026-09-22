## ADDED Requirements

### Requirement: Every track in the project has a row

The track list SHALL show one row for every track in the project, including a
track the map cannot draw because it has fewer than two points in every
segment. Such a track is part of the project and is exported with it, so it
SHALL be selectable, renamable and deletable like any other.

The list SHALL NOT be derived from the map's features, whose omissions are
correct for drawing and wrong for a listing.

#### Scenario: A track of a single point

- **WHEN** the project contains a track whose only segment holds one point
- **THEN** the track list shows a row for it, and the map draws nothing for it

### Requirement: The track list is fetched without track geometry

The data behind the track list SHALL carry only what a row shows. Track
coordinates SHALL NOT be transferred in order to render the list, so that a
project of many long recordings costs the list nothing beyond its rows.

#### Scenario: A project of long recordings

- **WHEN** the track list is loaded for a project whose tracks hold hundreds of thousands of points
- **THEN** no track coordinates are transferred
