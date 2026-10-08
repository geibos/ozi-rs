## ADDED Requirements

### Requirement: A search's results are sent to a results account

The system SHALL send the processed files of the open search — every `.plt`
and `.wpt` directly in its `10-Tracks` folder, and nothing from that folder's
subfolders — to a results account chosen by the operator, into
`<account folder>/<search folder>`, where the search folder is named as the
search's bundle is, `ГГГГ-ММ-ДД_Место`. Before sending, the operator SHALL
see the files and the folder they will go to. Files of the same name SHALL
be replaced. The result SHALL list the files sent and the folder; a failure
SHALL name the file and what the server said, and SHALL list the files sent
before it.

When no search bundle is open — a search with no map ordered — the operator
SHALL be able to pick the search's folder instead; its name is then the
folder on the server, and its `10-Tracks` holds the files.

A file or folder name holding a control character, or a file or search
folder name holding a slash, SHALL NOT be sent: a line break inside an FTP
command would end it and start another of the name's making.

#### Scenario: A search's results sent

- **WHEN** `10-Tracks` holds two PLT files, a WPT file and a subfolder of raw
  GPX, and the operator sends to a results account whose folder is
  `/results`
- **THEN** the three files are in `/results/<search folder>` and the raw GPX
  stays on the machine

#### Scenario: Nothing to send

- **WHEN** `10-Tracks` holds no PLT or WPT files
- **THEN** nothing is sent and the operator is told so

### Requirement: A search's folder is made on the server only when asked

When the search's folder is not under the account's folder, the system SHALL
send nothing and SHALL ask the operator whether to make it, saying that the
standard wants the coordinator's sanction for it (п. 33); only on a yes SHALL
it make the folder and send.

#### Scenario: The first upload of a search

- **WHEN** the search's folder is not on the server
- **THEN** nothing is sent until the operator agrees to make the folder

### Requirement: A file with the finding is named before it is sent

When a waypoint file to be sent holds a mark named `BVP`, `BVP1`, `BVP2`…,
the system SHALL name the file before sending and SHALL say that it goes to
the server only with the coordinator's sanction (п. 36).

#### Scenario: BVP among the marks

- **WHEN** `Waypoints_20261008.wpt` holds a mark named `BVP`
- **THEN** the operator is told so, by file name, before anything is sent

#### Scenario: A name that would end a command

- **WHEN** a file in `10-Tracks` is named with a line break followed by
  `DELE important.plt`
- **THEN** nothing is sent and the server receives no command at all
