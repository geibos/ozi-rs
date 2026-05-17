## Why

The current frontend is a non-standard pure-Vite + Svelte 5 setup with a **dual-window architecture**: the project workspace lives in the main `WebviewWindow` (mounted from `src/App.svelte`), and the bundle loader lives in a separate, pre-created `WebviewWindow` with label `"bundles"` (mounted from `src/views/BundleLoaderView.svelte` via the `?view=bundles` URL parameter in `src/main.ts`). `src/lib/windows.ts` owns the second window's lifecycle (`precreateBundleLoader` at startup, `openBundleLoader` from the Sidebar "Maps…" button, `onCloseRequested` → `hide()` inside the loader view).

Every Svelte+Tauri project in 2026 ships on SvelteKit with `@sveltejs/adapter-static`: file-based routing, `$lib` alias, preconfigured tooling. Migrating now standardises the layout, **collapses the dual-window setup into a single window with real routes** (which unlocks future surfaces such as settings, bundle manager, history), and brings in the industry-standard tooling baseline (ESLint flat config, Prettier with the Svelte plugin, `svelte-check`, `@testing-library/svelte`) before the shadcn-svelte work in the follow-up changes lands. This change is the foundation move for the broader UI-kit work described in `docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md` (Change 1; decisions D2 and D3).

> **Note on the parent design doc:** D3 of `docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md` describes the current state as "conditional rendering inside `App.svelte`". That description is inaccurate — `App.svelte` has never conditionally rendered the bundle loader; the loader has lived in its own `WebviewWindow` since commit `79c99cd` (March 2026). This change captures the **actual** starting state and the resulting architectural shift (dual window → single window + routes).

## What Changes

- Adopt SvelteKit with `@sveltejs/adapter-static`. Add `svelte.config.js`, `src/app.html`, `src/app.d.ts`.
- Introduce `src/routes/+layout.svelte` and `src/routes/+layout.ts` (`ssr = false`, `prerender = true`).
- **Collapse the dual-window architecture into a single window with two routes:**
  - `/` — `BundleLoaderView` (welcome / load surface) at `src/routes/+page.svelte`.
  - `/project` — `Sidebar` + panels at `src/routes/project/+page.svelte`.
  - Redirects between `/` and `/project` happen client-side via `onMount` + `goto`, because `prerender = true` precludes runtime store access inside route-level `load` functions.
- **Persistent `MapView` in the root layout:** `MapView` is mounted once inside `src/routes/+layout.svelte` so cross-route navigation does not destroy and re-create the MapLibre map. The layout toggles its visibility (`display: none` on `/`, visible on `/project`). This is a clarification of D3 in the parent design — the spec's "MapView moves into routes/project/+page.svelte" is replaced by "MapView lives in the root layout; Sidebar + panels move into routes/project/+page.svelte".
- **Remove the second `WebviewWindow`:** delete `src/lib/windows.ts`, drop the `?view=bundles` branch in `src/main.ts` (the file itself is also deleted), replace the `openBundleLoader` call in `src/components/Sidebar.svelte` with `goto('/')`, and strip the `onCloseRequested` → `hide()` block from `BundleLoaderView` (it no longer owns its own window).
- Update `src-tauri/capabilities/default.json`: drop the `"bundles"` window label and the window-management permissions (`core:window:allow-create`, `core:window:allow-get-all-windows`, `core:window:allow-hide`, `core:window:allow-set-focus`, `core:window:allow-show`, `core:webview:allow-create-webview-window`) that only the second window needed. Keep `dialog:*` and `shell:*`.
- Update `src/test/capabilities.test.ts` (`REQUIRED_PERMISSIONS` and `REQUIRED_WINDOWS`) to match.
- Update `src-tauri/tauri.conf.json`: `frontendDist` points to `../build` (adapter-static output) instead of `../dist`. `beforeDevCommand` / `beforeBuildCommand` already use `npm run …` and stay correct.
- Install industry-standard tooling: ESLint flat config + `eslint-plugin-svelte`, Prettier + `prettier-plugin-svelte`, `svelte-check`, `@testing-library/svelte`. (`prettier-plugin-tailwindcss` is intentionally NOT installed here — it lands with Change 2 when Tailwind itself arrives.)
- Add `justfile` recipes: `lint`, `fmt`, `check`.
- Update `AGENTS.md` and `CLAUDE.md` to reflect the new `src/routes/` + `src/lib/` structure and the single-window model.

