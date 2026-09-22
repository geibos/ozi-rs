## 1. Backend

- [x] 1.1 `CatalogueWalk` carries projects, page count and whether it was stopped
- [x] 1.2 The walk checks the token before each page and after each page, with a test on a two-page server
- [x] 1.3 The walk takes its listing root so a test server can drive it
- [x] 1.4 `begin_load_projects` mints the token; `cancel_project_listing` on `AppState` trips it
- [x] 1.5 `cancel_project_listing` command, registered, bindings regenerated
- [x] 1.6 A stopped walk does not overwrite the cache
- [x] 1.7 The status line says a refresh was stopped rather than completed

## 2. Frontend

- [x] 2.1 `cancelProjectListing` in `api.ts`
- [x] 2.2 A stop control beside the refreshing hint, in both dictionaries
- [x] 2.3 Behavioural test: the control appears only while refreshing and reaches the backend
- [x] 2.4 The stand answers the new command

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
