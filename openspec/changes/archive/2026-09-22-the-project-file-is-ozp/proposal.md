## Why

The project format is `.ozp`. `AGENTS.md` says so, `docs/project-map.md` says
so, and `project-persistence` requires a save "to a user-chosen `.ozp` file".
Both dialogs filtered on `json`.

Two consequences, and the second is the one that costs a crew something:

- What they saved was not the format the documentation names.
- An `.ozp` was **invisible** in the open dialog. A filter hides what it does
  not match, so a project from an older build, from a colleague, or from a USB
  stick simply was not there to select.

The divergence was known — `project-persistence`'s own Purpose carries a
reality note about it — and owner decision 1 says the code is fixed rather than
the spec weakened where the owner wants the behaviour.

## What Changes

- `src/lib/project-file.ts` holds the extension. There were two dialogs and no
  shared value, which is how they came to disagree with the documentation and
  agree with each other.
- Save offers `.ozp`.
- Open accepts `ozp` first and `json` still. Everything this app has saved
  until now carries `json`, and a filter that hid those would lose a crew their
  work far more surely than the wrong extension ever did.
- The reality note in the capability's Purpose goes, since it is no longer real.

## Impact

- Affected specs: `project-persistence`
- Affected code: `src/lib/project-file.ts` (new), `src/lib/actions/project.ts`,
  `src/components/CommandPalette.svelte`
