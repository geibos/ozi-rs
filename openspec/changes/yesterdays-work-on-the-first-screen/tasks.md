## 1. Frontend

- [x] 1.1 `openProjectFile(known?)` in `$lib/actions/project.ts`, with tests written first
- [x] 1.2 The launch screen offers it, with the three most recent projects
- [x] 1.3 The palette's dialog and recents handlers delegate to it
- [x] 1.4 The Maps tab's button stops calling the catalogue a project

## 2. Verification

- [x] 2.1 On the stand's cold start: the button reads «Открыть сохранённый проект…» and two seeded recents render under it
- [x] 2.2 `just ci` green
- [ ] 2.3 `just smoke` green (blocked: the Mac2 driver cannot enable automation mode — see `docs/STATE.md`)
