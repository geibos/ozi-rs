## 1. The note

- [x] 1.1 `Waypoint.description`, `#[serde(default)]`, an empty note normalised to none
- [x] 1.2 `SetWaypointDescription` with its reverse, through the command stack
- [x] 1.3 `set_waypoint_description` command, registered, bindings regenerated

## 2. Travelling

- [x] 2.1 WPT field 11 read and written, round trip pinned
- [x] 2.2 GPX `<desc>` read and written, round trip pinned, `<desc>` before `<sym>` for the schema
- [x] 2.3 A mark with no note does not acquire one either way

## 3. Writing it

- [x] 3.1 A field in the Waypoint Inspector, committed on blur
- [x] 3.2 Both dictionaries
- [x] 3.3 The fixtures carry a mark with a note and one without, so the stand shows both

## 4. Gates

- [x] 4.1 `just ci` green
- [x] 4.2 Walked on the stand 2026-09-23 (`docs/progress/2026-09-23-a-note-on-the-mark/note.png`): selecting «ШТАБ» shows «сбор в 06:00, вода есть» under its name. The typed stand caught the DTO change as a compile error at three call sites, which is what it is typed for
