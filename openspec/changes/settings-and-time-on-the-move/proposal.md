## Why

A track's statistics carry one time: the span from its first point to its last.
For a recording that ran overnight that reads «26д 5ч» beside «4.9 км», and
even for one day it counts every rest stop and every wait at the car as
walking. What a coordinator asks is how long the crew was actually moving, and
how much they climbed.

Both need a threshold, because GPS lies in both directions when nothing is
happening: a standing navigator's position wanders by five to twenty metres,
and GPS elevation by five to ten. Without a threshold a rest stop is slow
walking and the climb is the sum of the noise. The owner decided on
2026-10-01: a stop is less than 25 m moved in two minutes, a rise or fall
counts from 5 m, and both are settings.

That needs somewhere for settings to live. There is none: the theme is a list
of seven entries in the command palette, the language a two-letter button in
the status bar, and `ThemePicker.svelte` — which `ui-shell` has required to be
reachable "from settings" — is mounted nowhere. The owner left its place to
us; it goes on the settings screen, which FTP accounts will need next.

## What Changes

- A settings screen, opened from a gear in the top bar, from the palette and
  with ⌘, — appearance (theme and language) and track statistics (the stop
  and the climb thresholds). Values persist on the machine.
- The track inspector shows time on the move beside the span, and ascent and
  descent, computed from the track's points with the thresholds from settings.
- A track without timestamps shows no moving time; a track without
  elevations, no ascent or descent.

## Impact

- Affected specs: `ui-shell`, `track-display`
- Affected code: `src/components/Settings.svelte` (new),
  `src/lib/track-motion.ts` (new), `src/lib/settings.ts` (new),
  `src/components/ThemePicker.svelte`, `src/components/WorkspaceShell.svelte`,
  `src/components/CommandPalette.svelte`, `src/components/inspector/TrackInspector.svelte`,
  `src/routes/+layout.svelte`, `src/lib/i18n.ts`
