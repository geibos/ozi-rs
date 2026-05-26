## Context

The current UX has two top-level surfaces sitting at distinct routes: the bundle loader at `/` (`src/routes/+page.svelte`) and the project workspace at `/project` (`src/routes/project/+page.svelte`). The root layout (`src/routes/+layout.svelte:55-57`) toggles `MapView` visibility based on the active route. Clicking "Maps…" in the workspace sidebar (`src/components/Sidebar.svelte:188-196`) calls `goto(resolve("/"))`, which causes the layout to hide `MapView` and render the loader page — the workspace effectively disappears even though the map remains mounted.

After reviewing UX mockups, the user explicitly rejected the modal-dialog and the native-floating-window variants in favour of a slide-in side panel. The cited reason is that the workspace must remain visible alongside the loader so the user can keep the map context while swapping which map is active.

The shadcn-svelte `Sheet` primitive (a wrapper around `bits-ui`'s dialog with `side` orientation) is the established library choice for slide-in panels and matches the existing primitive aesthetic. It is currently absent from `src/lib/components/ui/` and must be added via `npx shadcn-svelte@latest add sheet`.

## Goals / Non-Goals

**Goals:**
- Replace the "Maps…" navigation with a slide-in Sheet overlay so the workspace map stays visible.
- Extract the bundle loader markup into a reusable `BundleLoader` component decoupled from routing.
- Preserve the cold-start UX: with no active map, the loader is still the first thing the user sees.
- Keep MapView mounted once across the Sheet toggle (it's already mounted in the layout — must not break this).
- Use only existing IPC; no backend changes.

**Non-Goals:**
- True dual-surface side-by-side layouts (split view) — out of scope, the Sheet is an overlay.
- Multiple Sheets open at once — only one Sheet, opened by one button.
- Replacing the `/` route entirely — kept for cold-start and as a deep-link target.
- Adding map preview thumbnails — explicitly deferred (Decision 4).
- Resizable Sheet width — fixed 480px; a follow-up may add user-resize if asked for.

## Decisions

### Decision 1: Sheet open state is a UI-only Svelte store, not a URL search param

The Sheet's open/closed state SHALL be a `writable<boolean>` Svelte store. `bundleLoaderOpen` already exists at `src/lib/stores.ts:134` and is unused — reuse it.

**Rationale**: the bundle loader as a Sheet is a transient UI affordance, not a navigable surface. Encoding it in the URL (e.g. `/project?bundles=open`) adds complexity (router subscription, history-stack pollution on every toggle, the back button confusingly closing the Sheet) without buying anything — the user does not deep-link to "workspace with Sheet open", they deep-link to a map. Other transient panels in the codebase (`tracksPanelOpen`, `waypointsPanelOpen`, `consoleOpen`) follow the same pattern; consistency wins.

**Alternatives considered**:
- _URL search param `?bundles=open`_: rejected — see above. The "Maps…" button is a workspace-internal action; URL state should reflect navigation, not panel toggles.
- _Local state inside Sidebar.svelte_: rejected because the Sheet's body (`BundleLoader`) needs to close itself after a successful map open, which requires the open-state to be hoisted above both `Sidebar` and `BundleLoader`. The store is the cleanest hoist.

### Decision 2: `BundleLoader` lives in `src/components/`, not `src/lib/`

The extracted component SHALL be `src/components/BundleLoader.svelte`. It SHALL contain all the markup, script, and styles that today sit inline in `src/routes/+page.svelte:1-792`. The route page becomes a thin wrapper that mounts `<BundleLoader />` inside the existing full-width layout.

**Rationale**: `src/components/` is the established home for feature components (panels, pickers, MapView, Sidebar). `src/lib/` is reserved for runtime utilities (`api.ts`, `stores.ts`, `theme.ts`, `maplibre/`, `forms/`) per `docs/frontend-architecture.md`. The bundle loader is a feature component, not a runtime helper.

### Decision 3: `/` route is preserved for cold-start; the Sheet variant is for "from-workspace" usage

Cold-start behavior is unchanged: when `appState` reports no `activeMap`, the application lands on `/` and renders the full-width bundle loader. The cross-route redirect logic in `src/routes/+page.svelte:65-79` and `src/routes/project/+page.svelte:12-24` remains correct.

What changes: the workspace sidebar no longer calls `goto(resolve("/"))`. Instead, it opens the Sheet. With an active map present, the user stays on `/project` permanently for the lifetime of that session unless the project is closed.

**Why keep `/` at all?**
1. Cold-start: there is no workspace yet, so a Sheet on top of nothing is pointless; a route-level page is the right surface.
2. Determinism: SvelteKit's `prerender = true` requires statically-resolvable routes; ripping the route out would require synthesising the loader inside `+layout.svelte` conditionally, which is fragile.
3. Recovery: if a future code path clears the active map mid-session, the existing redirect to `/` still works.

**Spec consequence**: the existing requirement "Top-level surfaces live at distinct routes `/` and `/project`" is MODIFIED, not REMOVED. The route mapping stays; the workspace gains a Sheet variant on top of it. The "Sidebar Maps… button navigates within the main window" scenario inside the "single Tauri WebviewWindow" requirement is also MODIFIED — the assertion is now that the button opens a Sheet without creating a new window.

### Decision 4: Map preview thumbnails are OUT of scope

The user's suggestion "с превью карты может или еще как-то" is acknowledged but deferred to a future change. Reasons:

1. **New IPC required**: previewing a SQLite tile or an OZF2 raster needs a backend command (e.g. `get_map_preview(map_path) -> png_bytes`) that decodes a representative tile/region and renders it as PNG. This is not a trivial wrapper around existing code; OZF2 specifically requires the projection step in `infrastructure/`. That is a backend-shaped task, not a frontend Sheet task.
2. **Caching question**: previews must be cached (decoding on every Sheet open would be slow and would re-trigger every time the user opens the panel); designing the cache key, invalidation, and on-disk location belongs to its own proposal.
3. **Visual design**: thumbnail size, fallback for failed previews, lazy loading, placeholder during cache miss — these are design choices the user has not weighed in on yet.

The current Sheet change has a clear, bounded scope (extract + wire + cold-start preservation). Bundling thumbnails would double the surface area for a feature the user described as optional ("может").

**Recommended follow-up**: a separate proposal `map-preview-thumbnails` that adds `get_map_preview` and the cache infrastructure, then surfaces the thumbnails in whichever variant of the bundle loader exists at that time (route or Sheet — both consume the same `BundleLoader` component).

### Decision 5: Sheet width is fixed at 480px

The Sheet SHALL render with `class="w-[480px] sm:max-w-[480px]"` (or equivalent Tailwind class on the shadcn `SheetContent`). This is ~33-40% of typical workspace viewport widths used during SAR work (1280-1440px laptop screens) and matches the proportions the user approved during the mockup review.

**Rationale**: the existing bundle loader has two columns (projects + maps). 480px is the minimum at which both columns remain readable without truncation. Smaller breaks the layout; larger covers too much of the workspace map.

**Alternatives considered**:
- _Auto-width based on content_: rejected — content width is unstable as project names vary, would cause the Sheet to jump open at different widths between sessions.
- _User-resizable_: rejected for v1 — not requested, adds a drag handle + persistence concern.

### Decision 6: Sheet does not block the workspace map

The shadcn `Sheet` ships with an overlay (a translucent backdrop covering the rest of the viewport). For this change the overlay SHALL be styled to be non-interactive — i.e. the workspace map below the Sheet SHALL still be pannable / zoomable while the Sheet is open.

**Implementation**: the default shadcn `SheetOverlay` is `pointer-events-auto`. Override with `pointer-events-none` and remove its background tint (or set opacity to 0). The Sheet content itself remains interactive (it sits above the overlay with its own `pointer-events-auto`).

**Rationale**: the user's whole reason for picking a Sheet over a modal was that the workspace stays usable alongside it. A blocking overlay would re-create the "all-or-nothing" problem the modal variant has. The Sheet content still captures click-outside on the X button and the Esc keystroke via bits-ui's focus management — close behavior is unaffected.

**Alternatives considered**:
- _Default blocking overlay_: rejected — defeats the purpose of choosing a Sheet over a modal.
- _No overlay at all_: rejected — bits-ui uses the overlay for focus-trap edge detection; removing it entirely breaks the Esc/click-outside contract. Keeping a transparent, pass-through overlay preserves the contract while letting the map stay interactive.

## Risks / Trade-offs

- **Risk**: the bundle-loader page (`+page.svelte`) and the Sheet body (`BundleLoader.svelte`) both subscribe to the same Tauri events (`download-progress`, `bundle-progress`, `bundle-file-ready`, `projects-chunk`). When the loader is opened in the Sheet on top of `/project`, only the Sheet's instance is mounted (the `/` page is not in the route stack), so double-subscription is not introduced by this change. **Mitigation**: the listener-consolidation work in change `consolidate-state-event-flow` will eventually move all `listen()` registrations to the layout, eliminating any latent ambiguity. This change MUST NOT introduce new listeners or duplicate the existing ones.
- **Risk**: closing the Sheet while a download is in flight — the download MUST continue, and the partial state MUST persist when the user re-opens the Sheet. **Mitigation**: the stores driving the loader (`activeDownloadId`, `downloadProgress`, `readyBundleFiles`, `bundleProgress`) live in `src/lib/stores.ts` and survive component unmounts. The Sheet is a presentation layer over those stores; closing it has no effect on the underlying download.
- **Risk**: opening the Sheet does not give it initial focus correctly, breaking keyboard navigation. **Mitigation**: shadcn's `Sheet` handles focus-trap via bits-ui by default; the manual verification step in tasks covers Tab/Esc/Enter behavior.
- **Trade-off**: maintaining both the `/` route and the Sheet host duplicates the "where does the loader render" decision in two places. Acceptable: the route stays simple ("render BundleLoader full-width"), the Sheet stays simple ("render BundleLoader inside SheetContent"), and the `BundleLoader` component itself is identical in both. The duplication is in mount sites, not in logic.
- **Trade-off**: a fixed 480px width may feel narrow on ultrawide monitors and wide on small laptops. Acceptable for v1; a resize affordance can land later if requested.

## Migration Plan

Implementation order, to keep the working tree shippable at each step:

1. Add the shadcn `sheet` primitive (`npx shadcn-svelte@latest add sheet`). No call sites yet.
2. Create `src/components/BundleLoader.svelte` containing the markup currently inline in `src/routes/+page.svelte`. Make the route page render the new component. Verify cold-start still works exactly as before.
3. Wire the Sheet in `src/components/Sidebar.svelte`: replace the `goto(resolve("/"))` onclick with `bundleLoaderOpen.set(true)`. Add a `<Sheet.Root bind:open={...}>` block that renders `<BundleLoader />` as its body.
4. Add the "close Sheet on successful map open" behavior — `handleOpenMap` inside `BundleLoader` (or a prop passed in from Sidebar) calls `bundleLoaderOpen.set(false)` after `openSelectedMap` resolves.
5. Style the Sheet overlay to be pass-through (Decision 6).
6. Manual verification of all listed scenarios (cold-start path, Sheet open/close, map switch via Sheet, download in flight while Sheet closes, etc.).

No user data, session file, or persisted state is affected. No backend changes. No new IPC.
