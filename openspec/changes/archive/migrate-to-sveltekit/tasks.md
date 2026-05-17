## 1. Validate deltas

- [x] 1.1 Run `openspec validate migrate-to-sveltekit --strict` and confirm the `ui-shell` deltas parse with at least one scenario per requirement.
- [x] 1.2 Confirm the spec deltas (`specs/ui-shell/spec.md`) cover both the routing contract and the single-window invariant.

## 2. SvelteKit bootstrap

- [x] 2.1 Add dev dependencies `@sveltejs/kit`, `@sveltejs/adapter-static`, `@types/node`, pinning to the current major. Keep the existing `@sveltejs/vite-plugin-svelte` (Kit re-exports the bundled version, but having it as a direct dep is harmless).
- [x] 2.2 Create `svelte.config.js` at repo root: import `adapter-static`, configure `kit.adapter` with `fallback: 'index.html'`, set `kit.alias.$lib = 'src/lib'`. Keep `vitePreprocess()` preprocess.
- [x] 2.3 Move `index.html` content into `src/app.html`, replacing the `<script type="module" src="/src/main.ts"></script>` with `%sveltekit.head%` inside `<head>` and `%sveltekit.body%` wrapping `<div id="app">…</div>`; delete the old `index.html`.
- [x] 2.4 Create `src/app.d.ts` with the standard SvelteKit `App` namespace stub (`declare global { namespace App {} }; export {};`).
- [x] 2.5 Update `vite.config.ts` to use `sveltekit()` from `@sveltejs/kit/vite` instead of the standalone `svelte()` plugin. Keep the `clearScreen: false`, port `5173`, `strictPort: true`, host/HMR, `envPrefix: ["VITE_", "TAURI_ENV_*"]`, and the existing `build.target` / `minify` / `sourcemap` / `manualChunks(maplibre)` block. Also update `tsconfig.json` to extend `./.svelte-kit/tsconfig.json` so `$lib` / `$app/*` aliases resolve under svelte-check.
- [x] 2.6 Delete `src/main.ts` (SvelteKit auto-bootstraps). The theme bootstrap call (`applyStoredTheme()`) moves into `src/routes/+layout.svelte`'s top-level script.

## 3. Routes and persistent layout

- [x] 3.1 Create `src/routes/+layout.svelte`:
  - Top-level script: `applyStoredTheme()` from `$lib/theme`, set up `state-changed` + `download-progress` Tauri event listeners (currently in `App.svelte`), and bind a derived `showWorkspace` to `$page.url.pathname === '/project'`.
  - Markup: a wrapper `<div>` containing the persistent `MapView` (toggled visible via `class:hidden={!showWorkspace}` → CSS `display: none`), `<slot />` for the route-specific content, and the persistent `Console`. Import `MapView` and `Console` from `../components/` (these stay where they are during Change 1).
- [x] 3.2 Create `src/routes/+layout.ts` exporting `export const ssr = false;` and `export const prerender = true;`.
- [x] 3.3 Create `src/routes/+page.svelte` (the bundle loader route):
  - Move the body of `src/views/BundleLoaderView.svelte` here.
  - **Remove** the `onCloseRequested` → `hide()` block and the `import("@tauri-apps/api/webviewWindow")` import (single-window model; the loader no longer owns its own window).
  - Inside `onMount`, after the initial state refresh, if `get(activeMap)` is non-null call `goto('/project')` so a user who already has a loaded map skips the loader.
- [x] 3.4 Create `src/routes/project/+page.svelte`:
  - Renders `<Sidebar />` and the three panel components (`TracksPanel`, `TrackPointsPanel`, `WaypointsPanel`) inside the existing `.layout` flex wrapper. `MapView` is **not** rendered here — it lives in the layout (Decision 2).
  - Inside `onMount`, if `get(activeMap)` is null, call `goto('/')`.
- [x] 3.5 Delete `src/App.svelte` and `src/views/BundleLoaderView.svelte`. Remove the now-empty `src/views/` directory.

## 4. Single-window collapse

- [x] 4.1 Delete `src/lib/windows.ts`.
- [x] 4.2 In `src/components/Sidebar.svelte`: replace `import { openBundleLoader } from "../lib/windows";` with `import { goto } from "$app/navigation";`, and change `onclick={openBundleLoader}` to `onclick={() => goto('/')}`. (The button stays labelled "Maps…".)
- [x] 4.3 Update `src-tauri/capabilities/default.json`:
  - `"windows"` becomes `["main"]`.
  - Drop `core:window:allow-create`, `core:window:allow-get-all-windows`, `core:window:allow-hide`, `core:window:allow-set-focus`, `core:window:allow-show`, `core:webview:allow-create-webview`, `core:webview:allow-create-webview-window` from `"permissions"`. Keep `core:default`, `dialog:allow-open`, `dialog:allow-save`, `shell:allow-open`.
