## MODIFIED Requirements

### Requirement: System fetches the LizaAlert project list as a stream

The system SHALL fetch the list of available projects from `maps.lizaalert.ru` and SHALL deliver results to the frontend in chunks via a `projects-chunk` event so the UI can render progressively.

The frontend SHALL merge each incoming chunk into its working project list with upsert-by-slug semantics: an entry whose `slug` is already present SHALL replace the existing entry in place; an entry whose `slug` is new SHALL be appended at the end of the list.

An entry absent from a refresh SHALL be removed when, and only when, that refresh walked the whole listing. A refresh that was stopped read only a prefix of the catalogue and SHALL remove nothing. Rows sent from the cache before the walk began SHALL NOT count as evidence that a project still exists.

This replaces the previous rule, under which an entry absent from a refresh was never removed and stayed reachable until the operator cleared the cache by hand. That rule was written when there was no way to tell a complete refresh from an interrupted one; there is now, so a search taken down upstream no longer stays in the list and in the cache for good, offered to a crew and failing when opened.

#### Scenario: Streaming project list

- **WHEN** the frontend invokes the project list refresh
- **THEN** the backend emits one or more `projects-chunk` events carrying partial project summaries, and the UI appends each chunk to the displayed list

#### Scenario: Refresh updates an existing entry in place

- **WHEN** a `projects-chunk` payload contains an entry whose `slug` already exists in the working project list AND whose `name` differs from the cached value
- **THEN** the existing entry's `name` is updated in place and its position in the list is preserved

#### Scenario: A search is taken down between two refreshes

- **WHEN** a complete catalogue walk does not list a project the previous walk listed
- **THEN** that project is no longer in the list or in the cache

#### Scenario: The operator stops the refresh

- **WHEN** a catalogue walk is stopped partway
- **THEN** no project is removed from the list

#### Scenario: The cached rows sent before the walk

- **WHEN** a refresh emits the cached catalogue before walking
- **THEN** those rows alone do not count as evidence that a project still exists

### Requirement: Catalog cache merges refresh deltas without dropping known entries

The local catalog cache SHALL hold what the last complete refresh listed, together with entries seen since that no refresh has contradicted. A refresh that was stopped SHALL NOT remove anything from the cache, because it read only a prefix of the listing. A refresh that walked the whole listing SHALL remove the entries it did not reference.

This replaces the previous rule, under which the cache accumulated the union of everything ever observed and was therefore a superset of any single refresh. That rule kept a crew's list intact across an interrupted refresh, which was the right trade when a complete refresh could not be distinguished from an interrupted one. It also meant that a search taken down upstream was never removed: it stayed listed, stayed cached, and failed when opened, with no way to clear it short of deleting the cache by hand.

#### Scenario: A stopped refresh keeps the cached entries it did not reach

- **WHEN** the operator has a cached catalogue of 200 entries AND stops a refresh after it has listed 195 of them
- **THEN** the cache still contains 200 entries and all 200 stay listed

#### Scenario: A complete refresh drops what it did not list

- **WHEN** the operator has a cached catalogue of 200 entries AND a refresh walks the whole listing and returns 195 of them
- **THEN** the five entries the listing no longer carries are removed from the list and from the cache

#### Scenario: New entries appear after refresh

- **WHEN** a refresh returns three previously-unseen entries among its chunks
- **THEN** those three entries are appended to the working list AND included in the next cache write
