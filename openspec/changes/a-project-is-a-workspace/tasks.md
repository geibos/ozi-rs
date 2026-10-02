## 1. One rule

- [x] 1.1 `workspace-route.ts`: worth opening when a map is active or a project file is open; tests
- [x] 1.2 The launcher's start-up redirect uses it
- [x] 1.3 The workspace route decides after the state arrives, and on later changes

## 2. Gates

- [x] 2.1 `just ci` green (2026-10-02: 423 Rust, 722 vitest, 60 shots)
- [x] 2.2 The screenshot matrix photographs the Maps tab's empty state, which needs a project with no map to stay in the workspace — `library-maps__empty__*`, 2026-10-02; walked on the stand too: `/project?maps=none` stays in the workspace, `/project` too, `/?state=cold` stays on the catalogue
