## 1. Preconditions

- [x] 1.1 Confirm `consolidate-state-event-flow` has been merged to `main`; if not, hold this change until it lands (see design.md, Migration Plan step 1)
- [x] 1.2 Rebase this change branch onto current `main` after the consolidate change merges; resolve any conflicts in `src/routes/+layout.svelte` and `src/routes/project/+page.svelte` before starting implementation
- [x] 1.3 Coordinate with parent on whether to archive the active `bundle-loader-as-overlay` proposal as superseded, or to keep it as a reference document for `library-sidebar` (see design.md, Open Question Q4)

## 2. Design-token layer

- [x] 2.1 Create `src/lib/tokens.css` declaring a Tailwind v4 `@theme` block with: a Zinc neutral scale (off-black `#0a0a0a` for the dark surface base), a single Teal accent (saturation ≤ 70%), `--radius-card` (1.5rem), `--radius-pill` (full), `--shadow-elev-1` / `--shadow-elev-2` / `--shadow-elev-3` (tinted, not flat gray), and an inner-border token (`zinc-200/50` light, `zinc-800/60` dark)
- [x] 2.2 Update `src/app.css` to `@import './lib/tokens.css'` and to set `font-sans` and `font-mono` to Geist and Geist Mono respectively; remove any Inter import or font-family declaration if present
- [x] 2.3 Confirm that shadcn-svelte semantic-token utilities (`bg-background`, `text-foreground`, `border-border`, `bg-card`, `bg-popover`, etc.) resolve against the new token layer in both light and dark mode

## 3. Typography (Geist)

- [x] 3.1 Add the `geist` npm package to `package.json` (`npm install geist`)
- [x] 3.2 Wire Geist + Geist Mono through `src/app.css` per the `geist` package's documented usage (CSS variable handles or `next/font`-equivalent — the package ships both)
- [x] 3.3 Update `THIRD_PARTY_LICENSES.md` with an entry for `geist` (Vercel, MIT or SIL OFL as applicable per upstream)
- [x] 3.4 Static check: `grep -ri 'inter' src/` returns no font-family / font-import results (only word-occurrence false positives in identifiers are acceptable, see spec scenario for the exact criterion)

## 4. Catppuccin theme pack relocation

- [x] 4.1 Create `src/lib/themes/catppuccin.css` containing the current Catppuccin `--ctp-*` palette layer and the semantic-token mapping tables (`SEMANTIC_MAP_LIGHT` / `SEMANTIC_MAP_DARK`)
- [x] 4.2 Update `src/lib/theme.ts` (or the equivalent theme module) so the Catppuccin layer is loaded only when a Catppuccin flavour is the active theme; the native Zinc + Teal layer is loaded by default
- [x] 4.3 Update the theme selector UI so the top-level theme picker offers Native (Auto / Light / Dark) and a "Catppuccin pack" entry that expands to the four flavours + Catppuccin Auto when enabled
- [x] 4.4 Verify: selecting "Mocha" still applies the same palette and semantic tokens as before this change (Catppuccin behavior is preserved, just gated behind the pack toggle)
- [x] 4.5 Verify: launching with no prior theme preference uses the native default tracking OS light/dark, NOT Catppuccin

## 5. shadcn-svelte primitives (sheet + command)

- [x] 5.1 Run `npx shadcn-svelte@latest add sheet` from the repo root; commit the generated `src/lib/components/ui/sheet/` files
- [x] 5.2 Run `npx shadcn-svelte@latest add command` from the repo root; commit the generated `src/lib/components/ui/command/` files
- [x] 5.3 Verify each generated primitive imports `cn` from `$lib/utils` and uses semantic-token Tailwind classes (`bg-background`, `text-foreground`, `border-border`)
- [x] 5.4 If either primitive brings a new transitive dependency that is not yet credited (typically both reuse `bits-ui` and `cmdk`-equivalent libs already in tree — verify before editing), add the entry to `THIRD_PARTY_LICENSES.md`
- [x] 5.5 Confirm no application component imports either primitive yet (the wiring lands in `library-sidebar` and `inspector-pane`); the primitive files are tree-available but unmounted

## 6. WorkspaceShell component

