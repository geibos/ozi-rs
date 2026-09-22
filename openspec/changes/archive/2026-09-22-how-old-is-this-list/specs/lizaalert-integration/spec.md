## MODIFIED Requirements

### Requirement: LizaAlert project catalog is cached locally with a write timestamp

The system SHALL persist the merged LizaAlert project catalog to local browser storage with a write timestamp. The cache SHALL live under a versioned key (e.g. `liza:projects:v1`) and SHALL hold a JSON object `{ items: LizaProjectSummaryDto[], writtenAt: string }` where `writtenAt` is an ISO-8601 UTC timestamp. The system SHALL write the cache once per completed refresh, after the projectsStore stops changing for a short debounce window (~800 ms), not on every `projects-chunk` event.

The system SHALL hydrate the working catalog from this cache synchronously at frontend module initialization, before any component mounts. When the cache is absent, malformed, or fails shape validation, hydration SHALL silently fall back to an empty list — no toast, no error surface.

The system SHALL swallow storage write failures (e.g. `QuotaExceededError`) silently and log them only to the developer console; the cache is best-effort and SHALL NOT block any user-facing flow.

The system SHALL also show that timestamp to the operator beside the project list, and within the notice shown when a refresh has failed, so that a saved list can be told apart from a current one. It SHALL be shown as a calendar date and a clock time rather than an elapsed interval. A timestamp that cannot be read as a date SHALL be shown as nothing at all.

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

#### Scenario: Reading a saved list offline

- **WHEN** the refresh has failed and the operator is reading the cached list
- **THEN** the notice names the date and time that list was written, so a search published since then can be recognised as missing rather than as non-existent

#### Scenario: An unreadable timestamp

- **WHEN** the stored timestamp cannot be parsed as a date
- **THEN** nothing is shown in its place, rather than a placeholder that looks like a date