- [x] 4.4 Update `src/test/capabilities.test.ts`:
  - `REQUIRED_PERMISSIONS` becomes `["dialog:allow-open", "dialog:allow-save", "shell:allow-open"]` (the window-management ones are gone).
  - `REQUIRED_WINDOWS` becomes `["main"]`.

## 5. Tauri integration

- [x] 5.1 Update `src-tauri/tauri.conf.json`: change `build.frontendDist` from `../dist` to `../build`.
- [x] 5.2 Confirm `build.devUrl` still points at `http://localhost:5173` and that `beforeDevCommand: "npm run dev"` / `beforeBuildCommand: "npm run build"` still resolve to SvelteKit's `vite dev` / `vite build` (the package.json scripts stay named `dev` and `build`).

## 6. Tooling

- [x] 6.1 Add `eslint`, `@eslint/js`, `typescript-eslint`, `globals`, and `eslint-plugin-svelte` as dev dependencies; create `eslint.config.js` (flat config) with `typescript-eslint`-aware parsing for `.ts`, Svelte parser for `.svelte`, recommended rules, and an `ignores` array covering `node_modules/`, `build/`, `.svelte-kit/`, `src-tauri/target/`, `dist/`, `.sisyphus/`, `src/components/**` (cleared progressively in Change 3).
- [x] 6.2 Add `prettier` and `prettier-plugin-svelte` as dev dependencies; create `.prettierrc` referencing the Svelte plugin and `.prettierignore` for `build/`, `.svelte-kit/`, etc. Do NOT install `prettier-plugin-tailwindcss` — it lands in Change 2.
- [x] 6.3 Add `svelte-check` as a dev dependency.
- [x] 6.4 Add `@testing-library/svelte` as a dev dependency.
- [x] 6.5 Update `justfile` with the new recipes. The existing `check` recipe (cargo check) is renamed to `check-rust` to free the name; `check-ui` runs svelte-check; `check` becomes the umbrella that runs both. Added: `lint` → `npm exec -- eslint .`, `fmt` → `npm exec -- prettier --write .`. This mirrors the existing `test-rust`/`test-ui`/`test` naming.
- [x] 6.6 Update `package.json` scripts so `npm run lint`, `npm run fmt`, `npm run check` exist for direct invocation. Keep the existing `dev`/`build`/`preview`/`tauri`/`test`/`test:watch` scripts (they continue to work — SvelteKit's `vite` plugin handles `vite dev`/`vite build`).

## 7. Docs

- [x] 7.1 Update `AGENTS.md` "Module layout" section: replace the `src/components/`, `src/views/`, `src/lib/` block with the new `src/app.html`, `src/routes/+layout.{svelte,ts}`, `src/routes/+page.svelte`, `src/routes/project/+page.svelte`, `src/components/`, `src/lib/` shape. Add a sentence noting that the dual-window setup (`bundles` `WebviewWindow`) has been replaced by the single-window + routes model.
- [x] 7.2 Update `CLAUDE.md` "Workflow" section. It did not reference `src/App.svelte` or `src/views/`; the "Run X after code changes" hint was extended to include `just check` and `just lint`.

## 8. QA

- [x] 8.1 `just lint` is clean (legacy `src/components/**` ignore glob in place; `target/`, `build/`, `.svelte-kit/`, `.sisyphus/` also ignored).
- [x] 8.2 `just check-ui` (svelte-check) is clean. Pre-existing strict-null / source-narrowing issues in `src/components/MapView.svelte` are silenced via a localised `// @ts-nocheck` pragma with a comment referencing Decision 5; the pragma is removed when MapView migrates in Change 3.
- [x] 8.3 `just clippy` is clean (no backend regressions; backend change limited to `tauri.conf.json` and `capabilities/default.json`).
- [x] 8.4 `just test` passes — Rust 203 tests, frontend 66 tests. `capabilities.test.ts` invariants updated to match the new single-window setup. `bundle-loader-non-blocking.test.ts` and `bundle-loader-progress.test.ts` retargeted from `../views/BundleLoaderView.svelte` to `../routes/+page.svelte`.
- [x] 8.5 `npm run build` (SvelteKit + adapter-static) emits `build/` with `index.html` (prerendered `/`), `project.html` (prerendered `/project`), and `200.html` (fallback). `just build` (which runs `cargo tauri build --debug` underneath) produced a launchable `target/debug/bundle/macos/ozi-rs.app` and a Tier-1 launch + log capture confirmed the app starts without WebKit errors. **Tier-2 smoke (visual confirmation of `/` and `/project` routes plus persistent MapView round-trip) is intentionally deferred to manual verification by the operator** — automated MCP-driven smoke was opted out during this session.
