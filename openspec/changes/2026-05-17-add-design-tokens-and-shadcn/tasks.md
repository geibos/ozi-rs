## 1. Validate deltas

- [ ] 1.1 Confirm Change 1 (`2026-05-17-migrate-to-sveltekit`) is merged: SvelteKit, `$lib` alias, `routes/`, ESLint flat config, Prettier config, `svelte-check`, and `@testing-library/svelte` are all in place.
- [ ] 1.2 Run `openspec validate 2026-05-17-add-design-tokens-and-shadcn --strict` and confirm the `ui-shell` MODIFIED + ADDED requirements parse with at least one scenario each.
- [ ] 1.3 Confirm the MODIFIED requirement's body matches the full final text of the current entry in `openspec/specs/ui-shell/spec.md` plus the additions described in this change.

## 2. Tailwind v4 install + configuration

- [ ] 2.1 Install `tailwindcss@^4`, `autoprefixer`, `postcss`, `tailwindcss-animate`, `@tailwindcss/typography` as dev dependencies.
- [ ] 2.2 Add `postcss.config.js` enabling `tailwindcss` and `autoprefixer`.
- [ ] 2.3 Create `tailwind.config.ts` with `darkMode: 'class'`, `content` globs covering `src/**/*.{svelte,ts,js}`, the `tailwindcss-animate` + `@tailwindcss/typography` plugins, and a token-aware colour scale (`background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `border`, `input`, `ring`) all using the `hsl(var(--<token>) / <alpha-value>)` pattern.
- [ ] 2.4 Add `@tailwind base; @tailwind components; @tailwind utilities;` to `src/app.css` (created in Change 1).
- [ ] 2.5 Add `prettier-plugin-tailwindcss` to the Prettier config from Change 1 (`.prettierrc` or `prettier.config.js`).
- [ ] 2.6 Run `pnpm vite build --report` (or equivalent) and record the resulting bundle size delta in a comment on the PR; confirm Tailwind purge is active.

## 3. Theme system extension

- [ ] 3.1 In `src/lib/theme.ts`, add a `hsl` rendering helper that takes `@catppuccin/palette` colour `.hsl` and returns the triplet string `` `${h} ${s}% ${l}%` ``.
- [ ] 3.2 Add `SEMANTIC_MAP_LIGHT` (Latte) and `SEMANTIC_MAP_DARK` (Frappé / Macchiato / Mocha) tables mapping each semantic token to a `@catppuccin/palette` colour name per the token mapping table in `design.md`.
- [ ] 3.3 Implement `applySemanticTokens(name: Exclude<ThemeName, 'auto'>)` that (a) writes each semantic token as `` `${h} ${s}% ${l}%` `` to `document.documentElement.style`, (b) toggles `html.classList.toggle('dark', name !== 'latte')`.
- [ ] 3.4 Modify `applyTheme(name)` to call `applySemanticTokens(resolveTheme(name))` after writing the existing `--ctp-*` palette variables (no regression to existing behaviour).
- [ ] 3.5 Ensure `auto` flavour + OS preference flip continues to work end-to-end: the `matchMedia` listener (already in Change 1's layout bootstrap, or added here if missing) re-runs `applyTheme('auto')`.
- [ ] 3.6 Keep `--ctp-*` palette writes unchanged; the two layers coexist on `<html>`.

## 4. `cn()` helper

- [ ] 4.1 Install `clsx` and `tailwind-merge`.
- [ ] 4.2 Create `src/lib/utils.ts` exporting `export function cn(...inputs: ClassValue[]) { return twMerge(clsx(inputs)); }`.

## 5. shadcn-svelte initialisation

- [ ] 5.1 Run `pnpm dlx shadcn-svelte@latest init` with non-interactive flags: base colour `slate` (semantic tokens override it anyway), CSS variables `yes`, components path `$lib/components/ui`, utils path `$lib/utils`, Tailwind config path `tailwind.config.ts`, global CSS `src/app.css`.
- [ ] 5.2 Inspect the generated `components.json`; confirm `componentDir = src/lib/components/ui`, `utils = $lib/utils`, `tailwind.css = src/app.css`.
- [ ] 5.3 Replace the default `@layer base { :root { ... } }` block in `src/app.css` so the semantic-token values come from `applySemanticTokens` at runtime, not from a static block (we still need the variable names declared as fallbacks for SSR/prerender; declare them empty or with the Latte mapping as defaults).

## 6. Install primitives

- [ ] 6.1 Add primitives: `button`, `dialog`, `popover`, `select`, `tabs`, `tooltip`, `switch`, `separator`, `scroll-area`, `input`, `label`, `sonner`, `slider`, `card`, `table` via `pnpm dlx shadcn-svelte@latest add <name>`.
- [ ] 6.2 Verify each landed under `src/lib/components/ui/<name>/`, each imports `cn` from `$lib/utils`, and none accidentally reach into `src/lib/components/<feature>.svelte`.
- [ ] 6.3 Install `lucide-svelte`, `svelte-sonner`, `tailwind-variants`.

## 7. Forms runtime

- [ ] 7.1 Install `felte` and `zod`.
- [ ] 7.2 Create `src/lib/forms/create-form.ts` exporting a thin wrapper that wires `createForm` from `felte` with a zod resolver helper (so feature panels pass a `z.object(...)` schema and get a typed form back).
- [ ] 7.3 Create `src/lib/forms/schemas/.gitkeep` so the empty directory is committed and ready for Change 3 to populate.

## 8. Licensing and credits

- [ ] 8.1 Create `THIRD_PARTY_LICENSES.md` at repo root summarising each new dependency added in this change (Tailwind, shadcn-svelte, bits-ui, lucide-svelte, svelte-sonner, tailwind-variants, clsx, tailwind-merge, felte, zod, `@tailwindcss/typography`, `tailwindcss-animate`, `autoprefixer`, `postcss`, `prettier-plugin-tailwindcss`) with their licences.
- [ ] 8.2 Update `src-tauri/tauri.conf.json` `tauri.bundle.licenseFile` to point at `THIRD_PARTY_LICENSES.md` (or repo `LICENSE` if already wired; this change only needs the third-party file to exist).
- [ ] 8.3 Add a `## Credits` section to `README.md` linking to `THIRD_PARTY_LICENSES.md` and acknowledging `shadcn-svelte`, `bits-ui`, and `@catppuccin/palette`.

## 9. Tests

- [ ] 9.1 Vitest: for each of the four flavours (`latte`, `frappe`, `macchiato`, `mocha`), call `applySemanticTokens(flavour)` against a JSDOM `document` and assert that every semantic token in `SEMANTIC_MAP_LIGHT`/`SEMANTIC_MAP_DARK` is present on `document.documentElement.style` and matches the HSL-triplet regex `/^\d+(\.\d+)?\s+\d+(\.\d+)?%\s+\d+(\.\d+)?%$/`.
- [ ] 9.2 Vitest: assert `html.classList.contains('dark')` is `true` for Frappé/Macchiato/Mocha and `false` for Latte after `applySemanticTokens`.
- [ ] 9.3 Vitest: assert `applyTheme('auto')` followed by a simulated `matchMedia` change re-resolves the dark/light class correctly.
- [ ] 9.4 `@testing-library/svelte` smoke test per installed primitive (button, dialog, popover, select, tabs, tooltip, switch, separator, scroll-area, input, label, sonner, slider, card, table): mount the primitive in a minimal harness and assert it renders without throwing and that `cn()` from `$lib/utils` resolves.
- [ ] 9.5 Vitest: assert that no semantic-token name in `SEMANTIC_MAP_*` overlaps with a track / waypoint colour field (compile-time guard via a typed allow-list).

## 10. QA — visual + integration

- [ ] 10.1 Manual smoke per `docs/agent-verification.md`: launch the app and switch through all four flavours (Latte → Frappé → Macchiato → Mocha). Verify `--ctp-*` variables and the new semantic variables both update on each switch, and `html.dark` is set correctly. Existing components must look unchanged.
- [ ] 10.2 Render a shadcn `Select` (or `Popover`) above `MapView` in a throwaway harness route (`/dev/popover-smoke` or equivalent) and confirm it portals correctly into `<body>`, does not clip behind the MapLibre canvas, and the colours match the active flavour after a theme switch.
- [ ] 10.3 Confirm a `<input type="color">` placed in the same harness does NOT change colour when the flavour switches — proving the theme system is isolated from per-feature colour inputs (D6 / KD5 constraint).
- [ ] 10.4 `just lint && just check && just clippy && just test` all green.
- [ ] 10.5 `tauri build` produces a working application binary.
