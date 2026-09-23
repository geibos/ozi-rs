# Tasks

- [x] 1.1 `askBeforeClosing()` / `resolveCloseGuard()` in `stores.ts`: the
      handler must `preventDefault()` before its first await, so the question
      is a promise it can wait on afterwards.
- [x] 1.2 `CloseGuard.svelte`: the safe answer first, the destructive one
      plainly named, dismissal meaning stay.
- [x] 1.3 The layout's close handler asks, saves when asked, and quits only
      once the project is clean.
- [x] 1.4 `cj7-continuity.test.ts` updated: it pinned the two-button confirm,
      including the `dialog:allow-confirm` permission the guard no longer needs.
- [x] 1.5 Walked on the stand: three answers offered; cancelling keeps the
      window; saving runs `save_project`, clears the indicator and then
      destroys the window.
- [x] 1.6 `just ci` green.
