## Why

Walking the workspace on the stand: the library rows — the ones a crew touches
most — were still English inside a Russian window. The eye button's tooltip
read "Hide", its accessible name "Hide 20260708_Veter2", the row menu
"Actions", the colour swatch "Track color". The landmarks around them said
"Library", "Map canvas", "Inspector".

The tooltip is visible text. The rest is what a screen reader speaks and, on
this project, what the customer-journey smoke matches on — `docs/STATE.md`
records that WKWebView publishes a control's `aria-label` rather than the text
inside it, so these strings are the automated gate's view of the app as well.

The Waypoints tab already passed a translated label into the row; the Tracks
and Maps tabs did not, and fell through to the English default written into the
component.

## What Changes

- The row's visibility tooltip, its accessible name, the row menu and the
  colour swatch come from the dictionaries.
- The shell's landmarks and the remaining static labels — pin the inspector,
  close the console, colour theme, the symbol picker's "none" — likewise.
- The Russian label is `Скрыть: {name}`, not `Скрыть {name}`. Russian declines
  the object of the verb, and a track name substituted raw stays nominative:
  "Скрыть точка отсечки" is what a crew would have read. The colon sidesteps
  the case for any name, including the ones that are dates and call signs.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/lib/i18n.ts`, `src/components/library/LibraryRow.svelte`,
  `src/components/library/TracksTab.svelte`,
  `src/components/WorkspaceShell.svelte`,
  `src/components/InspectorRail.svelte`, `src/components/Console.svelte`,
  `src/components/ThemePicker.svelte`, `src/components/SymbolPicker.svelte`
