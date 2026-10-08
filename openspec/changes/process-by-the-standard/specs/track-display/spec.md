## MODIFIED Requirements

### Requirement: Track name validation surfaces a non-blocking warning for non-conforming names

The system SHALL display a non-blocking warning indicator on tracks whose
names do not follow the cartographic standard, п. 14–15: eight digits of the
date, `_`, and a callsign of latin letters, digits, `_` and `-`
(`^\d{8}_[A-Za-z0-9][A-Za-z0-9_-]*$`). The backend SHALL remain permissive:
rename, save, and export operations SHALL NOT be blocked by a non-conforming
name.

#### Scenario: Non-conforming name shows warning but does not block

- **WHEN** a track is renamed to `temp` (does not match the pattern)
- **THEN** the Tracks panel shows a warning indicator on that track and the rename, save, and export operations still succeed

#### Scenario: Conforming name has no warning

- **WHEN** a track is renamed to `20240601_Ivanov`
- **THEN** the warning indicator is not shown

#### Scenario: A Cyrillic callsign is flagged

- **WHEN** a track is named `20260709-ЛИСА15`
- **THEN** the warning indicator is shown

## REMOVED Requirements

### Requirement: Validation pattern is alphabet-agnostic and does not validate calendar dates

**Reason**: The standard (п. 14) allows only latin letters in a track name;
the warning accepted Cyrillic because field files carry it, which is why
they need renaming, not a reason to call them correct.
**Migration**: The pattern in the requirement above; it still checks the
pattern only, not that the eight digits are a real date.

## ADDED Requirements

### Requirement: The standard's name is offered for a track

Where a standard name can be worked out for a track, the warning SHALL offer
it, and choosing it SHALL rename the track. The date SHALL be the local day
of the track's first timed point, or failing that a date written at the
start of its name; the callsign SHALL be the rest of the name with the group
word and its number joined (`Лиса 19` → `Lisa19`, `ветер2` → `Veter2`) and
other words transliterated as the standard's example spells them (`Мохнатый`
→ `Mohnatiy`), each capitalised, joined by `_`. Words that are no callsign
(`track`, `трек`, `gpx`, `file`) SHALL be dropped. With no date or no
callsign, nothing SHALL be offered. An action SHALL rename every track for
which a name is offered and say how many are left to name by hand.

When several tracks would take the same name — a group's navigator and its
phone — the names SHALL be numbered `_1`, `_2`… as п. 18 has it, and a name
another track already carries SHALL be numbered past.

#### Scenario: A crew's file name

- **WHEN** a track named `Лиса 19 Мина` has its first point at 00:45 on
  8 October, local time
- **THEN** the offered name is `20261008_Lisa19_Mina`

#### Scenario: One group, two files

- **WHEN** «Имена по стандарту» renames two tracks that both work out to
  `20261008_Lisa19`
- **THEN** they are named `20261008_Lisa19_1` and `20261008_Lisa19_2`

#### Scenario: Nothing to build from

- **WHEN** a track named `File` is offered a name
- **THEN** nothing is offered and the warning says what the standard wants

### Requirement: Tracks are coloured from the standard's palette

The track colour control SHALL offer the standard's colours — red, blue and
green for foot groups, pink for Ветра, yellow, light blue and white for
Борты and БПЛА (п. 20–21) — beside a free choice, SHALL say when a track is
black that black is reserved for tasks (п. 23), and SHALL offer to give the
chosen colour to every visible track at once.

#### Scenario: A search day in one colour

- **WHEN** the operator hides the previous day's tracks, picks blue for one
  track and chooses «Этот цвет — всем видимым трекам»
- **THEN** every visible track is blue
