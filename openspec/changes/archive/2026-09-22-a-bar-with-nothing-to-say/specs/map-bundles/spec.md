## ADDED Requirements

### Requirement: The status bar is sized to what it has to say

The launch screen's status bar SHALL occupy only the space its content needs.
With no download on screen it SHALL show the status line alone; when a download
begins it SHALL grow to hold the file name, the progress and the byte counts,
and SHALL return to one line when the download ends. It SHALL NOT render an
empty progress track, which reads as a broken control rather than as reserved
space.

#### Scenario: The application has just been launched

- **WHEN** the operator reaches the first screen and nothing is downloading
- **THEN** the status bar is a single line and no empty progress track is drawn

#### Scenario: A download begins

- **WHEN** a bundle download starts
- **THEN** the bar grows to report it, and does not resize again when the first byte counts arrive

#### Scenario: The download ends

- **WHEN** it finishes
- **THEN** the bar returns to a single line
