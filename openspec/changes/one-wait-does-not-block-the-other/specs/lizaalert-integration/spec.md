## ADDED Requirements

### Requirement: A catalogue refresh does not block a bundle download

A running catalogue refresh SHALL NOT prevent a bundle from being downloaded or
opened from disk, and a running bundle operation SHALL NOT prevent the
catalogue from being refreshed. Each SHALL still refuse a second instance of
itself, and SHALL say which of the two is in the way. Completing one SHALL NOT
clear the other's in-progress state.

#### Scenario: Downloading during the launch-time refresh

- **WHEN** the catalogue walk started at launch is still running AND the operator asks to open a bundle
- **THEN** the download starts, rather than being refused until the walk ends or is stopped

#### Scenario: Refreshing during a download

- **WHEN** a bundle download is in flight AND the operator asks to refresh the catalogue
- **THEN** the refresh starts

#### Scenario: A second bundle

- **WHEN** a bundle is already being downloaded or opened AND another is asked for
- **THEN** it is refused, and the refusal names a bundle operation rather than the project list

#### Scenario: The walk ends while a download runs

- **WHEN** the catalogue walk finishes or is stopped while a download is in flight
- **THEN** the download keeps its progress and its own in-progress state
