## 1. Validate deltas

- [ ] 1.1 Run `openspec validate 2026-05-17-migrate-to-sveltekit --strict` and confirm the `ui-shell` MODIFIED requirement parses with at least one scenario.
- [ ] 1.2 Confirm the full-text copy of the modified requirement matches the current `openspec/specs/ui-shell/spec.md` body before the additions.

## 2. SvelteKit bootstrap

- [ ] 2.1 Add dev dependencies `@sveltejs/kit`, `@sveltejs/adapter-static`, `@sveltejs/vite-plugin-svelte` (or rely on Kit's bundled version), pinning to the current major.
- [ ] 2.2 Create `svelte.config.js` at repo root: import `adapter-static`, configure `kit.adapter` with `fallback: 'index.html'`, set `kit.alias.$lib = 'src/lib'`.
- [ ] 2.3 Move `index.html` content into `src/app.html`, replacing the script tag with `%sveltekit.head%` / `%sveltekit.body%` placeholders; delete the old `index.html`.
- [ ] 2.4 Create `src/app.d.ts` with the standard SvelteKit `App` namespace stub.
- [ ] 2.5 Update `vite.config.ts` to use `sveltekit()` from `@sveltejs/kit/vite` instead of the standalone `svelte()` plugin.
- [ ] 2.6 Delete `src/main.ts` (SvelteKit auto-bootstraps).

## 3. Routes

- [ ] 3.1 Create `src/routes/+layout.svelte` containing the global chrome (existing Console host, theme bootstrap entry point) wrapping `<slot />`.
- [ ] 3.2 Create `src/routes/+layout.ts` exporting `export const ssr = false;` and `export const prerender = true;`.
- [ ] 3.3 Move `BundleLoaderView` into `src/routes/+page.svelte`. Inside `onMount`, if a bundle is already loaded in the store, call `goto('/project')`.
- [ ] 3.4 Move the `MapView + Sidebar + panels` composition into `src/routes/project/+page.svelte`. Inside `onMount`, if no bundle is loaded, call `goto('/')`.
- [ ] 3.5 Delete the now-empty `src/App.svelte` and `src/views/` directory once both routes render correctly.

## 4. Tauri integration

- [ ] 4.1 Update `src-tauri/tauri.conf.json`: change `build.frontendDist` from `../dist` to `../build`.
- [ ] 4.2 Confirm `build.devUrl` still points at the Vite dev server (unchanged port) and that the `before-dev-command` / `before-build-command` still resolve to `vite dev` / `vite build` via SvelteKit.

## 5. Tooling

- [ ] 5.1 Add `eslint` and `eslint-plugin-svelte` as dev dependencies; create `eslint.config.js` (flat config) with Svelte parser, recommended rules, and an `ignores` glob covering `src/components/**`, `src/views/**` (cleared progressively in Change 3).
- [ ] 5.2 Add `prettier` and `prettier-plugin-svelte` as dev dependencies; create `.prettierrc` referencing the Svelte plugin. Do NOT install `prettier-plugin-tailwindcss` — it lands in Change 2.
- [ ] 5.3 Add `svelte-check` as a dev dependency.
- [ ] 5.4 Add `@testing-library/svelte` as a dev dependency.
- [ ] 5.5 Update `justfile` with three recipes:
  - `lint` → `pnpm exec eslint .`
  - `fmt` → `pnpm exec prettier --write .`
  - `check` → `pnpm exec svelte-check --tsconfig ./tsconfig.json`
- [ ] 5.6 Update `package.json` scripts so `pnpm lint`, `pnpm fmt`, `pnpm check` mirror the just recipes for direct invocation.

## 6. Docs

- [ ] 6.1 Update `AGENTS.md`: replace the "frontend directory layout" section with the new `src/app.html` / `src/routes/` / `src/lib/` structure and note the `/` and `/project` routes.
- [ ] 6.2 Update `CLAUDE.md` "Workflow" section if it references `src/App.svelte` or `src/views/`; ensure ProjectCommand routing notes still hold.

## 7. QA

- [ ] 7.1 `just lint` is clean (with the legacy ignore glob in place).
- [ ] 7.2 `just check` (svelte-check) is clean across the new routes and `src/lib/`.
- [ ] 7.3 `just clippy` is clean (no backend regressions).
- [ ] 7.4 `just test` passes (Rust + frontend).
- [ ] 7.5 `pnpm tauri build` produces a working desktop binary with `frontendDist: build` — smoke-launch and confirm both `/` (bundle loader) and `/project` (workspace) render after a bundle load, per `docs/agent-verification.md`.
- [ ] 7.6 Cold-start redirect smoke: launch with no saved bundle → land on `/`; load a bundle → URL becomes `/project`; relaunch with the saved bundle → land on `/project` directly.
