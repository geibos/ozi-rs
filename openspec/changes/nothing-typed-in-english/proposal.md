## Why

Having found English literals on the library rows, I swept the rest. There
were more, and one of them is on the track-editing path a crew uses in the
field: the map's point context menu read **"Delete Point"** and **"Insert Point
After"**. Also English: the inspector rail's heading and its empty state, the
console's title, the symbol picker's "none", the theme pack's label and hint,
the collapsed rails' placeholders, the track tab's live-preview label, and the
two inspector sections' accessible names.

That is the second sweep in a day for one defect: a string typed straight into
a component. The reason it keeps happening is that nothing catches it —
`aria-label`, `title` and `placeholder` are invisible while writing markup, and
a visible string only shows itself if someone opens that screen in Russian.

## What Changes

- Every remaining literal above comes from the dictionaries.
- A test scans every component for a literal `aria-label`, `title` or
  `placeholder` whose value contains a Latin letter, and fails on it. Anything
  computed, translated or interpolated passes, so the test constrains only
  where a label comes from, not how it is built. It carries an allowlist of one
  entry, with its reason.

## Impact

- Affected specs: `ui-shell`
- Affected code: `src/lib/i18n.ts`, `src/components/MapView.svelte`,
  `src/components/InspectorRail.svelte`, `src/components/Console.svelte`,
  `src/components/SymbolPicker.svelte`, `src/components/ThemePicker.svelte`,
  `src/components/WorkspaceShell.svelte`,
  `src/components/library/TracksTab.svelte`,
  `src/components/inspector/MapInspector.svelte`,
  `src/components/inspector/TrackInspector.svelte`