**Package manager:** the project already uses **npm** (`package-lock.json`, `npm run` in justfile, `npm run …` in `tauri.conf.json`). The change keeps npm. The earlier draft of `tasks.md` referenced `pnpm` — that was a copy-over from a template; all `pnpm` references are replaced with their `npm` equivalents.

## Impact

- Affected capabilities:
  - `ui-shell` — MODIFIED: the bootstrap requirement gains explicit "SvelteKit + adapter-static" framing, the two-route surface contract (`/` and `/project`), the single-window invariant (no separate `"bundles"` `WebviewWindow`), and the persistent-`MapView`-in-layout invariant.
- Affected code (implementation, this change):
  - `src/App.svelte` → deleted; project workspace composition moves into `src/routes/project/+page.svelte`; persistent `MapView` (and `Console`) move into `src/routes/+layout.svelte`.
  - `src/views/BundleLoaderView.svelte` → moved to `src/routes/+page.svelte`; window-lifecycle block (`onCloseRequested` → `hide()`) removed; new `onMount` redirects to `/project` when a map is already active.
  - `src/views/` directory → deleted (empty after the move).
  - `index.html` → moved to `src/app.html` with `%sveltekit.head%` / `%sveltekit.body%` placeholders.
  - `src/main.ts` → deleted (SvelteKit auto-bootstraps; theme bootstrap and bundle-loader pre-creation move into `src/routes/+layout.svelte`'s `onMount`, sans pre-creation).
  - `src/lib/windows.ts` → deleted.
  - `src/components/Sidebar.svelte` → `import { openBundleLoader } from "../lib/windows"` replaced with `import { goto } from "$app/navigation"`; `onclick={openBundleLoader}` replaced with `onclick={() => goto('/')}`. (Sidebar itself stays in `src/components/` per Decision 3.)
  - `src/test/capabilities.test.ts` → `REQUIRED_PERMISSIONS` trimmed to only what the single-window setup still needs (`dialog:*`, `shell:*`); `REQUIRED_WINDOWS` becomes `["main"]`.
  - `src-tauri/capabilities/default.json` — `windows` becomes `["main"]`; window/webview-management permissions removed.
  - `src-tauri/tauri.conf.json` — `frontendDist` switched from `../dist` to `../build`.
  - `vite.config.ts` → updated to use `@sveltejs/kit/vite`'s `sveltekit()` plugin.
  - `svelte.config.js` → rewritten to configure `adapter-static` (fallback: `'index.html'`) and `kit.alias.$lib = 'src/lib'`.
  - `src/app.html`, `src/app.d.ts` — new.
  - `src/routes/+layout.svelte`, `src/routes/+layout.ts`, `src/routes/+page.svelte`, `src/routes/project/+page.svelte` — new.
  - `package.json` — new dev dependencies (`@sveltejs/kit`, `@sveltejs/adapter-static`, `eslint`, `eslint-plugin-svelte`, `prettier`, `prettier-plugin-svelte`, `svelte-check`, `@testing-library/svelte`); scripts realigned to use `vite` via SvelteKit and to add `lint`/`fmt`/`check`.
  - `justfile` — `lint`, `fmt`, `check` recipes added.
  - `AGENTS.md`, `CLAUDE.md` — directory-layout sections updated; dual-window references replaced with single-window + routes.
- Out of scope (covered by sibling changes): Tailwind, shadcn-svelte primitives, semantic token mapping (Change 2); per-component migration to primitives (Change 3); a follow-up change to migrate the package manager (npm → pnpm/bun) if desired.
