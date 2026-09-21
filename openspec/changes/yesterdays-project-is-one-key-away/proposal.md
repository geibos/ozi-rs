## Why

Reopening yesterday's search meant finding the file in a dialog again. ADR-0020
declares a recent-projects list in scope; what existed was a recent-*maps* list
— a bundle and one of its tiles, which answers a different question.

A crew comes back to the same search for days. The one they want is almost
always the one they had open last.

## What Changes

- `src/lib/recent-projects.ts` keeps the last eight project paths, newest
  first, upserted by path.
- Both saves record, not just the open dialog: a save is where a never-saved
  project first gets a path at all, and the project just written is the one
  most likely wanted next. A failed save records nothing.
- The command palette offers them, showing the file name with its path beneath.
- A path that no longer opens is dropped from the list and said so, rather than
  being left to fail every time it is chosen. Paths go stale — the file moved,
  the disk is not mounted, the crew is on the other machine.
- A corrupt or unreadable list returns empty instead of throwing: a broken
  convenience must not stop the palette opening.

Deliberately not merged with `recentFiles.ts`. The records differ and the
shared part is forty lines of storage plumbing around a working feature with
its own tests; if a third list appears, extract then.

## Impact

- Affected specs: `project-persistence`
- Affected code: `src/lib/recent-projects.ts` (new),
  `src/lib/actions/project.ts`, `src/components/CommandPalette.svelte`,
  `src/lib/i18n.ts`
