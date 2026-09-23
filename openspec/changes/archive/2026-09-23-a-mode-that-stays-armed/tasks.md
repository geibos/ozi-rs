# Tasks

- [x] 1.1 Remove the `finally { addWaypointMode.set(false) }` from the map's
      click handler, with the reason beside it.
- [x] 1.2 `src/test/a-mode-that-stays-armed.test.ts`: the click handler does not
      leave the mode, and all three ways out are still where they live — a
      sticky mode with no way out is worse than the defect it replaced.
- [x] 1.3 Walked on the stand: the chip arms the mode, three clicks produce
      three `add_waypoint` calls with the chip still pressed, and Escape leaves.
- [x] 1.4 `just ci` green.
