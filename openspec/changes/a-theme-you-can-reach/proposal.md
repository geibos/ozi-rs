## Why

`ui-shell` has required a theme selector all along, and `ThemePicker.svelte`
has existed all along, mounted nowhere: the workspace redesign took away the
sidebar it lived in. What was left in its place was a command-palette entry
that raised a toast saying — in English, inside a Russian window — to use a
picker in a sidebar that is not there.

A setting the operator cannot reach is not a setting, and a message pointing
at a surface that was deleted is worse than no message.

## What Changes

- The palette lists the themes themselves, one entry each, with a tick beside
  the current one. Choosing one applies it and remembers it.
- The palette is where the other settings already are, and it is searchable,
  which a select in a corner is not.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/components/CommandPalette.svelte`
