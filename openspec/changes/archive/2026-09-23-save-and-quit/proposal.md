## Why

Closing a window with unsaved work asked «Выйти без сохранения?» through the
operating system's confirm box, and that box has two buttons: quit, and cancel.

The answer an operator almost always wants at that moment — save, then quit —
was not among them. They had to cancel, find Cmd+S, and close again, at the one
point in a session when they are already leaving and least likely to be
careful. CJ-7 has carried «Осталось: „Save and quit" в close-guard» since July.

Three answers need a dialog of our own, which is also the chance to get the
dismissal right: Escape, the overlay and the corner cross all mean stay, which
is what a dismissed dialog should mean when the alternative is losing a night's
work.

## What Changes

- The close guard offers three answers: save and quit, quit without saving,
  and stay.
- Saving that does not land keeps the window open. A failed save says so
  already; a save-as the operator cancelled leaves the work unsaved, and
  quitting then would lose it just as surely.
- Dismissing the dialog means stay.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/components/CloseGuard.svelte` (new), `src/lib/stores.ts`,
  `src/routes/+layout.svelte`, `src/lib/i18n.ts`
