## Why

`revive-ui-cycle` set out to restore the development cycle: a stand to look at
screens, fixtures generated from the core, a smoke gate that drives the real
application, a state page, a definition of done. All of that exists and is in
use — most of the defects found between 2026-09-20 and 2026-09-23 were found by
walking the stand, and two desktop journeys run green.

Three pieces of it do not exist, and leaving them inside a change that is
otherwise finished would mean either archiving requirements the code does not
meet, or holding the finished work out of the baseline to keep company with the
unfinished. This change carries the three, so `revive-ui-cycle` can close on
what it delivered.

None of the three is visible to an operator. They are the instrument, not the
product, which is why they came last.

**The screenshot matrix.** Every screen × five states × two languages × two
themes, rendered and compared against a committed baseline. Today a screen is
looked at by walking it, which finds behaviour but not a row that grew four
pixels. The `ui-shell` requirement that every screen declares its five states
and the `ci-pipeline` requirement that CI compares the baseline belong with it.

**The view-model layer.** Components read the application state directly and
the map's behaviour is one 1800-line component. `$lib/actions/modes.ts`, added
on 2026-09-23 when the mode chips were wired, is the first piece of it: one
place that owns what entering and leaving a mode means. The rest — `vm/`
modules per area, the map composed from attachable pieces, the legacy flags
derived and then deleted — is a refactor no operator can see, and it is worth
doing only deliberately.

**A smoke journey per Customer Journey.** Two of eight have one. Six are
blocked on two things: a file dialog the Mac2 driver can answer, and a project
loaded at launch. `tools/ozi-rs-mcp/tests/smoke_core_workflow.rs` carries a line
each saying which. Cropping the Appium screenshot to the window belongs here
too — a screenshot of the whole display is evidence nobody can read.

## What Changes

Nothing yet: this is the ledger for the three pieces, kept out of the baseline
until they are built, with the requirements that describe them.

One requirement moves here rather than being archived as met: "Slice equals
branch equals pull request". Every slice since July has gone to `main` directly,
because the owner is the only committer and a PR against yourself buys review
from nobody. What the rule was for — the five checks before a slice is done — is
now `AGENTS.md` § "Before a slice is done" and the PR template. Revisit when a
second person commits.

## Impact

- Affected specs: `visual-verification`, `frontend-view-model`, `ci-pipeline`,
  `ui-shell`, `agent-workflow`
- Affected code: none yet
