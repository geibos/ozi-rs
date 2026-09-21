## ADDED Requirements

### Requirement: A saved project is reachable without the command palette

The screen the application opens on SHALL offer to open a saved project file,
and SHALL list the most recently opened projects as direct choices. No control
SHALL use the same words for opening a saved project and for opening the
LizaAlert catalogue, since they are different things.

#### Scenario: A crew arrives with yesterday's work

- **WHEN** the application is launched and no project is open
- **THEN** the first screen offers to open a saved project, and lists the recent ones, without the operator needing the command palette

#### Scenario: A recent project has moved

- **WHEN** an entry in that list no longer opens
- **THEN** the failure is reported and the entry is dropped, rather than failing again the next time it is chosen

#### Scenario: Opening from either surface

- **WHEN** a project is opened from the first screen or from the command palette
- **THEN** the same thing happens: it is loaded, remembered, and the map is framed on it
