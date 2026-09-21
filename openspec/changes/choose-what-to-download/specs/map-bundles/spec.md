## ADDED Requirements

### Requirement: The operator chooses what a bundle download fetches

A previewed bundle SHALL report the entries at the top level of its directory,
with the size the listing states where it states one. The loader SHALL present
them for selection with everything selected, and the download SHALL omit the
entries the operator cleared. With nothing cleared the download SHALL fetch the
whole bundle, as before.

#### Scenario: Leaving the print sheets on the server

- **WHEN** the operator clears the print-maps folder and starts the download
- **THEN** every other entry is fetched and nothing under that folder is

#### Scenario: Changing which project is previewed

- **WHEN** the operator clears an entry and then previews a different project
- **THEN** the choice resets, because a different bundle has different contents

#### Scenario: An offline bundle

- **WHEN** a cached bundle is opened without a network
- **THEN** its contents are reported from disk rather than from a listing
