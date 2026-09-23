## Context

This change exists because `revive-ui-cycle` finished most of what it set out
to do, and archiving it whole would have put four requirements into the
baseline that the code does not meet. The alternative — holding a stand, a
fixture generator, a smoke gate and a state page out of the baseline until a
view-model refactor is done — would have been worse.

## Goals / Non-Goals

- **Goal:** keep the three unbuilt pieces recorded, with the requirements that
  describe them, so they are neither forgotten nor pretended to be done.
- **Non-Goal:** schedule them. None is visible to an operator, and the order
  they are taken in is the owner's call.

## Decisions

### The pieces are separable, so they are separate tasks

The screenshot matrix, the view-model and the smoke journeys were one change
because they were one session's plan. They share nothing: the matrix needs
Playwright and a baseline, the view-model is a frontend refactor, the journeys
need the Mac2 driver to answer a file dialog. Any one can be done without the
others.

### The view-model starts from what exists rather than from the plan

The plan described `src/lib/vm/selection.ts` with a five-value
`interactionMode` and the legacy flags deleted. What exists after 2026-09-23 is
`$lib/actions/modes.ts` with four values and the flags still read by a dozen
components — built because the mode chips needed one place that knew what
entering a mode means, not because the refactor started.

That is the right seam and the wrong shape to pretend about: the requirement
moved here says what the layer must do, and the first task says to build on the
file that already does part of it.

### "Slice equals branch equals pull request" is not adopted

Every slice since July went to `main` directly. The owner is the only
committer; a pull request against yourself buys review from nobody and costs a
round trip per slice. What the rule was for is the five checks before a slice
is done, and those are in `AGENTS.md` and the PR template now. The requirement
stays here rather than being archived as met, because it is not met and might
be wanted when a second person commits.

## Risks / Trade-offs

- [The matrix is deferred indefinitely and a visual regression ships] → the
  stand is walked on every slice and the smoke gate drives the real
  application; what is missing is the four-pixel class of change, which is the
  cheapest kind to fix late.
- [The view-model is never done and `MapView.svelte` keeps growing] → it is
  1800 lines and every slice that touches it adds to the cost. The counter is
  that two of this week's defects were in that file and both were found by
  walking, not by reading.

## Migration Plan

None. Nothing here changes behaviour; the requirements enter the baseline when
the work is built.
