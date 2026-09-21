# The stand

The real frontend, served in a browser, with the Tauri transport answered from
the fixtures the Rust core writes.

```
just fixtures   # only when a DTO changed
just stand      # http://localhost:5273
```

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
handle anyway) and nothing emits events; `standEmit` in `tauri-event.ts` lets a
console session deliver one by hand.
