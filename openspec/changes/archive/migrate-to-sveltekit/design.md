## Context

This change is the foundation move of the broader UI-kit refactor captured in `docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md` — specifically "Change 1 — `migrate-to-sveltekit`" and its underlying decisions **D2** (SvelteKit + `adapter-static` over pure-Vite) and **D3** (dedicated routes for the two top-level surfaces). The parent design doc is the source of truth for the rationale; this `design.md` only records the decisions specific to executing Change 1 in isolation, so the change is self-contained and reviewable on its own.

### Correction to the parent design doc

D3 of the parent design doc claims the current setup uses "conditional rendering inside `App.svelte`". That is inaccurate. The actual starting state — verified against `src/App.svelte`, `src/main.ts`, `src/lib/windows.ts`, `src/views/BundleLoaderView.svelte`, `src-tauri/capabilities/default.json`, and commit `79c99cd16` ("open bundle loader in a separate OS window", Mon 30 Mar 2026) — is a **dual-window** architecture:

- Main `WebviewWindow` (label `main`): `App.svelte` → project workspace (`Sidebar` + `MapView` + panels + `Console`). No conditional.
- Second `WebviewWindow` (label `bundles`): `BundleLoaderView`. Pre-created hidden at startup by `precreateBundleLoader()` in `src/lib/windows.ts`. Routed to by reading `?view=bundles` in `src/main.ts`. Owns its own close-request handler (`hide()` instead of `close()`).

The new model in this change is a **single window with two routes** (`/` and `/project`). The decisions below are the consequences of executing that collapse correctly.

## Key decisions

### Decision 1 — Single window, two routes, no `bundles` `WebviewWindow`

The `bundles`-labelled `WebviewWindow` and the associated `src/lib/windows.ts` module are deleted. The bundle loader surface becomes a regular SvelteKit page at `/`; the project workspace becomes a page at `/project`. The Sidebar's "Maps…" button calls `goto('/')` instead of `openBundleLoader()`. The Tauri capabilities file (`src-tauri/capabilities/default.json`) drops the `"bundles"` window label and the six window/webview-management permissions that only the second window required, leaving `dialog:*` and `shell:*` plus `core:default`. The `src/test/capabilities.test.ts` invariant test is updated to match.

### Decision 2 — Persistent `MapView` lives in `src/routes/+layout.svelte`

`MapView` initialises a MapLibre map, registers custom protocols (`sqlite://`, `ozi://`), loads style, markers, and tracks. Re-mounting it on every `/` → `/project` transition costs ~100–300 ms and discards transient view state (current zoom, pan). To make `/` ↔ `/project` switching feel instant (~10–20 ms), `MapView` is mounted **once** in the root layout (`src/routes/+layout.svelte`) and its visibility is toggled via a derived CSS class based on `$page.url.pathname`. `Console` (the global log dock) lives in the layout for the same reason — it is route-agnostic.

This is a **clarification** of D3 in the parent design (which reads "`BundleLoaderView` moves into `routes/+page.svelte`; the main working view into `routes/project/+page.svelte`"). The parent's "main working view" splits in this change into:

- `MapView` + `Console` → `src/routes/+layout.svelte` (mounted always).
- `Sidebar` + panels (`TracksPanel`, `TrackPointsPanel`, `WaypointsPanel`) → `src/routes/project/+page.svelte` (mounted only on `/project`).

The Svelte stores (`appState`, `currentProject`, `activeMap`, …) are module-level in `src/lib/stores.ts` and already survive any route change without ceremony, so this split has no state-management cost.

### Decision 3 — Client-side redirect, not `load`-function redirect

`prerender = true` is required so Tauri can load the built static output without a Node runtime. With prerender on, route-level `+page.ts` `load` functions execute at build time and have no access to the runtime Svelte store that tracks "is a bundle loaded?". The redirect from `/` → `/project` (when a bundle is already loaded) and from `/project` → `/` (when none is) therefore happens inside `onMount` of each `+page.svelte`, using `goto()` from `$app/navigation`. This costs one extra paint on cold start but keeps prerender semantics intact.

