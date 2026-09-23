## ADDED Requirements
### Requirement: Slice equals branch equals pull request

Every slice SHALL be developed on its own branch and merged into `main` through a pull request whose CI is green. Direct commits to `main` SHALL be limited to documentation-only fixes. `main` SHALL be pushed to `origin` after every merge.

#### Scenario: Local and remote stay in sync

- **WHEN** a slice is merged
- **THEN** `git log origin/main..main` is empty within the same session

