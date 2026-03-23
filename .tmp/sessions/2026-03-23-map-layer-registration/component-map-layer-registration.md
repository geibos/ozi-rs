# Component: Map Layer Registration

Status: started

## Why this next
- The UI can download and display a map, but the project model still does not know that a map was opened.
- That leaves `MapLayer` underused and blocks the requirement that project state carries independent map data.
- Registering opened maps through AppState is the smallest slice that reconnects the UI flow to the domain and command layers.

## This slice
- Add source-path metadata to domain map layers.
- Add application support for registering an opened map as a project layer.
- Cover first-open and reopen behavior with focused tests.

## Out of scope
- track and waypoint editing flows
- persistence of project map layers
- replacement of the current active offline-map viewer
