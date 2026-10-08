## ADDED Requirements

### Requirement: Each visible track is saved to its own PLT in 10-Tracks

The system SHALL offer to write every visible track to its own PLT file,
named after the track (standard п. 24), in the open search's `10-Tracks`
folder (п. 25), or in a folder the operator picks when no search folder is
open. When files of those names are already there, nothing SHALL be written
until the operator agrees to replace them. When tracks are not named by the
standard, the operator SHALL be told which before anything is written. Two
tracks that would write one file SHALL be refused before anything is
written. The result SHALL name how many files were written and where.

#### Scenario: A search's tracks saved

- **WHEN** three tracks are visible and one hidden, and the operator chooses
  «Сохранить в 10-Tracks»
- **THEN** three PLT files named after the tracks are in `10-Tracks`

#### Scenario: A colleague's file is in the way

- **WHEN** `10-Tracks` already holds a file named like one of the tracks
- **THEN** nothing is written until the operator agrees to replace it

#### Scenario: Two tracks, one name

- **WHEN** two visible tracks have the same name
- **THEN** the export is refused and no file is written
