## Why

Every time the user clicks Sidebar "Maps…" the app navigates to `/`, `+page.svelte:65` mounts, and `onMount` calls `loadProjects()` (`+page.svelte:103`). That IPC re-fetches the entire LizaAlert catalog from `maps.lizaalert.ru`, re-parses it on the backend, and streams it back as `projects-chunk` events that the frontend appends one by one via `appendProjectsChunk` (`stores.ts:58`). Because `projectsStore` (`stores.ts:30`) lives only as long as the JS module, the cost is paid in full on every visit — both at cold start and every subsequent round-trip from the workspace.

Two concrete consequences the user feels:

1. The list is empty when the loader first paints. The filter input is unresponsive — clicking it lags before any keystroke registers — because the same JS thread is fielding `projects-chunk` events that mutate `projectsStore` and re-derive `filtered` (`+page.svelte:112`) for each chunk.
2. Re-entering the loader from the workspace shows an empty list again, even though the catalog is essentially the same as it was 30 seconds ago. The user must wait for the entire catalog to refetch before they can pick anything.

The LizaAlert catalog is dominated by historical missions: per the user, "Исторические практически не меняются во времени." A catalog with hundreds of entries that changes by a handful per week does not need to be refetched from scratch on every mount.

## What Changes

- The frontend SHALL persist the LizaAlert catalog (`LizaProjectSummaryDto[]`) locally with a write timestamp. The default storage is browser `localStorage`; the implementation SHALL fall back to in-memory only if `localStorage` is unavailable or full.
- On mount, the bundle loader SHALL hydrate `projectsStore` synchronously from the local cache before any IPC. The list, the filter input, and the project count SHALL be interactive within the first paint when a cache exists.
- After the cached list renders, the frontend SHALL kick off `loadProjects()` as a background refresh. Incoming `projects-chunk` events SHALL merge into the existing list — new slugs are appended, existing slugs are replaced in place. Entries present in the cache but absent from the refresh SHALL be retained (deletions are not applied automatically; historical missions stay reachable).
- The cache SHALL be written after every successful refresh — after `appState.busy` transitions back to `false` and the latest chunk has been merged. The write SHALL be a single JSON serialization, debounced or fired on `busy → false`, not on every chunk.
- The filter `$derived` SHALL continue to operate on the full in-memory list. The filter input no longer waits for the IPC to finish before it becomes usable, because the cached list is already in the store before the input mounts.
- Note: an earlier draft of this proposal included list virtualization in scope. The first attempt broke list rendering in the Tauri WebView; virtualization is deferred to a dedicated follow-up change so it can be debugged in the running app. The cache-layer pieces are the dominant win and stand on their own.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `ui-shell`: the bundle loader renders the cached catalog instantly on mount. (Virtualization deferred to a separate change.)
- `lizaalert-integration`: the project catalog is persisted across sessions with stale-while-revalidate refresh semantics — known historical entries survive a refresh that omits them.

## Impact

- **Frontend**: `src/lib/stores.ts` (catalog cache load/save helpers, append-with-replace merge for known slugs), `src/routes/+page.svelte` (hydrate from cache before `loadProjects()`, swap the `{#each}` for a virtualized list), `src/lib/api.ts` (no shape changes — `loadProjects()` stays the same wrapper), `src/lib/types.ts` (only adds a `LizaProjectCatalogCache` type for the persisted shape).
- **Backend**: none. `load_projects`, `projects-chunk`, and `LizaProjectSummaryDto` keep their existing wire format.
- **Storage**: a single `localStorage` key (e.g. `liza:projects:v1`) holding `{ items: LizaProjectSummaryDto[], writtenAt: string }`. Estimated payload: ~50 bytes per entry × few hundred entries ≈ tens of kilobytes, well inside the localStorage quota; see design.md for the IndexedDB fallback decision.
- **Spec evidence**: smoke that re-entering the loader from the workspace renders the previous catalog within the first paint; smoke that filter typing is immediately responsive on a populated cache; smoke that a refresh which omits a known historical slug still keeps that slug in the list.
- **Risk**: low. The cache is a write-through copy of data the backend already serves; on any cache parse failure the frontend silently falls back to "no cache" and behaves exactly as before. Cache key versioning (`v1` suffix) lets us invalidate on schema changes.
