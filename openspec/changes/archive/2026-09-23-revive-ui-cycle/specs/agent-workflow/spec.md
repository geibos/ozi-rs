## ADDED Requirements

### Requirement: The repository carries a single state page for agents

`docs/STATE.md` SHALL answer three questions in this order: where the project is (last merged slice and date), what the next slice is (name and link to its OpenSpec change or roadmap row), and what is known to be broken. `AGENTS.md` and `CLAUDE.md` SHALL instruct agents to read it first and update it last in every session. The page SHALL stay under 60 lines; history lives in git.

#### Scenario: Resuming after a pause

- **WHEN** an agent starts a session after a month without activity
- **THEN** reading `docs/STATE.md` alone identifies the next slice and the known-broken list without consulting chat history

### Requirement: Progress is recorded as a screenshot gallery in the repository

`docs/progress/README.md` SHALL be a reverse-chronological gallery; each merged slice SHALL add one entry with date, slice name, one-line outcome, and links to before/after screenshots stored in `docs/progress/<date>-<slice>/`. Screenshots SHALL come from `just shots` for UI slices and from the cropped Appium screenshot for desktop-only slices.

#### Scenario: Gallery entry per slice

- **WHEN** a slice PR is merged
- **THEN** the gallery has a new topmost entry whose screenshots exist at the linked paths

### Requirement: Slices have a fixed Definition of Done

A slice SHALL be considered done only when all of the following hold: (1) gallery entry with before/after screenshots exists; (2) new code is covered by unit or component tests and adds no test that reads component source with `readFileSync`; (3) when the slice touches a Customer Journey, that journey's smoke test passes; (4) `just ci` passes; (5) the change is merged to `main` through a PR, `main` is pushed, and `docs/STATE.md` is updated. The PR template SHALL list these five items as checkboxes.

#### Scenario: Missing evidence blocks the merge

- **WHEN** a slice PR lacks the gallery entry
- **THEN** the review checklist marks the PR as not done and it is not merged

### Requirement: GUI-hijacking verification runs once per session

Test runs that take over the screen (Appium Mac2 smoke, live app launches) SHALL be batched into one run at the end of a session, after all non-GUI verification (`just ci`, `just shots`) has passed. Agents SHALL stop the WebDriver session, quit the app and kill leftover WebDriverAgent processes afterwards.

#### Scenario: One GUI batch

- **WHEN** a session produces three UI changes
- **THEN** the smoke suite runs once, after the third change, and no app or WebDriverAgent process remains afterwards

### Requirement: The native QA MCP server is always built from source

`.mcp.json` SHALL start `ozi-rs-mcp` through `cargo run --quiet -p ozi-rs-mcp --` (as `opencode.json` already does) so agents never run a stale prebuilt binary.

#### Scenario: Source change is picked up

- **WHEN** `tools/ozi-rs-mcp/src/appium.rs` changes and a new agent session starts
- **THEN** the MCP server that answers the session includes the change without a manual rebuild step

### Requirement: Verification reports state a falsifiable claim and link evidence

Before running desktop verification an agent SHALL state the concrete claim it is verifying as an observable outcome (for example "clicking Maps… opens a list with at least one row"), not a summary such as "Maps works". The report or PR description SHALL repeat that claim, the verdict, and the absolute paths of every artifact the claim rests on (screenshots, logs, smoke test output). Artifacts SHALL come from a run performed for that claim in the current session; files left under `.sisyphus/evidence/` by earlier sessions SHALL NOT be cited as evidence for new work.

#### Scenario: Vague claim is not acceptable

- **WHEN** a report says "the Maps button now works" and lists no artifact paths
- **THEN** the review checklist marks the change as unverified and it is not merged

#### Scenario: Paths are openable

- **WHEN** a reviewer opens each artifact path listed in the report
- **THEN** every path exists and shows the state the claim describes

### Requirement: Two failed verification attempts end in a diagnostic hand-back

An agent whose desktop verification fails MAY diagnose, change the code and run the full protocol once more, after copying the first attempt's artifacts out of the evidence root (the tools overwrite fixed paths). If the second attempt also fails, the agent SHALL stop instead of starting a third attempt and SHALL hand back to the owner with: the original claim, the failing step, both attempts' artifact paths, the current hypothesis and the proposed next step phrased as a question. A third attempt requires an explicit instruction from the owner.

#### Scenario: Second failure stops the loop

- **WHEN** the same smoke step fails again after one code change
- **THEN** the agent's turn ends with the diagnostic hand-back and no further code change or app launch

#### Scenario: First attempt's evidence survives

- **WHEN** the agent starts the second attempt
- **THEN** the screenshots and logs of the first attempt are still available at the paths later listed in the hand-back
