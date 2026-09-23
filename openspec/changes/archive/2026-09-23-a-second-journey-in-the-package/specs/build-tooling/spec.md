## ADDED Requirements

### Requirement: The gate walks more than one journey, one at a time

The customer-journey gate SHALL cover more than the editing spine, and SHALL
run its journeys sequentially.

Each journey drives the packaged application through a session that owns the
screen. Two at once click into each other's window: the first time a second
journey was added, both failed, including the one that had passed on its own
a minute earlier.

#### Scenario: Two journeys in one run

- **WHEN** the gate is run
- **THEN** each journey runs in turn, and one journey's session never interferes with another's

#### Scenario: Layers in the packaged application

- **WHEN** the gate walks the layer journey
- **THEN** a layer is created from the library, becomes the active one, and is removed again, all over real IPC
