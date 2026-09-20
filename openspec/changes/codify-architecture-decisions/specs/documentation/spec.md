## ADDED Requirements

### Requirement: Public docs claim only verified behaviour, in three explicit states

Public documentation (`README.md`, `AGENTS.md`, `docs/*.md`) SHALL describe every feature in exactly one of three states: implemented and reachable in the interface; backend capability without interface exposure (phrased "backend supports …, the UI does not surface …"); or planned / not implemented. A doc SHALL NOT call a feature "supported" when only the backend or only a plan exists, and every behavioural claim SHALL be backed by code, a test or a QA artefact.

#### Scenario: Vocabulary is prescribed in conventions

- **WHEN** `docs/conventions.md` § "Docs hygiene" is read
- **THEN** it prescribes the phrases "backend supports", "UI surfaces", "planned" and "not implemented" for divergences between backend capability and interface exposure

#### Scenario: Backend-only capability is labelled as such

- **WHEN** a registered IPC command has no caller in `src/`
- **THEN** its `docs/feature-status.md` row states the backend capability and says the interface does not surface it

#### Scenario: Unverifiable claim is downgraded

- **WHEN** a reviewer cannot point to code, a test or a `docs/qa/` artefact for a behavioural claim
- **THEN** the claim is rewritten as planned / not implemented or removed

### Requirement: Feature-status matrix has fixed columns and a bounded audit vocabulary

`docs/feature-status.md` SHALL be a table with the columns `Feature | Backend | UI | Docs | Status | Evidence | Audit-verified`. `Evidence` SHALL name a concrete artefact (test name, `docs/qa/smoke-*.md`, screenshot or QA log). `Audit-verified` SHALL be `pending` until the smoke has run and afterwards one of `works`, `partial`, `broken`, `hidden` or `missing`, optionally followed by a link to the smoke document. The "Registered backend commands" preamble SHALL list exactly the commands in the specta registry.

#### Scenario: Header row matches the contract

- **WHEN** the first table header of `docs/feature-status.md` is read
- **THEN** it is `| Feature | Backend | UI | Docs | Status | Evidence | Audit-verified |`

#### Scenario: Audit values stay in the vocabulary

- **WHEN** the `Audit-verified` cell of every row is stripped of its parenthesised link
- **THEN** each value is one of `pending`, `works`, `partial`, `broken`, `hidden`, `missing`

#### Scenario: Command preamble matches the registry

- **WHEN** the command names in the preamble are compared with the commands registered in `src-tauri/src/lib.rs` (`tauri_specta::collect_commands!` plus the `tauri::generate_handler!` tile block)
- **THEN** the two sets are identical

### Requirement: Navigation docs are dated and match the source tree

`AGENTS.md`, `docs/project-map.md`, `docs/architecture.md`, `docs/frontend-architecture.md`, `docs/feature-status.md`, `docs/requirements.md` and `docs/roadmap.md` SHALL carry a `> Last verified against code: YYYY-MM-DD` line directly under their title, refreshed whenever the document is re-verified. Every file path these documents cite SHALL exist, and `docs/commands-reference.md` SHALL list every registered IPC command and every `ProjectCommand` variant.

#### Scenario: Verification date is present

- **WHEN** line 3 of each listed document is read
- **THEN** it matches `> Last verified against code: \d{4}-\d{2}-\d{2}`

#### Scenario: Cited paths exist

- **WHEN** every repository path in `docs/project-map.md` is resolved
- **THEN** each path exists in the working tree

#### Scenario: Command reference is complete

- **WHEN** every command registered in `src-tauri/src/lib.rs` (`collect_commands!` and the `generate_handler!` tile block) is searched in `docs/commands-reference.md`
- **THEN** every registered command has a row

### Requirement: Decisions are recorded in OpenSpec, not in new ADR files

New architecture and product decisions SHALL be recorded in OpenSpec: the change's `design.md` states the decision, its rationale and rejected alternatives, and after archive the affected capability's `## Purpose` → `### Decision history` gains one bullet (source, date, status, decision, rationale, codified-as). `docs/adr/` is frozen at its 24 historical records; superseding an old decision SHALL be a decision-history entry, never a new ADR file. Normative requirements change only through change deltas under `openspec/changes/<change>/specs/`.

#### Scenario: ADR directory stays frozen

- **WHEN** `docs/adr/` is listed
- **THEN** it contains exactly 24 files and none is numbered above `adr-0024`

#### Scenario: A change that decides something documents it

- **WHEN** a change introduces or supersedes an architecture or product decision
- **THEN** its `design.md` records the decision and the delta for the affected capability is the only place the requirement text changes

#### Scenario: Purpose carries the decision after archive

- **WHEN** such a change is archived
- **THEN** `openspec/specs/<capability>/spec.md` § "Decision history" contains a bullet for the decision

### Requirement: OpenSpec specs are normative; feature-status is evidence

`openspec/specs/<capability>/spec.md` SHALL be the single normative source for behaviour. `docs/feature-status.md` SHALL record verification evidence and SHALL NOT redefine or override requirements; when the two disagree, the spec governs and the matrix is corrected through the QA workflow.

#### Scenario: Conflict resolution

- **WHEN** a feature-status row contradicts a requirement in the capability spec
- **THEN** the row is corrected (or the disagreement is raised as an OpenSpec change), and the spec text is not edited to match the matrix

#### Scenario: Agent guidance states the precedence

- **WHEN** `AGENTS.md` § "Behavioral changes via OpenSpec" is read
- **THEN** it states that `docs/feature-status.md` does not supersede OpenSpec specs
