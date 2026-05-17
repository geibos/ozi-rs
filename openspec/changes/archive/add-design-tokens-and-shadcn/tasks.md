## 1. Validate deltas

- [x] 1.1 Confirm Change 1 (`migrate-to-sveltekit`) is merged: SvelteKit, `$lib` alias, `routes/`, ESLint flat config, Prettier config, `svelte-check`, and `@testing-library/svelte` are all in place.
- [x] 1.2 Run `openspec validate add-design-tokens-and-shadcn --strict` and confirm the `ui-shell` MODIFIED + ADDED requirements parse with at least one scenario each.
- [x] 1.3 Confirm the MODIFIED requirement's body matches the full final text of the current entry in `openspec/specs/ui-shell/spec.md` plus the additions described in this change.

## 2. Tailwind v4 install + configuration

- [x] 2.1 Install `tailwindcss@^3` (downgraded from v4 per implementation decision: shadcn-svelte stable expects v3), `autoprefixer`, `postcss`, `tailwindcss-animate`, `@tailwindcss/typography` as dev dependencies.
- [x] 2.2 Add `postcss.config.js` enabling `tailwindcss` and `autoprefixer`.
- [x] 2.3 Create `tailwind.config.ts` with `darkMode: 'class'`, `content` globs covering `src/**/*.{svelte,ts,js}`, the `tailwindcss-animate` + `@tailwindcss/typography` plugins, and a token-aware colour scale (`background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `border`, `input`, `ring`) all using the `hsl(var(--<token>) / <alpha-value>)` pattern.
- [x] 2.4 Add `@tailwind base; @tailwind components; @tailwind utilities;` to `src/app.css` (created in Change 1).
- [x] 2.5 Add `prettier-plugin-tailwindcss` to the Prettier config from Change 1 (`.prettierrc` or `prettier.config.js`).
- [x] 2.6 Run `npx vite build` and record the resulting bundle size: layout CSS ~12 kB, page CSS ~12 kB (Tailwind purge active — only utilities actually used in templates are emitted).

## 3. Theme system extension

- [x] 3.1 In `src/lib/theme.ts`, add a `hsl` rendering helper that takes `@catppuccin/palette` colour `.hsl` and returns the triplet string `` `${h} ${s}% ${l}%` ``.
- [x] 3.2 Add `SEMANTIC_MAP_LIGHT` (Latte) and `SEMANTIC_MAP_DARK` (Frappé / Macchiato / Mocha) tables mapping each semantic token to a `@catppuccin/palette` colour name per the token mapping table in `design.md`.
- [x] 3.3 Implement `applySemanticTokens(name: Exclude<ThemeName, 'auto'>)` that (a) writes each semantic token as `` `${h} ${s}% ${l}%` `` to `document.documentElement.style`, (b) toggles `html.classList.toggle('dark', name !== 'latte')`.
- [x] 3.4 Modify `applyTheme(name)` to call `applySemanticTokens(resolveTheme(name))` after writing the existing `--ctp-*` palette variables (no regression to existing behaviour).
- [x] 3.5 `installAutoThemeListener()` exported from `theme.ts` and wired in `src/routes/+layout.svelte` `onMount`: subscribes to `prefers-color-scheme` and re-runs `applyTheme('auto')` only while the stored choice is `auto`.
- [x] 3.6 Keep `--ctp-*` palette writes unchanged; the two layers coexist on `<html>`.

## 4. `cn()` helper

- [x] 4.1 Install `clsx` and `tailwind-merge`.
- [x] 4.2 Create `src/lib/utils.ts` exporting `export function cn(...inputs: ClassValue[]) { return twMerge(clsx(inputs)); }`.

## 5. shadcn-svelte initialisation

- [x] 5.1 During implementation we discovered the legacy `tw3.shadcn-svelte.com` registry is offline; `shadcn-svelte@latest init` requires Tailwind v4. We migrated to Tailwind v4 (CSS-first `@theme inline` in `src/app.css`, `@tailwindcss/vite` plugin) and authored `components.json` directly (style `nova`, baseColor `zinc`, alias group identical to the spec). The `init` flow itself is non-interactive only via `--preset`, so we replicated its output by hand.
- [x] 5.2 `components.json` records `tailwind.css = src/app.css`, `aliases.ui = $lib/components/ui`, `aliases.utils = $lib/utils`, `aliases.lib = $lib`. No `tailwind.config` field — Tailwind v4 doesn't use one.
- [x] 5.3 `src/app.css` declares Latte HSL triplets in `@layer base :root` as fallbacks; `applySemanticTokens` overwrites them on every theme change.

## 6. Install primitives

- [x] 6.1 Added all 15 primitives via `npx shadcn-svelte@1.2.7 add ... -y --skip-preflight --no-deps`. Files landed in `src/lib/components/ui/<primitive>/`.
- [x] 6.2 Verified: every primitive imports `cn` (and supporting types) from `$lib/utils.js`; nothing leaks into feature panels.
- [x] 6.3 Installed `bits-ui`, `tailwind-variants`, `@internationalized/date`, `svelte-sonner`, `mode-watcher` (peer dep of sonner — we do NOT use its theme toggle, our `applySemanticTokens` remains the source of truth), `@lucide/svelte`, `lucide-svelte`, plus `csstype` and `skipLibCheck: true` in `tsconfig.json` to satisfy bits-ui type declarations.

## 7. Forms runtime

- [x] 7.1 Installed `felte`, `zod`, `@felte/validator-zod`.
- [x] 7.2 `src/lib/forms/create-form.ts` exports a typed wrapper combining felte's `createForm` with a zod schema (passed to `@felte/validator-zod`).
- [x] 7.3 `src/lib/forms/schemas/.gitkeep` placed.

## 8. Licensing and credits

- [x] 8.1 `THIRD_PARTY_LICENSES.md` created at repo root. List reflects the Tailwind v4 migration: replaces `autoprefixer`/`postcss`/`tailwindcss-animate` with `@tailwindcss/vite` and `tw-animate-css`; adds `csstype`, `bits-ui`, `@internationalized/date`, `svelte-sonner`, `mode-watcher` (with the "not used as theme source" caveat), `@felte/validator-zod`, `@lucide/svelte`.
- [x] 8.2 `src-tauri/tauri.conf.json` `bundle.licenseFile = "../THIRD_PARTY_LICENSES.md"`.
- [x] 8.3 `README.md` gains a `## Credits` section linking the file and acknowledging shadcn-svelte, bits-ui, and `@catppuccin/palette`.

## 9. Tests

- [x] 9.1 `src/test/theme.test.ts` calls `applySemanticTokens(flavour)` for all four flavours under jsdom and asserts every token in `SEMANTIC_MAP_LIGHT`/`SEMANTIC_MAP_DARK` matches `/^\d+(\.\d+)?\s+\d+(\.\d+)?%\s+\d+(\.\d+)?%$/`.
- [x] 9.2 Same suite asserts `html.dark` is `true` for Frappé/Macchiato/Mocha and `false` for Latte.
- [x] 9.3 Same suite: `applyTheme('auto')` + `installAutoThemeListener()` + simulated `matchMedia` change drives the dark class on and off.
- [x] 9.4 `src/test/ui-primitives.test.ts` (under `@vitest-environment jsdom`) imports all 15 primitives, asserts they export Svelte component constructors, mounts `Button`/`Input`/`Label`/`Separator` via `@testing-library/svelte` to prove `cn()` resolves and the renderer works. Auto-cleanup wired via the `svelteTesting()` Vite plugin.
- [x] 9.5 The theme suite includes an allow-list check that none of the semantic-token names overlap with track / waypoint colour fields (`color`, `track-color`, `waypoint-color`, `stroke`, `fill`, `rgba`).

## 10. QA — visual + integration

- [ ] 10.1 Manual smoke per `docs/agent-verification.md`: launch the app and switch through all four flavours (Latte → Frappé → Macchiato → Mocha). Verify `--ctp-*` variables and the new semantic variables both update on each switch, and `html.dark` is set correctly. Existing components must look unchanged. **(Deferred per implementation decision — manual QA is to be run by the user; covered indirectly by §9 vitest assertions.)**
- [ ] 10.2 Render a shadcn `Select` (or `Popover`) above `MapView` in a throwaway harness route (`/dev/popover-smoke` or equivalent) and confirm it portals correctly into `<body>`, does not clip behind the MapLibre canvas, and the colours match the active flavour after a theme switch. **(Deferred — manual.)**
- [ ] 10.3 Confirm a `<input type="color">` placed in the same harness does NOT change colour when the flavour switches — proving the theme system is isolated from per-feature colour inputs (D6 / KD5 constraint). **(Deferred — manual.)**
- [x] 10.4 `just lint && just check && just clippy && just test` all green (104 vitest cases, cargo check + clippy clean, eslint clean once the generated `src/lib/components/ui/**` tree and the `.claude/worktrees/**` mirror were added to the ignore list — both are external generated content).
- [x] 10.5 `just build` (tauri build --debug) produces a working application binary.
