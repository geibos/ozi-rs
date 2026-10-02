## ADDED Requirements
### Requirement: CI verifies fixtures and the screenshot baseline

CI SHALL fail when the committed frontend fixtures differ from what the Rust core generates — the `fixtures_are_up_to_date` test in the Rust suite of "Lint & test" — and SHALL run a job that renders the screenshot matrix in the same container image as `just shots` and compares it with the committed baseline under `src/test/stand/baseline/`, uploading the rendered matrix and any diff images as a workflow artifact whether it passes or fails. The jobs SHALL run on pull requests and on `main`.

#### Scenario: Stale fixtures fail CI

- **WHEN** a change alters a DTO without regenerating fixtures
- **THEN** the Rust tests fail on `fixtures_are_up_to_date` with a diff of the changed fixture file

#### Scenario: Matrix is inspectable

- **WHEN** the screenshot job finishes, passing or failing
- **THEN** the workflow run exposes an artifact containing every rendered PNG and, on failure, the diff images
