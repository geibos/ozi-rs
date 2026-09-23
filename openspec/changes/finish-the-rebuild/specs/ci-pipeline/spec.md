## ADDED Requirements
### Requirement: CI verifies fixtures and the screenshot baseline

The CI workflow SHALL add a job that regenerates fixtures with `just fixtures`, fails if the result differs from the committed fixtures, runs the fixture conformance test, and runs `just shots --compare` against the committed baseline, uploading the rendered matrix and any diff images as a workflow artifact. The job SHALL run on pull requests and on `main`.

#### Scenario: Stale fixtures fail CI

- **WHEN** a PR changes a DTO without regenerating fixtures
- **THEN** the fixtures job fails with a diff of the changed fixture file

#### Scenario: Matrix is inspectable

- **WHEN** the screenshot job finishes, passing or failing
- **THEN** the workflow run exposes an artifact containing every rendered PNG and, on failure, the diff images
