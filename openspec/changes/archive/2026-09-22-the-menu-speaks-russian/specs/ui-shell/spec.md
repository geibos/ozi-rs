## ADDED Requirements

### Requirement: Text shown between the tags comes from the dictionary

A component SHALL take the words it renders as its own content — a menu item,
a button, a heading, an empty state — from the interface dictionary rather than
from the markup, and a test over the source SHALL enforce it. Units and standard abbreviations written identically in both languages
MAY be exempt, by name and with a stated reason.

A count SHALL be worded so that it needs no plural form, since Russian requires
three and the interface has no rule for choosing between them.

#### Scenario: The actions menu on a track row

- **WHEN** the operator opens a track's actions menu
- **THEN** every item, including the destructive one, is in the interface language

#### Scenario: A word typed into the markup

- **WHEN** a component renders a word of its own rather than a dictionary lookup
- **THEN** the build fails, naming the file and line

#### Scenario: A counted badge

- **WHEN** a badge shows how many points a drawing has
- **THEN** it is worded without a noun that would need to agree with the number
