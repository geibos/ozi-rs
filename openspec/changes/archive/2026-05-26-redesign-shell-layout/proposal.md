## Why

The current shell is a literal port of OziExplorer chrome — a 65-button Win95 toolbar implied by the 9-menu menubar history, beveled gray surfaces, MDI floating panels (Tracks / TrackPoints / Waypoints over MapView), and Catppuccin as a default theme that does not match macOS or Windows native conventions. Users on a SAR laptop expect a modern native-feeling workspace closer to Gaia GPS or CalTopo, with one focused canvas and one sidebar of context. The existing two-column `/project` shell (`Sidebar.svelte` + 3 floating panels) cannot host that direction without first replacing the layout primitive itself.

This change lays the foundation: a three-pane workspace shell, native-tinted design tokens, Geist typography, and the two shadcn primitives (`Sheet`, `Command`) the next two changes will consume. It is the first of three sequential changes; library content and inspector content land afterwards.

## What Changes

- The `/project` workspace SHALL be restructured into a three-pane shell with named slots: `library-rail` (left, ~280px), `canvas` (center, MapView), `inspector-rail` (right, ~360px, on-demand). The current 2-column `Sidebar.svelte` + floating panel layout SHALL be replaced. For this change the two rail slots render empty placeholders — they are filled by the follow-up changes `library-sidebar` and `inspector-pane`.
- A thin top context-bar SHALL be added above the canvas with mode-chip placeholders (View / Draw / Edit / Measure) and a Cmd-K palette trigger button. A status bar SHALL be added below the canvas. Both are visual scaffolding only; their content wiring is partially deferred to `library-sidebar` (status text) and `inspector-pane` (Cmd-K invocation behavior). The status bar SHALL preserve the stable-layout contract from the previous bundle-loader-progress requirements (the status region SHALL NOT jitter under high-frequency events).
- The default theme SHALL switch from Catppuccin to native auto/light/dark using Zinc neutrals (off-black `#0a0a0a`, never pure `#000`) and a single Teal accent (saturation ≤ 70%). Catppuccin SHALL be relocated to `src/lib/themes/catppuccin.css` as an opt-in theme pack accessible from settings; the existing Catppuccin theme selector SHALL continue to function once the pack is enabled.
- A new `src/lib/tokens.css` SHALL declare the design-token layer through Tailwind v4 `@theme`: Zinc neutrals, Teal accent, radii (`--radius-card` = 1.5rem, `--radius-pill`), tinted shadow tokens (`--shadow-elev-1`, `--shadow-elev-2`, `--shadow-elev-3`), and a 1px inner border token (`zinc-200/50` light, `zinc-800/60` dark). Inspector cards SHALL use `--radius-card`.
- Typography SHALL switch to Geist (UI) and Geist Mono (numbers, tabular-nums) loaded via the `geist` npm package wired through `app.css`. Inter is explicitly banned.
- Two shadcn-svelte primitives SHALL be generated: `sheet` (slide-in right) and `command` (Cmd-K palette). They are added by this change as primitive files only; their wiring lives in the follow-up changes. This change supersedes the active `bundle-loader-as-overlay` proposal — the Sheet primitive addition, the cold-start vs in-workspace loader split, and the "Maps…" slide-in behavior described there are absorbed into this shell change.
- The cold-start route (`/`) SHALL remain the bundle loader and SHALL keep its current 2-column projects/maps structure for v1; only its surfaces SHALL be restyled to consume the new tokens. Structural redesign of cold-start may be revisited in `library-sidebar`.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `ui-shell`: the workspace layout invariant is replaced by the 3-pane shell with named slots; the default theme requirement is rebased from Catppuccin to native Zinc + Teal tokens (Catppuccin becomes opt-in); the primitives list adds `sheet` and `command`; the typography invariant adds Geist + Inter ban; the status-bar stable-layout contract is preserved across the redesign.

## Impact

- **Frontend**: new `src/lib/tokens.css`; `src/app.css` updated to load Geist and consume tokens; new `src/components/WorkspaceShell.svelte`; `src/routes/project/+page.svelte` rewritten to mount `WorkspaceShell`; `src/components/Sidebar.svelte` left in place for change 2 to replace (this change does not delete it — it is hidden behind the empty `library-rail` slot until then); `src/lib/themes/catppuccin.css` created from the current Catppuccin layer; `src/lib/components/ui/sheet/` and `src/lib/components/ui/command/` generated via shadcn-svelte; `package.json` gains the `geist` dependency.
- **Backend**: none.
- **Spec evidence**: smoke screenshot of the new shell with empty rails (proves the layout grid renders correctly); smoke that switching the active theme between native auto/light/dark and Catppuccin still works; smoke that the cold-start `/` route renders with the new tokens; status bar does not jitter when bundle-progress events arrive at 5+ per second; the existing MapView "mounted once across route changes" invariant is preserved.
- **Coordination**: this change MUST land AFTER `consolidate-state-event-flow`. That change rewrites the event-listener topology inside `src/routes/+layout.svelte` (single owner per event, store-hoisted page-local state). Touching `+layout.svelte` and the workspace route in the same window would force a merge. Land the listener consolidation first; rebase this change on top.
- **Supersession**: this change supersedes the active `bundle-loader-as-overlay` proposal. After this change is implemented, the Sheet primitive and the "Maps…" slide-in behavior described in `bundle-loader-as-overlay` are obsolete as a standalone change. The `library-sidebar` change will wire the actual "Maps…" affordance against the Sheet primitive added here.
- **Risk**: medium. The shell rewrite touches the workspace's outermost layout primitive and the theme defaults. Mitigations: the floating panels and MapView wiring are isolated behind named slots so internal component code (MapView, TracksPanel, WaypointsPanel) is untouched by this change; the Catppuccin theme code is moved, not deleted, so the existing theme selector continues to work once the user enables the pack.

## Out of scope

- Library content (project picker, layer tree, tracks/waypoints list) — `library-sidebar` change.
- Inspector content (selection-driven detail panels) and Cmd-K command palette behavior — `inspector-pane` change.
- Elevation chart, route planning, and any new MapView features.
- Backend changes to the `.ozf2`/SQLite tile renderer.
- Resizable rails — fixed widths for v1.
- Removing or rewriting `Sidebar.svelte` itself — it stays in the tree, hidden behind the empty rail slot, until `library-sidebar` replaces it.
