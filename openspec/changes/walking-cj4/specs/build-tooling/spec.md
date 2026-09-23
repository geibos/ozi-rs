## ADDED Requirements

### Requirement: The stand answers every command the frontend can send

Every command in the generated bindings SHALL have an answer on the stand, or
be listed with the reason it is answered elsewhere.

A command with no answer is a screen nobody can walk. The stand refuses one
loudly, which is right, but nobody finds out until they reach the screen that
sends it — and the whole of the track-editing journey was unwalkable that way
without anything saying so.

An answer SHALL change what is on screen rather than report acceptance: a
stand that answers "accepted" tells the same lie as one that answers nothing.

#### Scenario: A command added without a stand answer

- **WHEN** a new command reaches the generated bindings and the stand has no answer for it
- **THEN** the test suite fails, naming the command
