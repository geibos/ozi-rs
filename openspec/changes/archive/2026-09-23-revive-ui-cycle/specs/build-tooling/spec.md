## ADDED Requirements

### Requirement: Stand, fixtures and screenshot recipes are available through `just`

The `justfile` SHALL provide: `just stand` (browser stand on the mock transport), `just fixtures` (regenerate `src/test/fixtures/` from the Rust core), `just shots [--screen <id>] [--compare|--update] [--slice <name>]` (Playwright screenshot matrix) and `just smoke [<N>]` (all CJ smoke tests or one). `just ci` SHALL include the fixture conformance test and `just shots --compare`; it SHALL NOT include Appium smoke.

#### Scenario: Recipes are listed

- **WHEN** `just --list` runs
- **THEN** `stand`, `fixtures`, `shots` and `smoke` appear with one-line descriptions

#### Scenario: CI recipe stays GUI-free

- **WHEN** `just ci` runs on a machine without Appium or an app bundle
- **THEN** it completes without attempting to launch the app or a WebDriver session
