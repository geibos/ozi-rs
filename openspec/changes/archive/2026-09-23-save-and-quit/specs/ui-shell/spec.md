## ADDED Requirements

### Requirement: Closing with unsaved work offers to save

When a window with unsaved work is asked to close, the system SHALL hold the
window open and offer three answers: save and then quit, quit without saving,
and stay. Dismissing the question — Escape, the overlay, the corner control —
SHALL mean stay.

Quitting SHALL happen only once the work is saved. A save that fails, and a
save the operator cancelled at the file dialog, SHALL leave the window open
with the work still in it.

#### Scenario: Save and quit

- **WHEN** the operator closes a window with unsaved work and chooses to save
- **THEN** the project is saved and the window closes

#### Scenario: A save that did not land

- **WHEN** the save fails, or the operator cancels the file dialog it opened
- **THEN** the window stays open and the work is still unsaved

#### Scenario: Quitting without saving

- **WHEN** the operator chooses to quit without saving
- **THEN** the window closes and nothing is written

#### Scenario: Dismissing the question

- **WHEN** the operator presses Escape on the question
- **THEN** the window stays open and nothing is written
