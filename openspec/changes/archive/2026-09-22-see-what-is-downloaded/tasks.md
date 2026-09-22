## 1. Backend

- [x] 1.1 `cached_project_slugs` over the bundles root, with a test
- [x] 1.2 `cached` on `LizaProjectSummaryDto`, filled for the state snapshot and every streamed chunk; bindings regenerated

## 2. Frontend

- [x] 2.1 `filterProjects` shared by the list, matching name and slug plus the downloaded filter, with tests
- [x] 2.2 Downloaded badge, "downloaded only" toggle and an honest count in the loader
- [x] 2.3 Restored cache entries default to "not downloaded", with tests

## 3. Single-map downloads

- [x] 3.1 `download_map` takes a cancel token and drops the partial file when stopped
- [x] 3.2 `open_selected_map` registers the download and returns its id
- [x] 3.3 A `download-finished` event from both paths; the panel is tied to the id, with tests
- [x] 3.4 Callers show the panel for a map download; failures reach a toast

## 4. Verification

- [x] 4.1 `just ci` green
- [x] 4.2 Screenshot in `docs/progress/`
