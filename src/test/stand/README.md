# The stand

The real frontend, served in a browser, with the Tauri transport answered from
the fixtures the Rust core writes.

```
just fixtures   # only when a DTO changed
just stand      # http://localhost:5273
```

States:

- `http://localhost:5273/project` — a project open, two tracks, three waypoints
- `http://localhost:5273/?state=cold` — the app as a crew first sees it: the
  catalogue loaded, nothing open

## Why

Every visual check before this cost a full `just build`, an Appium session that
owns the screen and dims the display, and a window that had to be caught in the
right state before it moved on. Most of a working day went into that. The stand
opens the same screens in a browser tab in a second, at any size, with
devtools.

## What it proves and what it does not

It proves how a screen looks and behaves given a state: layout, truncation,
empty states, translations, what a click does to the DOM. The data is the same
bytes the backend sends, because the fixtures come from the same mappers the
commands use.

It proves nothing about the Rust side, the IPC boundary, tiles, file dialogs or
the packaged app. That is what `just smoke` is for (ADR-0024: Playwright is not
evidence for desktop integration). A change that touches the backend still
needs the smoke gate.

## How it answers

`src/test/stand/tauri-core.ts` holds the command table. A command with no
answer throws, with the command name and where to add it — a screen that
renders because a mock quietly returned `undefined` is the failure this whole
exercise exists to stop. Dialogs answer "cancelled" (the branch a screen must
handle anyway) unless a session asks otherwise —
`standAnswerDialogsWith("/tmp/day.gpx")` from `tauri-dialog.ts` makes the
pickers answer that path, which is the only way to look at import, export,
save-as or open-a-project on the stand, and `standCancelDialogs()` puts it
back. Nothing emits events; `standEmit` in `tauri-event.ts` lets a
console session deliver one by hand. Tile requests answer with a transparent
pixel: the stand has no tile store, and a wall of error toasts would hide the
screen it exists to show.

A handler must respect its arguments. Answering every layer with the same
waypoints listed each of them twice in the rail, which read as an app defect
until the call transcript showed two calls with different ids — the same class
of lie the hand-written mocks told.

## Failure states

```
http://localhost:5273/?fail=catalogue&state=cold   the listing is unreachable
http://localhost:5273/project?fail=download        the bundle download dies
```

Both are read once at load, because the app navigates between its own routes
and would otherwise lose the flag. `?fail=download` plays the same sequence as
a successful one and ends it with the error the backend really sends.
