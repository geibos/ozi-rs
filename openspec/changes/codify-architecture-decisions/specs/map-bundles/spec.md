## ADDED Requirements

### Requirement: Bundles root defaults to Documents and holds one directory per bundle

Unless the user has chosen another directory in the current session, the bundles root SHALL be `<platform Documents directory>/LizaAlert Maps`, resolved through the Tauri path resolver at startup. When the Documents directory cannot be resolved, the system SHALL fall back to a relative `bundles` directory and log a warning instead of failing to start. Each LizaAlert bundle SHALL live in its own subdirectory of the bundles root named by the project slug (`<bundles root>/<slug>/`), and the system SHALL treat a project as cached when `<bundles root>/<slug>/2-Coordinates.txt` exists.

#### Scenario: Default root on a fresh start

- **WHEN** the application starts and the user has not changed the bundles root in this session AND downloads the project with slug `2026-03-29_demo`
- **THEN** the bundle files land under `<Documents>/LizaAlert Maps/2026-03-29_demo/`

#### Scenario: Bundles are sibling directories under the root

- **WHEN** two projects with slugs `2026-03-29_demo` and `2026-04-02_forest` have been downloaded
- **THEN** the bundles root contains the two sibling directories `2026-03-29_demo/` and `2026-04-02_forest/`, each holding only its own bundle files

#### Scenario: Cached marker drives the catalog status

- **WHEN** `<bundles root>/<slug>/2-Coordinates.txt` exists for a catalog entry
- **THEN** that entry is reported as cached in the bundle catalog and can be opened without network access
