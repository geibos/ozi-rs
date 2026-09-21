## ADDED Requirements

### Requirement: Bundle progress is reported in the interface's language

Progress reported while a bundle is being opened SHALL reach the interface as a
translation key with its arguments, not as a finished sentence, so that it can
be shown in the language the interface is set to. The phase word shown beside
it SHALL be translated on the same terms.

The backend SHALL also send the English wording, and the interface SHALL show
that wording when it has no translation for the key, so that an untranslated
message degrades to English rather than to a key.

#### Scenario: A download watched in a Russian window

- **WHEN** a bundle download reports its progress and the interface is Russian
- **THEN** the message and the phase beside it are in Russian

#### Scenario: A message the interface does not know

- **WHEN** progress arrives with a key the dictionaries do not define
- **THEN** the backend's own wording is shown, and the key is not

#### Scenario: A bundle name that looks like a placeholder

- **WHEN** a message argument itself contains placeholder-shaped text
- **THEN** it appears in the message unchanged
