## ADDED Requirements

### Requirement: The catalogue refresh can be stopped

The catalogue refresh SHALL be stoppable by the operator while it runs, and the
interface SHALL offer that control for as long as a refresh is in flight and
not otherwise. The projects already read SHALL remain listed after a stop.

Stopping SHALL release the busy flag exactly as completion does, so that a
bundle download can start immediately afterwards.

#### Scenario: A crew needs a bundle before the listing finishes

- **WHEN** the operator stops the refresh while it is walking the listing
- **THEN** the walk stops without requesting further pages, the projects already read stay in the list, and a bundle download can be started

#### Scenario: No refresh is running

- **WHEN** no catalogue refresh is in flight
- **THEN** no stop control is offered

### Requirement: A stopped refresh is not passed off as the whole catalogue

A refresh that was stopped SHALL NOT replace the cached catalogue, because it
read only its first pages. The system SHALL report a stopped refresh
distinguishably from a completed one rather than reporting a count as if it
were the total.

#### Scenario: Stopping a refresh and reopening the application offline

- **WHEN** a refresh is stopped partway and the application is later started without a network
- **THEN** the catalogue shown is the last complete one, not the fragment the stopped refresh read

#### Scenario: Reporting the outcome

- **WHEN** a refresh is stopped partway
- **THEN** the status says the refresh was stopped at that many projects, not that that many were loaded
