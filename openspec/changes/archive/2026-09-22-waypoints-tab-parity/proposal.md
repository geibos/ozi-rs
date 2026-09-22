## Why

The Tracks tab has been brought up to field speed — search, bulk visibility,
isolate, a locate button, a stats subline. The Waypoints tab lists the same kind
of object and offers none of it: a flat list of names, no way to find one, no way
to see where it is without clicking it, and visibility only one row at a time.

A search carries waypoints for the task point, the found object, dangerous spots
and the group's marks. Once there are more than a screenful, the tab stops being
useful for the same reasons the Tracks tab did.

## What Changes

- The Waypoints tab gains a search field with a clear button, a "shown of total"
  counter and a distinct empty state, matching the Tracks tab. The matching rule
  is shared code, so both tabs behave identically.
- Show-all and hide-all controls, plus a per-row "only this one" action, each a
  single backend command.
- Each row carries its coordinates as a subline and a "show on map" button that
  flies the map to that waypoint.

## Capabilities

### Modified Capabilities
- `waypoints`: finding a waypoint by name, bulk visibility and isolation, and
  locating a waypoint on the map from its row.

## Impact

- **Backend**: `set_all_waypoints_visible` and `show_only_waypoint` in the
  domain, application and command layers; generated bindings.
- **Frontend**: `src/components/library/WaypointsTab.svelte`; a shared name
  filter extracted from `src/lib/track-features.ts`; a `waypoint` focus request
  in `src/lib/stores.ts` consumed by `MapView`; i18n keys.
- **Risk**: low. Waypoint visibility is already a non-undoable style mutation,
  so the bulk operations do not touch the command stack.
