## ADDED Requirements
### Requirement: Screenshot matrix renders every registered screen state

The repository SHALL provide `just shots`, which drives the stand with Playwright and captures one screenshot per entry of a screen registry (`src/test/stand/screens.ts`) crossed with state (`empty`, `loading`, `loaded`, `error`, `overflow`), locale (`ru`, `en`) and theme (`light`, `dark`). Screenshots SHALL be deterministic: fixed viewport, bundled fonts, animations disabled. Output SHALL go to `docs/progress/<date>-<slice>/` when a slice name is given and to a temporary directory otherwise.

#### Scenario: Full matrix for one screen

- **WHEN** `just shots --screen library-tracks` runs
- **THEN** twenty PNG files are produced (5 states × 2 locales × 2 themes) and each file name encodes screen, state, locale and theme

#### Scenario: Registry entry without a fixture state fails

- **WHEN** a screen registry entry declares the `overflow` state but no fixture provides it
- **THEN** `just shots` exits non-zero naming the screen and the missing state

### Requirement: Screenshot baseline comparison detects visual regressions

`just shots --compare` SHALL compare the freshly rendered matrix against the committed baseline under `src/test/stand/baseline/` with a per-pixel tolerance and SHALL exit non-zero on any difference, writing diff images next to the report. `just shots --update` SHALL rewrite the baseline and SHALL be run only in a dedicated commit.

#### Scenario: Unintended change is caught

- **WHEN** a change alters the Library row height and `just shots --compare` runs
- **THEN** the command exits non-zero, lists the affected screenshots and writes diff images

#### Scenario: Intended change is accepted explicitly

- **WHEN** the developer runs `just shots --update` and commits the baseline
- **THEN** the next `just shots --compare` passes

### Requirement: Every Customer Journey has an Appium smoke scenario

`tools/ozi-rs-mcp/tests/` SHALL contain one `#[ignore]`d smoke test per Customer Journey listed in `docs/customer-journeys.md` (CJ-1 … CJ-8), named `smoke_cj<N>_<slug>`, each driving the bundled app through the journey's happy path and asserting on the journey's OUTCOME, not merely on the accessibility tree: a written file that exists and re-imports to equal data (CJ-3, CJ-6), a project that reopens with identical content after save, kill and relaunch (CJ-7, CJ-8), tiles served from disk with networking disabled (CJ-2), a cleaned track whose point count and segment count changed as expected (CJ-4). A smoke test whose only assertions are on UI element presence SHALL be rejected in review. `just smoke` SHALL run all of them; `just smoke <N>` SHALL run one. Smoke tests SHALL kill leftover app and WebDriverAgent processes in preflight and teardown. Until a journey's UI exists, its smoke test SHALL exist and be marked `#[ignore = "pending: <reason>"]` so the gap is visible in the test list.

#### Scenario: Smoke list is complete

- **WHEN** `cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml -- --list --ignored` runs
- **THEN** eight `smoke_cj*` tests are listed, each mapped to one CJ

#### Scenario: Teardown leaves no processes

- **WHEN** a smoke test fails mid-journey
- **THEN** no `ozi-rs.app` or `WebDriverAgentRunner` process remains after the test returns

#### Scenario: Smoke asserts an outcome

- **WHEN** `smoke_cj6_export_roundtrip` exports a track to PLT through the UI
- **THEN** the test re-imports the written file with the Rust importer and asserts equal point count, segment count and track name; the test does not pass on the presence of an "Exported" toast alone

### Requirement: Appium screenshots are cropped to the application window

`appium_screenshot` in `tools/ozi-rs-mcp` SHALL decode the WebDriver base64 payload to PNG and SHALL crop it to the bounds of the `ozi-rs` window obtained from the window server, so evidence never includes the owner's desktop.

#### Scenario: Screenshot contains only the app window

- **WHEN** `appium_screenshot` is called with the app window occupying a quarter of the screen
- **THEN** the written PNG has the window's pixel dimensions and no other window or desktop content