- [x] 6.1 Create `src/components/WorkspaceShell.svelte` with a CSS grid (or flex — see design.md, Decision 4) of `[280px library-rail] [1fr canvas] [360px inspector-rail-when-present]`
- [x] 6.2 Expose three named slots: `library-rail`, `canvas`, `inspector-rail`; the `library-rail` and `inspector-rail` slots default to empty placeholder content (a subtle 1px inner-border box with a small "Library" / "Inspector" label, or simply empty — implementer's call, document the decision in the PR)
- [x] 6.3 Render the top context-bar above the canvas: four mode-chip placeholders labelled "View" / "Draw" / "Edit" / "Measure" (styled as pill chips against the Zinc surface, no active-state binding), plus a single Cmd-K trigger button on the right (styled as an inline pill with the `⌘K` glyph; no `onclick` binding)
- [x] 6.4 Render the bottom status bar below the canvas: a stable-height region (use a single CSS custom property `--status-bar-height` declared in `tokens.css` and consumed both by the bar's height and by the workspace grid's bottom inset, per spec stable-layout requirement)
- [x] 6.5 Conditional inspector-rail rendering: mount the right rail only when an `inspectorOpen` store (or equivalent — define a placeholder `writable<boolean>` in `stores.ts` for this change; the `inspector-pane` change replaces it with content-driven logic) is true; default to false in this change so the right rail is absent on first paint
- [x] 6.6 Confirm `MapView` mount-once invariant is preserved: `WorkspaceShell.svelte` does NOT mount `MapView` itself; `MapView` continues to be mounted from `src/routes/+layout.svelte` and SHALL become visible inside the canvas region while the workspace route is active (cross-check with the existing `+layout.svelte:55-57` visibility toggle and adjust the toggle's target selector if the grid structure requires it)

## 7. Workspace route rewrite

- [x] 7.1 Rewrite `src/routes/project/+page.svelte` to mount `<WorkspaceShell />` with empty rail slots; remove the current `Sidebar.svelte` mount and the 3 floating panel mounts (`TracksPanel`, `TrackPointsPanel`, `WaypointsPanel`)
- [x] 7.2 Leave `Sidebar.svelte`, `TracksPanel.svelte`, `TrackPointsPanel.svelte`, `WaypointsPanel.svelte` in the repository tree — they are reused or replaced by `library-sidebar` and `inspector-pane`; do NOT delete them in this change
- [x] 7.3 Confirm the existing redirect logic from `/project` to `/` when there is no active map still works (this lives in the route file, not in the shell component)

## 8. Cold-start route restyling

- [x] 8.1 Restyle the surfaces of `src/routes/+page.svelte` (the bundle-loader cold-start route) to consume the new tokens: replace any Catppuccin-specific class references with semantic-token utilities; confirm the projects column, maps column, filter input, status-bar region, and progress region all render against the new Zinc + Teal palette
- [x] 8.2 Do NOT change the structural 2-column layout of the cold-start route in this change — that structural redesign is deferred to `library-sidebar` if pursued at all
- [x] 8.3 Confirm the bundle-loader status-bar stable-layout contract (from `bundle-loader-non-blocking` and `stream-bundle-file-availability`) is preserved after restyling

## 9. Smoke verification

- [x] 9.1 Manual: launch the app with no active map; confirm the cold-start `/` route renders against the new Zinc + Teal tokens, the project list is interactive, the filter input is responsive, and the status-bar layout remains stable when bundle-progress events arrive
- [x] 9.2 Manual: open a map; confirm the workspace `/project` route renders the 3-pane shell with the 280px library rail empty, the canvas filled by `MapView`, the top context-bar with four mode-chip placeholders and a Cmd-K trigger button, the bottom status bar present, and the right inspector rail absent from the DOM
- [x] 9.3 Manual: click any mode-chip placeholder or the Cmd-K trigger button; confirm no state change occurs and no palette opens (inert scaffolding)
- [x] 9.4 Manual: round-trip between `/` and `/project`; confirm `MapView` is not re-initialized (mount-once invariant preserved)
- [x] 9.5 Manual: enable the Catppuccin pack from settings; select Mocha; confirm the same palette and semantic tokens apply as before this change; select native default again; confirm the Zinc + Teal palette returns
- [x] 9.6 Manual: in dark mode, sample the workspace background colour with a colour picker; confirm the value is `#0a0a0a` (or its OKLCH equivalent), not `#000000`
- [x] 9.7 Capture screenshots: cold-start route under new tokens (light + dark), workspace shell with empty rails (light + dark), Catppuccin Mocha (regression check). Attach to the PR
- [x] 9.8 Per the project's verification policy (`docs/agent-verification.md`), if any verification attempt fails twice, stop and hand back a diagnostic dump rather than retrying a third time

## 10. CI and validation

- [x] 10.1 `just ci` passes (clippy, check, lint, test) locally
- [x] 10.2 `openspec validate redesign-shell-layout --strict` passes
- [x] 10.3 PR description references the supersession of `bundle-loader-as-overlay` and the landing-order dependency on `consolidate-state-event-flow`