### Decision 4 — `ssr = false` and `prerender = true` set once, in `+layout.ts`

SSR is disabled at the layout level rather than per-page so every future route inherits it without ceremony. `prerender = true` is set in the same file. The combination matches the adapter-static + Tauri pattern documented in SvelteKit's own Tauri integration guide.

### Decision 5 — Legacy `src/components/` is gated from new tooling during this change

The 9 existing components keep their current Svelte 5 syntax and styles during Change 1. They will be migrated one-by-one in Change 3. Two gates are in place so the SvelteKit migration is not blocked on a tooling sweep:

- `eslint.config.js` ships with an `ignores` glob for `src/components/**` (and historically `src/views/**`, although that directory is deleted by this change). ESLint also ignores `build/`, `.svelte-kit/`, `node_modules/`, and the QA-evidence directories under `.sisyphus/`.
- `svelte-check` does not support per-file ignores together with `--tsconfig`, so legacy components that surface pre-existing strict-null / source-narrowing issues (currently only `src/components/MapView.svelte`) carry a localised `// @ts-nocheck` pragma at the top of their `<script lang="ts">` block, with a comment referencing this decision. Each pragma is removed as Change 3 migrates that component to its shadcn-svelte primitives form.

Both gates tighten progressively as Change 3 lands each component.

### Decision 6 — `prettier-plugin-tailwindcss` deferred

The Tailwind ordering plugin only adds value once Tailwind itself is present. It is intentionally omitted from this change's Prettier config and added together with Tailwind in Change 2. This keeps Change 1 hermetic — no half-installed Tailwind ecosystem leaking into the foundation move.

### Decision 7 — `frontendDist` swap is verified by `tauri build` inside this change

Switching `frontendDist` from `../dist` to `../build` is a small but high-blast-radius edit (CI runs `tauri build`). The Tasks section explicitly mandates a local `tauri build` smoke as the closing QA step, so any regression surfaces inside this change rather than poisoning Change 2.

### Decision 8 — Stay on npm

The project ships with `package-lock.json` and `npm run` invocations across `justfile` and `src-tauri/tauri.conf.json` (`beforeDevCommand: "npm run dev"`, `beforeBuildCommand: "npm run build"`). The earlier draft of `tasks.md` referenced `pnpm`, which was a copy-over from a template. Migrating the package manager is orthogonal to the SvelteKit move and is deferred to its own (optional) follow-up change. All command examples in this change use `npm` / `npm exec`.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| SvelteKit migration breaks `tauri build` | Medium | Run `tauri build` immediately after `adapter-static` install, before any other tooling churn; mandated as a QA task. |
| ESLint trips on legacy components mid-change | Medium | Ignore `src/components/**` in ESLint flat config; rules tighten incrementally during Change 3. |
| `tauri.conf.json` `frontendDist` swap breaks CI | Low | Caught immediately by the `tauri build` smoke step inside this change. |
| Removing the `bundles` `WebviewWindow` regresses the "open bundle loader as a side panel" UX | Low | `BundleLoaderView` is still reachable instantly from Sidebar via `goto('/')`; persistent `MapView` keeps the round-trip cheap (no MapLibre re-init). If feedback later wants a true side panel, it can be re-added as `routes/bundles/+page.svelte` opened in a fresh `WebviewWindow` — that is a non-breaking addition. |
| Persistent `MapView` in the layout breaks MapLibre when the container is hidden via `display: none` (canvas dimensions go to 0) | Low | MapLibre tolerates a hidden container; the canvas resumes correctly on `display: flex` because the existing `ResizeObserver` in `MapView` re-fits. The redirect-on-cold-start scenario only crosses the boundary once per session, so a one-time `map.resize()` call on the `/project` route's `onMount` is the documented fallback if a glitch shows up. |
| `BundleLoaderView` loses its `onCloseRequested` → `hide()` block and starts closing the main window when the user hits ⌘W on `/` | Low | The block is window-specific to the now-deleted `bundles` `WebviewWindow`. On `/`, ⌘W targets the `main` window, which is the same behaviour as `/project` already has — no regression. |
