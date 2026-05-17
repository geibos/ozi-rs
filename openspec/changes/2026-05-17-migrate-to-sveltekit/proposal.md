## Why

The current frontend is a non-standard pure-Vite + Svelte 5 setup that conditionally renders two top-level surfaces (bundle loader vs. project workspace) inside `App.svelte`. Every Svelte+Tauri project in 2026 ships on SvelteKit with `@sveltejs/adapter-static`: file-based routing, `$lib` alias, preconfigured tooling. Migrating now standardises the layout, unlocks real routes for future surfaces (settings, bundle manager, history), and brings in the industry-standard tooling baseline (ESLint flat config, Prettier with the Svelte plugin, `svelte-check`, `@testing-library/svelte`) before the shadcn-svelte work in the follow-up changes lands. This change is the foundation move for the broader UI-kit work described in `docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md` (Change 1; decisions D2 and D3).

## What Changes

- Adopt SvelteKit with `@sveltejs/adapter-static`. Add `svelte.config.js`, `src/app.html`, `src/app.d.ts`.
- Introduce `src/routes/+layout.svelte` and `src/routes/+layout.ts` (`ssr = false`, `prerender = true`).
- Split the two top-level surfaces into dedicated routes:
  - `/` — `BundleLoaderView` (welcome / load surface) at `src/routes/+page.svelte`.
  - `/project` — `MapView + Sidebar + panels` at `src/routes/project/+page.svelte`.
- Redirects between `/` and `/project` happen client-side via `onMount` + `goto`, because `prerender = true` precludes runtime store access inside route-level `load` functions.
- Update `src-tauri/tauri.conf.json`: `frontendDist` points to `build` (adapter-static output) instead of `dist`.
- Install industry-standard tooling: ESLint flat config + `eslint-plugin-svelte`, Prettier + `prettier-plugin-svelte`, `svelte-check`, `@testing-library/svelte`. (`prettier-plugin-tailwindcss` is intentionally NOT installed here — it lands with Change 2 when Tailwind itself arrives.)
- Add `justfile` recipes: `lint`, `fmt`, `check`.
- Update `AGENTS.md` and `CLAUDE.md` to reflect the new `src/routes/` + `src/lib/` structure.

## Impact

- Affected capabilities:
  - `ui-shell` — MODIFIED: the bootstrap requirement gains explicit "SvelteKit + adapter-static" framing and the two-route surface contract (`/` and `/project`).
- Affected code (implementation, follow-up change):
  - `src/App.svelte` → split into `src/routes/+layout.svelte`, `src/routes/+page.svelte`, `src/routes/project/+page.svelte`.
  - `index.html` → `src/app.html`.
  - `src/main.ts` → replaced by SvelteKit's auto-bootstrap; deleted.
  - `vite.config.ts` → updated to use `@sveltejs/kit/vite`'s `sveltekit()` plugin.
  - `svelte.config.js` → new; configures `adapter-static` and `kit.alias`.
  - `src-tauri/tauri.conf.json` — `frontendDist` switched from `dist` to `build`.
  - `package.json` — new dev dependencies (`@sveltejs/kit`, `@sveltejs/adapter-static`, `eslint`, `eslint-plugin-svelte`, `prettier`, `prettier-plugin-svelte`, `svelte-check`, `@testing-library/svelte`); scripts realigned.
  - `justfile` — `lint`, `fmt`, `check` recipes added.
  - `AGENTS.md`, `CLAUDE.md` — directory-layout sections updated.
- Out of scope (covered by sibling changes): Tailwind, shadcn-svelte primitives, semantic token mapping (Change 2); per-component migration to primitives (Change 3).
