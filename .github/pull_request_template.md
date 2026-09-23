## What this changes

<!-- One or two sentences: what an operator can now do, or what stopped being
     wrong. Not a list of files. -->

## Definition of done

Every slice answers all five. Tick what is true; strike out with a reason what
is not — an unticked box with no reason blocks the merge.

- [ ] **Gallery entry.** `docs/progress/<date>-<slice>/` holds before and after
      screenshots of the screens this touched.
- [ ] **Tests on behaviour.** New code is covered by unit or component tests
      that check the result. A test that greps the source is not coverage for
      new code.
- [ ] **The journey is green.** If this touches a Customer Journey, that CJ's
      smoke passes and asserts the outcome, not the presence of a button.
- [ ] **`just ci` green.** Locally, before the PR — GitHub Actions runs the
      same gates and will block otherwise.
- [ ] **`docs/STATE.md` updated.** What changed and what is next, so the work
      can be picked up from anywhere.

## For a behavioural change

- [ ] An OpenSpec change describes it, and `openspec validate <change> --strict`
      passes. New decisions go in the change's `design.md` and the capability's
      decision history — not in a new ADR file.
