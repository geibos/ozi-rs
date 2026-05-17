## Why

After `migrate-to-sveltekit` puts the file-based routing skeleton in place and `add-design-tokens-and-shadcn` installs Tailwind, the Catppuccin→semantic-token mapping, and the shadcn-svelte primitive library, the existing nine panels under `src/components/` still render through ad-hoc native controls and per-file `<style>` blocks. The visual feel does not yet match the meetily-style design language we committed to in [`docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md`](../../../docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md). This change finishes the UI-kit axis by migrating every existing panel to shadcn-svelte primitives + Tailwind utility classes that consume the semantic tokens, while leaving the domain (track/waypoint RGBA colours, MapLibre internals) untouched.

This change **depends on** both:
- `2026-05-17-migrate-to-sveltekit` (Change 1 — provides `$lib` alias, `routes/project/+page.svelte` shell, ESLint/Prettier baseline);
- `2026-05-17-add-design-tokens-and-shadcn` (Change 2 — provides Tailwind, `--background`/`--foreground`/… semantic tokens, the primitive components under `src/lib/components/ui/`, `cn()`, `lucide-svelte`, `svelte-sonner`).

Neither dependency can be skipped: Tailwind utilities and the primitive imports do not exist without Change 2, and the `$lib`-pathed imports do not resolve without Change 1.

## What Changes

- Migrate the nine existing panels under `src/components/` to shadcn-svelte primitives in the order **Console → ThemePicker → SymbolPicker → SimplifyPanel → TracksPanel → WaypointsPanel → TrackPointsPanel → Sidebar → MapView**. Per component, replace native controls with the matching primitive, delete the component-local `<style>` block, and reach the same visual through Tailwind utility classes that read semantic tokens (`bg-background`, `text-foreground`, `border-border`, …).
- `TracksPanel` and `WaypointsPanel` keep their native `<input type="color">` for track/waypoint colour pickers. Track/waypoint colour belongs to the domain (`TrackStyle.color: [u8; 4]` RGBA, OziExplorer-compatible) and SHALL NOT be bound to `--ctp-*` palette variables nor to semantic tokens (design decision D6 of the parent design).
- `MapView.svelte` is migrated **wrapper-only**: container chrome moves to Tailwind utilities, but MapLibre initialization, source/layer setup, drag handlers, click handlers, and the tile-protocol code are not touched.
- Replace text/emoji icons with `lucide-svelte` icons **where it improves clarity** (audit per component, not a blanket replace). The SymbolPicker emoji set is data, not UI chrome, and stays as-is.
- Route any toast notifications through `svelte-sonner` (the `<Toaster />` host lives in `routes/+layout.svelte` from Change 2).
- Dynamic values from the domain (e.g. a track's `color`, a per-row width) continue to be expressed through inline `style=` attributes; only theme-driven static styling moves to Tailwind utilities.
- Update `docs/frontend-architecture.md` and `docs/project-map.md` to describe the post-migration panel layout.
- Visual smoke pass via `ozi-rs-mcp` screenshots across all four Catppuccin flavours (Latte / Frappé / Macchiato / Mocha) after each panel migration and once more at the close of the change.

## Impact

- Affected capabilities:
  - `ui-shell` — MODIFIED: "All in-app panels render through shadcn-svelte primitives and Tailwind utility classes that consume semantic tokens; component-local `<style>` blocks are removed except for dynamic per-feature values from the domain."
- Affected code (implementation):
  - `src/components/Console.svelte`, `…/ThemePicker.svelte`, `…/SymbolPicker.svelte`, `…/SimplifyPanel.svelte`, `…/TracksPanel.svelte`, `…/WaypointsPanel.svelte`, `…/TrackPointsPanel.svelte`, `…/Sidebar.svelte`, `…/MapView.svelte` — primitive migration, `<style>` block removed (except dynamic inline `style=`)
  - `src/routes/project/+page.svelte` — only if the Sidebar/MapView wrappers move there (no behavioural change)
  - `docs/frontend-architecture.md`, `docs/project-map.md` — updated to describe primitive-based panels
- Out of scope (separate changes / future work):
  - Settings, Bundle Manager, History routes (Axis A backlog)
  - Bundle id in URL
  - Feature-grouped `src/lib/components/` (`sidebar/`, `pickers/`) restructure
  - Any axis other than visual feel (perceived speed, repo layout)
  - New components — only existing ones are migrated
- Project file (`.ozp`) format: unchanged. Domain models unchanged. Tauri command surface unchanged.
