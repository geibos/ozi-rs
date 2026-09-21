## ADDED Requirements

### Requirement: A map can be opened before its bundle finishes

A map whose file has landed during a bundle download SHALL be openable from
disk while the remaining files continue downloading, and the operator SHALL be
told when the first such map becomes available.

#### Scenario: The topo layer lands first

- **WHEN** a bundle download writes a map file and the rest of the bundle is still downloading
- **THEN** that map opens from disk without starting another download, and the operator is told it is ready

#### Scenario: Files that are not maps

- **WHEN** the file that lands is a print sheet, a coordinates file or an archive
- **THEN** nothing is announced, because none of them is something the crew can open

#### Scenario: One announcement per download

- **WHEN** several maps of the same bundle land in turn
- **THEN** the operator is told once, not once per file
