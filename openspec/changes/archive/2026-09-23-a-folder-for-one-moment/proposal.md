## Why

The owner asked, on 2026-09-23, for a way to capture a screenshot and the
application's diagnostics at the moment something is wrong, into a folder, so a
day's worth can be handed over at once.

The reason is the gap between noticing and describing. Between a coordinator
seeing something odd and writing it down later, almost everything useful is
lost: what the screen looked like, which map was open, what the last error
said, whether the project had unsaved work, what the application thought its
status was. By the time it becomes a message it is "the import did something
odd", which nobody can act on — and the person who could answer it is not in
the room and cannot ask.

This also answers a question that came up the same day: what form of feedback
is usable. A recording is not, directly; a folder of moments is.

## What Changes

- Shift+Cmd/Ctrl+D, and two entries in the command palette, capture a moment
  into `<Documents>/ozi-rs-отчёты/<date>_<time>/`: the window as a picture, the
  diagnostics as text, the whole application state as JSON, and what build on
  what machine.
- The description is asked for **after** the capture, as an action on the
  toast. The screen is photographed the instant the operator asks — before a
  dialog can cover it — and the sentence comes when there is a second to write
  it. A report with no sentence is still worth having.
- Diagnostics carry a time. Two hundred messages in an order with no times
  cannot tell a reader whether the error was before the screenshot or hours
  earlier.
- A screenshot that cannot be taken does not fail the report, and the toast
  says which permission is missing. The rest is most of the value.

**Not behind a debug flag**, which the owner raised as a possibility. The
strangeness happens in the field, on a release build, at four in the morning; a
report that exists only in builds somebody runs specially is a report that
never exists.

## Impact

- Affected specs: `agent-workflow`
- Affected code: `src-tauri/src/commands/report.rs` (new),
  `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`, `src/lib/actions/report.ts` (new),
  `src/components/ReportNote.svelte` (new), `src/components/CommandPalette.svelte`,
  `src/routes/+layout.svelte`, `src/lib/api.ts`, `src/lib/i18n.ts`
