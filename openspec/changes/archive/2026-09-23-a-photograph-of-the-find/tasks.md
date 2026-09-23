# Tasks

- [x] 1.1 `Waypoint.attachments`, `#[serde(default)]` so every older project
      loads; `set_attachments` drops blanks and repeats and answers the
      previous list for the undo delta. Three domain tests.
- [x] 1.2 `SetWaypointAttachments` command with its reverse, and
      `apply_set_waypoint_attachments`; a test that attach → undo → redo puts
      the list back exactly.
- [x] 1.3 `set_waypoint_attachments` command registered, `WaypointDto` carries
      the list, bindings and fixtures regenerated.
- [x] 1.4 The inspector: attach through the file dialog, show, detach, and the
      sentence saying where the files live.
- [x] 1.5 The stand keeps what was attached — a command answered "accepted"
      that changes nothing on screen is the lie the stand exists to stop.
- [x] 1.6 `docs/commands-reference.md` gains the command; its completeness
      test caught the omission.
- [x] 1.7 Walked on the stand: two photographs attached and listed, the first
      detached and exactly it gone.
- [x] 1.8 `just ci` green: 395 Rust, 660 frontend.
