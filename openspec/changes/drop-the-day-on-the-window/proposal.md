## Why

CJ-3 asks for drag and drop by name, and it was never built. A day's
recordings arrive as a handful of files from several navigators, and dropping
them on the window is how anybody expects to hand them over. Instead the
operator opens a picker, navigates to the folder, multi-selects — every time,
at four in the morning.

The dispatch that decides which reader a file goes to also lived inside the
Tracks tab, as a chain of `endsWith` in a component. That is how the picker
came to accept `.wpt` while nothing else did, and it is what two source-text
tests were pinning: that the component's text contains `endsWith(".plt")`.

## What Changes

- Files dropped on the window are imported, through the same dispatch the
  picker uses. One implementation, in `$lib/actions/import-paths`.
- Anything dropped that is not one of the four known extensions is taken as a
  folder and goes through the recursive import — which is what a memory card
  handed over actually is.
- The two source-text assertions become a behavioural test: a `.plt` reaches
  the PLT reader, a failure does not stop the rest of the day, and a failed
  file is named by its base name rather than its whole path.
- The stand answers the webview's drag-drop hook, so the drop can be walked
  here rather than only in the packaged application.

## Impact

- Affected specs: `track-import`
- Affected code: `src/lib/actions/import-paths.ts` (new),
  `src/components/library/TracksTab.svelte`, `src/routes/+layout.svelte`,
  `src/test/stand/tauri-webview.ts` (new), `vite.stand.config.ts`
