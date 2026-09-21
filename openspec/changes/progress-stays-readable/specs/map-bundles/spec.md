## ADDED Requirements

### Requirement: The download progress panel stays readable while a toast is up

Transient notifications SHALL NOT obscure the bundle download progress panel.
While the panel is on screen, the notification viewport SHALL be positioned
clear of it, and SHALL return to its usual position once the panel is gone.
The clearance SHALL follow the panel's measured height, so that a panel listing
many files is cleared as fully as a panel listing one.

#### Scenario: A file fails while the bundle is still downloading

- **WHEN** a bundle download is in flight AND a notification reports that one file failed
- **THEN** the notification is drawn clear of the progress panel, and the panel's title, cancel control and file/byte counters stay visible

#### Scenario: Nothing is downloading

- **WHEN** no download is on screen AND a notification appears
- **THEN** it occupies its usual corner, at the notification library's own edge offset

#### Scenario: The download finishes while a notification is up

- **WHEN** the panel disappears because the download finished
- **THEN** the notification returns to its usual corner rather than staying lifted
