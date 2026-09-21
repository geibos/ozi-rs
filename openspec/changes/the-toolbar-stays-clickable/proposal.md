## Why

Selecting a track opens the inspector, and opening the inspector put Save,
Undo, Redo and the command-palette trigger underneath it. Not visually crowded
— unreachable: at 1024×640 with a track selected, `document.elementFromPoint`
at the centre of each of those four controls returned the inspector's header,
so a click on Save went to the inspector.

The cause is one missing declaration. `.canvas-column` is a grid with
`grid-template-rows` and no `grid-template-columns`, so its single implicit
column is sized `auto`, which resolves to max-content. The context bar is wider
than the canvas whenever the inspector is open, so the column grew to the bar's
width — 742px inside a 384px track — and the overflow ran under the inspector,
which is painted after it.

It is not an edge case of a small window. The bar needs about 710px for the
inert mode chips plus the actions; with the library rail and the inspector
open, a 1280px laptop leaves it 640px.

## What Changes

- The canvas column declares `minmax(0, 1fr)`, so the bar shrinks to the
  column instead of the column growing to the bar.
- A narrow bar sheds the inert mode chips first, then the words on the two
  labelled controls, and never a control: the actions are `flex-shrink: 0`.
- The breakpoints are container queries on the bar itself, because what takes
  the width away is the library rail and the inspector, not the window.

## Capabilities

### Added Capabilities
- `ui-shell`: the workspace actions stay reachable at every width.

## Impact

- **Frontend**: `src/components/WorkspaceShell.svelte` (styles only).
- **Backend**: none.
- **Risk**: low, and the failure mode it replaces is total. `@container` needs
  Safari 16 / WKWebView on macOS 13, which the macOS-first scope allows.
