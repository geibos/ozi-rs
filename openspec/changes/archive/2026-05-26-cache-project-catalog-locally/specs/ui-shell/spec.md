## ADDED Requirements

### Requirement: Bundle loader hydrates the project list from the local catalog cache before the first paint

The bundle loader (`src/routes/+page.svelte`) SHALL display the cached LizaAlert project catalog on its first paint when a cache exists. The hydration SHALL happen synchronously at frontend module initialization (in `src/lib/stores.ts`), so that by the time the page mounts, the projects store already contains the cached entries.

The filter input, the project-count badge, and the `{#each}`-rendered list rows SHALL be interactive in that first paint — the user SHALL NOT have to wait for any IPC round-trip to type, scroll, or select a project that is already in the cache.

When no cache exists (first-ever launch, cache cleared, cache failed to parse), the bundle loader SHALL render in its current empty-until-chunks-arrive state with no regression.

The background `loadProjects()` refresh SHALL continue to fire — the cache is a fast path, not a replacement for refresh. As `projects-chunk` events arrive, the displayed list SHALL update via the upsert-by-slug merge defined in `lizaalert-integration`.

#### Scenario: Returning user sees the catalog instantly

- **WHEN** the user previously completed a refresh in any prior session AND opens the application again
- **THEN** the bundle loader's project column renders the cached catalog within the first paint, the filter input accepts keystrokes immediately, and any background refresh updates the list in place without blocking input

#### Scenario: Reopening the loader from the workspace is instant

- **WHEN** the user is in the workspace (`/project`) and clicks the Sidebar "Maps…" button to navigate back to the bundle loader (`/`)
- **THEN** the cached catalog is visible on the first paint of `/`, without the empty-list flicker that today's mount cycle produces

#### Scenario: First-ever launch behaves like today

- **WHEN** the application starts on a machine with no prior cache for the catalog
- **THEN** the bundle loader paints with an empty project list AND `loadProjects()` populates it via `projects-chunk` events exactly as before this change

#### Scenario: Cache hydration does not contend with the IPC refresh

- **WHEN** the cached catalog is hydrated on mount AND a `loadProjects()` refresh begins immediately afterward
- **THEN** entries from the refresh merge into the displayed list without removing any cached entry, without re-creating list rows that did not change, and without blocking filter input
