## 1. Backend

- [x] 1.1 `cached_project_slugs` over the bundles root, with a test
- [x] 1.2 `cached` on `LizaProjectSummaryDto`, filled for the state snapshot and every streamed chunk; bindings regenerated

## 2. Frontend

- [x] 2.1 `filterProjects` shared by the list, matching name and slug plus the downloaded filter, with tests
- [x] 2.2 Downloaded badge, "downloaded only" toggle and an honest count in the loader
- [x] 2.3 Restored cache entries default to "not downloaded", with tests

## 3. Verification

- [x] 3.1 `just ci` green
- [ ] 3.2 Screenshot in `docs/progress/`
