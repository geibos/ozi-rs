## MODIFIED Requirements

### Requirement: System fetches the LizaAlert project list as a stream

The system SHALL fetch the list of available projects from `maps.lizaalert.ru` and SHALL deliver results to the frontend in chunks via a `projects-chunk` event so the UI can render progressively.

The frontend SHALL merge each incoming chunk into its working project list with upsert-by-slug semantics: an entry whose `slug` is already present SHALL replace the existing entry in place; an entry whose `slug` is new SHALL be appended at the end of the list. Entries already present in the working list but absent from a refresh SHALL NOT be removed by that refresh; they SHALL remain reachable from the bundle loader until the user explicitly clears the local catalog cache.

#### Scenario: Streaming project list

- **WHEN** the frontend invokes the project list refresh
- **THEN** the backend emits one or more `projects-chunk` events carrying partial project summaries, and the UI appends each chunk to the displayed list

#### Scenario: Refresh updates an existing entry in place

- **WHEN** a `projects-chunk` payload contains an entry whose `slug` already exists in the working project list AND whose `name` differs from the cached value
- **THEN** the existing entry's `name` is updated in place and its position in the list is preserved

#### Scenario: Refresh that omits a known slug keeps the entry

- **WHEN** a complete refresh finishes (every `projects-chunk` event has been processed and no further chunk has arrived for the debounce window described in the Cache write Requirement) AND the working list contains a `slug` that was not referenced by any chunk in that refresh
- **THEN** that `slug` remains in the working list and stays selectable in the bundle loader

## ADDED Requirements

### Requirement: LizaAlert project catalog is cached locally with a write timestamp

The system SHALL persist the merged LizaAlert project catalog to local browser storage with a write timestamp. The cache SHALL live under a versioned key (e.g. `liza:projects:v1`) and SHALL hold a JSON object `{ items: LizaProjectSummaryDto[], writtenAt: string }` where `writtenAt` is an ISO-8601 UTC timestamp. The system SHALL write the cache once per completed refresh, after the projectsStore stops changing for a short debounce window (~800 ms), not on every `projects-chunk` event.

The system SHALL hydrate the working catalog from this cache synchronously at frontend module initialization, before any component mounts. When the cache is absent, malformed, or fails shape validation, hydration SHALL silently fall back to an empty list — no toast, no error surface.

The system SHALL swallow storage write failures (e.g. `QuotaExceededError`) silently and log them only to the developer console; the cache is best-effort and SHALL NOT block any user-facing flow.

#### Scenario: Cache survives across app restarts

- **WHEN** the user successfully completes a project list refresh in session 1 AND restarts the application
- **THEN** session 2's bundle loader renders the cached catalog within the first paint, without waiting for any IPC

#### Scenario: Cache is written once per refresh, not per chunk

- **WHEN** a single refresh delivers five `projects-chunk` events
- **THEN** the system writes the cache exactly once — after the refresh has settled (no further chunk arrives for ~800 ms) — not five times

#### Scenario: Corrupt cache is tolerated

- **WHEN** the local storage value under the catalog cache key is missing, empty, or fails JSON parsing or shape validation
- **THEN** the working catalog initializes to an empty list, no error is surfaced to the user, and the next refresh proceeds normally and writes a fresh cache on completion

#### Scenario: Quota exceeded does not break the UI

- **WHEN** a cache write fails with `QuotaExceededError` or any other storage error
- **THEN** the error is logged to the developer console only, the user sees no toast, and the in-memory catalog remains usable for the rest of the session

### Requirement: Catalog cache merges refresh deltas without dropping known entries

The local catalog cache SHALL accumulate the union of entries the frontend has ever observed via `projects-chunk` events. A refresh that does not reference a previously-seen `slug` SHALL NOT remove that entry from the cache. The cache SHALL therefore be a superset of any single refresh's payload.

#### Scenario: Historical entries persist across refreshes

- **WHEN** the user has a cached catalog containing 200 entries AND triggers a refresh that returns only 195 of those 200 entries
- **THEN** after the refresh settles, the cache still contains 200 entries and the bundle loader still lists all 200; the five entries absent from the refresh are not removed

#### Scenario: New entries appear after refresh

- **WHEN** a refresh returns three previously-unseen entries among its chunks
- **THEN** those three entries are appended to the working list AND included in the next cache write
