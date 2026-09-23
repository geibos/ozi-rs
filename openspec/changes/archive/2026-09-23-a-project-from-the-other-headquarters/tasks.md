# Tasks

- [x] 1.1 `openProjectFile` answers whether a project was opened; the launcher
      leaves for the workspace only when it did.
- [x] 1.2 `recentProjects` is a store that `rememberProject` and
      `forgetProject` publish to; `BundleLoader` reads it instead of a `const`.
- [x] 1.3 `src/test/recent-projects-store.test.ts`: gains a project when it is
      remembered, puts the one just opened first, loses a forgotten one, and
      notifies a subscriber rather than changing on the next read.
- [x] 1.4 Two stand lies fixed: `load_project_file` answered "accepted" and
      left the fixture alone, so the launcher could not tell an opened project
      from a cancelled dialog; and the cold-start fixture kept sending a
      visitor back to the launcher after a project was opened.
- [x] 1.5 Walked on the stand: a colleague's `.ozp` opens, the workspace shows,
      the recent list gains it, and Cmd+S writes back to the same path without
      asking.
- [x] 1.6 `just ci` green: 391 Rust, 618 frontend.
