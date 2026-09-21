## ADDED Requirements

### Requirement: A map's download size is known before the download starts

A map package SHALL carry its size in bytes when that is known — read from the
listing for a remote map and from the file for a cached one — and the screens
that offer a map for opening SHALL show it. An unknown size SHALL be shown as
unknown rather than as zero.

#### Scenario: Choosing between a topo and a satellite layer

- **WHEN** a project lists a 16 MiB topo map and a 185 MiB satellite map
- **THEN** each row states its size, so the operator can pick what the link can carry

#### Scenario: A listing without a size column

- **WHEN** the listing states no size for a file
- **THEN** that map's size is reported as unknown and no size is shown for it

### Requirement: Sizes are stated in the application's language

Byte sizes SHALL be formatted with the unit names of the active language, using
binary units as the source listing does, and the same rule SHALL apply wherever
a size is shown.

#### Scenario: Reading a size in Russian

- **WHEN** the application is running in Russian and a file is 185.1 mebibytes
- **THEN** it reads `185.1 МиБ`, in the map list and in the download panel alike
