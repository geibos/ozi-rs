## Why

CJ-8 is a shift change: the other headquarters, or the coordinator going off
duty, sends their `.ozp`, and the one taking over opens it, adds their own
day, and sends it back. It had never been walked. Walking it on 2026-09-23
found two things, both in the same place — the moment the file lands.

**Opening a project left the operator on the launcher.** Opening a *bundle*
closes the launcher and goes to the workspace; opening a `.ozp` did not. So a
coordinator handed a colleague's project chose it, and went on looking at the
catalogue of searches, with the project loaded behind the screen and nothing
whatever to say so. The same silence as an export that wrote a file and said
nothing.

**The recent list was a snapshot.** `BundleLoader` read `getRecentProjects()`
into a plain `const` at component init, which in Svelte 5 is read once. The
project just opened from that very screen did not appear in the list on it —
and that screen is also the Sheet that opens over the workspace, where the
component can stay mounted for a whole session.

Telling the launcher whether the file landed could not be done from outside:
a cancelled dialog and a file that would not read both leave the stores as they
were, and so does re-opening the project that is already open, which is exactly
what the recent list is for. So the action answers.

## What Changes

- Opening a saved project from the launcher — from the dialog or from the
  recent list — closes the launcher and opens the workspace, as opening a
  bundle does. A cancelled dialog or a file that would not read leaves the
  operator where they were, with the error they were given.
- `openProjectFile` answers whether a project was opened.
- The recent list is a store, so a screen showing it shows today's.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/lib/actions/project.ts`, `src/lib/recent-projects.ts`,
  `src/components/BundleLoader.svelte`, `src/test/stand/tauri-core.ts`
