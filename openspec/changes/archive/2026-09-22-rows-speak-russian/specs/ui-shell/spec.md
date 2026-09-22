## ADDED Requirements

### Requirement: Accessible names and tooltips are localized

Every control's tooltip and accessible name SHALL come from the interface's
dictionaries, not from a literal in a component, including the library rows'
visibility toggle, row menu and colour swatch, and the shell's landmark
regions. An accessible name is what a screen reader speaks and what the
platform's automation reads, so leaving it in one language is leaving the
application in that language.

A name substituted into a label SHALL NOT be required to take a grammatical
case the interface cannot give it.

#### Scenario: A row read in a Russian window

- **WHEN** the interface is Russian and a track row is read out or inspected
- **THEN** its visibility control, its menu and its colour swatch are named in Russian

#### Scenario: A track name inside a label

- **WHEN** a track's name appears inside a control's label in Russian
- **THEN** the label reads correctly without declining the name
