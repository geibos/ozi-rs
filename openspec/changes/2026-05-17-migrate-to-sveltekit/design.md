## Context

This change is the foundation move of the broader UI-kit refactor captured in `docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md` — specifically "Change 1 — `migrate-to-sveltekit`" and its underlying decisions **D2** (SvelteKit + `adapter-static` over pure-Vite) and **D3** (dedicated routes for the two top-level surfaces). The parent design doc is the source of truth for the rationale; this `design.md` only records the decisions specific to executing Change 1 in isolation, so the change is self-contained and reviewable on its own.

## Key decisions

### Decision 1 — Client-side redirect, not `load`-function redirect

`prerender = true` is required so Tauri can load the built static output without a Node runtime. With prerender on, route-level `+page.ts` `load` functions execute at build time and have no access to the runtime Svelte store that tracks "is a bundle loaded?". The redirect from `/` → `/project` (when a bundle is already loaded) and from `/project` → `/` (when none is) therefore happens inside `onMount` of each `+page.svelte`, using `goto()` from `$app/navigation`. This costs one extra paint on cold start but keeps prerender semantics intact.

### Decision 2 — `ssr = false` set once, in `+layout.ts`

SSR is disabled at the layout level rather than per-page so every future route inherits it without ceremony. `prerender = true` is set in the same file. The combination matches the adapter-static + Tauri pattern documented in SvelteKit's own Tauri integration guide.

### Decision 3 — Legacy `src/components/` ignored by ESLint during this change

The 9 existing components keep their current Svelte 5 syntax and styles during Change 1. They will be migrated one-by-one in Change 3. To avoid blocking the SvelteKit migration on a tooling sweep, `eslint.config.js` ships with an `ignores` glob for `src/components/**` and `src/views/**` that gets tightened progressively as Change 3 lands each component.

### Decision 4 — `prettier-plugin-tailwindcss` deferred

The Tailwind ordering plugin only adds value once Tailwind itself is present. It is intentionally omitted from this change's Prettier config and added together with Tailwind in Change 2. This keeps Change 1 hermetic — no half-installed Tailwind ecosystem leaking into the foundation move.

### Decision 5 — `frontendDist` swap is verified by `tauri build` inside this change

Switching `frontendDist` from `dist` to `build` is a small but high-blast-radius edit (CI runs `tauri build`). The Tasks section explicitly mandates a local `tauri build` smoke as the closing QA step, so any regression surfaces inside this change rather than poisoning Change 2.

## Risks

Extracted from the parent design's risks table — only rows that apply to Change 1:

| Risk | Likelihood | Mitigation |
|---|---|---|
| SvelteKit migration breaks `tauri build` | Medium | Run `tauri build` immediately after `adapter-static` install, before any other tooling churn; mandated in tasks.md §6. |
| ESLint trips on legacy components mid-change | Medium | Ignore `src/components/**` and `src/views/**` in ESLint flat config; rules tighten incrementally during Change 3. |
| `tauri.conf.json` `frontendDist` swap breaks CI | Low | Caught immediately by the `tauri build` smoke step inside this change (tasks.md §6.3). |
