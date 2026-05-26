## 1. Add the shadcn `sheet` primitive

- [ ] 1.1 Run `npx shadcn-svelte@latest add sheet` from the repo root to generate `src/lib/components/ui/sheet/`
- [ ] 1.2 Verify the generated primitive imports `cn` from `$lib/utils` and uses semantic-token Tailwind classes (`bg-background`, `text-foreground`, `border-border`) for its surfaces
- [ ] 1.3 Update `THIRD_PARTY_LICENSES.md` only if `shadcn-svelte add sheet` brought a new transitive dependency that is not already credited (typically it reuses `bits-ui` already in the tree — verify before editing)

## 2. Extract `BundleLoader` from the `/` route

- [ ] 2.1 Create `src/components/BundleLoader.svelte` and move the markup, script, and styles currently inline at `src/routes/+page.svelte:1-792` into it; keep the same DOM structure (`.root` → `.loader` + `.status-bar`)
- [ ] 2.2 Replace the body of `src/routes/+page.svelte` with a thin wrapper that imports and renders `<BundleLoader />` inside the existing full-width layout; preserve the module-level `initialRedirectChecked` guard and the cold-start `goto('/project')` logic at the route level (these are routing concerns, not loader concerns)
- [ ] 2.3 Hoist the `handleOpenMap` post-success behavior out of `BundleLoader` into a prop callback (e.g. `onMapOpened?: () => void`); the route page wires it to `goto(resolve('/project'))`, the Sheet host (task 3) wires it to `bundleLoaderOpen.set(false)`
- [ ] 2.4 Verify cold-start still works: launch the app with no active map, confirm `/` renders the loader full-width, confirm selecting a project + opening a map navigates to `/project` and renders the workspace

## 3. Wire the Sheet in the workspace sidebar

- [ ] 3.1 In `src/components/Sidebar.svelte`: import `* as Sheet from "$lib/components/ui/sheet"`, import `bundleLoaderOpen` from `$lib/stores`, and import `BundleLoader` from `../components/BundleLoader.svelte` (relative)
- [ ] 3.2 Replace the `goto(resolve("/"))` onclick at `src/components/Sidebar.svelte:188-196` with `() => bundleLoaderOpen.set(true)`; keep the icon and label as today
- [ ] 3.3 Add a `<Sheet.Root bind:open={$bundleLoaderOpen}>` block somewhere in the Sidebar tree (or hoisted to the workspace `+page.svelte` if a separate placement reads cleaner — document the chosen site in the PR); the `Sheet.Content` SHALL set `side="right"` and `class="w-[480px] sm:max-w-[480px] p-0 flex flex-col"`
- [ ] 3.4 Render `<BundleLoader onMapOpened={() => bundleLoaderOpen.set(false)} />` inside `Sheet.Content`
- [ ] 3.5 Style `Sheet.Overlay` to be visually transparent and `pointer-events-none` so the underlying workspace map remains pannable / zoomable while the Sheet is open (Decision 6 in design.md); confirm bits-ui's click-outside-to-close still triggers from the Sheet's own boundary detection, not from the overlay's pointer events

## 4. Preserve cold-start behavior

- [ ] 4.1 Confirm the redirect chain in `src/routes/+page.svelte:65-79` and `src/routes/project/+page.svelte:12-24` is unchanged by the extraction; the routing logic continues to live in the route files, not in `BundleLoader`
- [ ] 4.2 Confirm that with no active map present, the workspace route immediately redirects to `/` and the user never sees an empty workspace with the Sheet over nothing
- [ ] 4.3 Confirm that opening a map from the `/` route (cold-start) navigates to `/project` and never opens the Sheet (the Sheet is workspace-only)

## 5. Update routing & state expectations

- [ ] 5.1 Remove the navigation expectation from the "Sidebar Maps…" affordance in any frontend tests / smoke checks that asserted `goto('/')` was called
- [ ] 5.2 Ensure no listener registrations are added by this change; the Sheet body uses the same stores the route page already feeds, and the existing `listen()` calls in `+page.svelte` (now `BundleLoader.svelte`) continue to be the source of progress events
- [ ] 5.3 Verify that when the Sheet is open inside the workspace, the listeners attached by `BundleLoader.svelte`'s `onMount` are not duplicated against the layout-level listeners — a static grep for each event name SHALL still match the count it matched before this change (no new occurrences added)

## 6. Verification

- [ ] 6.1 Manual: from cold start (no active map), confirm `/` renders the loader full-width and behaves identically to today; open a map and confirm redirection to `/project`
- [ ] 6.2 Manual: with an active map at `/project`, click "Maps…" — confirm the Sheet slides in from the right at ~480px wide, the URL stays `/project`, and the workspace map remains visible and interactive (pan / zoom on the visible map area while the Sheet is open)
- [ ] 6.3 Manual: from inside the Sheet, click another map row — confirm the new map activates, the Sheet animates closed, the URL stays `/project`, and the workspace re-renders with the new map
- [ ] 6.4 Manual: open the Sheet, press Esc — confirm Sheet closes and workspace remains; open the Sheet, click the X — confirm Sheet closes; open the Sheet, click on the workspace map outside the Sheet content — confirm Sheet closes
- [ ] 6.5 Manual: start a download from inside the Sheet, close the Sheet mid-download, re-open after 2-3 seconds — confirm progress bar / ready-files reflect the events that arrived while the Sheet was closed (state stayed alive in the stores)
- [ ] 6.6 Manual: verify with all four Catppuccin flavours that the Sheet's background, border, and text colours follow the active theme without any per-component override
- [ ] 6.7 `just ci` passes (clippy, check, lint, test)
- [ ] 6.8 `openspec validate bundle-loader-as-overlay --strict` passes
