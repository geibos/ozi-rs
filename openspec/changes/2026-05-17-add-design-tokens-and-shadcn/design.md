## Context

This change implements **Change 2** of the three-change shadcn UI-kit migration plan defined in `docs/superpowers/specs/2026-05-17-shadcn-ui-kit-svelte-design.md`. The parent design doc is the source of truth — read it in full before touching this code. The following decisions from that document are load-bearing for this change:

- **D1** — Stay on Svelte; adopt `shadcn-svelte` + `bits-ui` rather than migrating to React.
- **D4** — Catppuccin (via `@catppuccin/palette`) remains the canonical colour source. We extend `theme.ts`, not replace it.
- **D5** — Two coexisting CSS variable layers on `<html>`: the existing `--ctp-*` hex palette layer (consumed by existing components and the MapLibre placeholder) and a new HSL-triplet semantic layer (`--background`, `--foreground`, …) consumed by shadcn-svelte and Tailwind utilities. A `class="dark"` toggle is set on `<html>` whenever the active flavour is not Latte.
- **D6** — Track and waypoint colours live in the domain as RGBA (`TrackStyle.color: [u8; 4]`) and stay there. The theme system must never bind to them.
- **D7** — Forms via `felte` + `zod`. `superforms` is rejected because it needs SvelteKit form actions, which we do not use.
- **D8** — Industry-standard tooling baseline; `prettier-plugin-tailwindcss` is added in this change on top of the Prettier config from Change 1.

This change depends on Change 1 (`2026-05-17-migrate-to-sveltekit`) being already merged: SvelteKit, the `$lib` alias, route layout, ESLint flat config, the base Prettier config, `svelte-check` in CI, and `@testing-library/svelte` must all exist before this change starts.

## Key decisions

### KD1 — Semantic tokens are HSL triplets without the `hsl()` wrapper

We emit `--background: 220 23% 95%;` (just the numbers), not `--background: hsl(220, 23%, 95%);`. The Tailwind config wraps them with `hsl(var(--background) / <alpha-value>)`. This is the standard shadcn pattern; it lets Tailwind utility classes such as `bg-background/80` apply alpha without a separate variable. `@catppuccin/palette` exposes `.hsl = {h, s, l}` per colour, which we render to the template literal `` `${h} ${s}% ${l}%` ``.

### KD2 — Two coexisting CSS variable layers on `<html>`

`applyTheme(name)` continues to write `--ctp-*` hex variables (consumed by existing components and the MapLibre placeholder background). `applySemanticTokens(name)` additionally writes the HSL-triplet semantic tokens. Both run on every theme switch; both live on the same `<html>` element. The existing nine components keep reading `--ctp-*`; shadcn primitives and any new Tailwind utility classes read the semantic layer. This lets Change 3 migrate components one by one without breaking the rest.

### KD3 — Light vs dark mapping branches

`popover` resolves to `base` in light and to `surface0` in dark to avoid an inverted surface. Similarly for `card`, `muted`, and `accent`. We keep two distinct tables (`SEMANTIC_MAP_LIGHT` for Latte, `SEMANTIC_MAP_DARK` for Frappé / Macchiato / Mocha) and select between them by the resolved flavour name, not by `prefers-color-scheme` (`auto` resolves to Latte or Mocha first, then picks the table).

### KD4 — `class="dark"` toggle, `mode-watcher` rejected

shadcn-svelte's dark-variant Tailwind utilities are gated on `html.dark`. We set the class manually inside `applySemanticTokens` whenever the resolved flavour is not Latte. `mode-watcher` is rejected because we already have a richer four-flavour state machine in `theme.ts` and adding a second source of truth would mean two systems racing on `<html>` classes.

### KD5 — Track and waypoint colour isolation from theme

`TrackStyle.color: [u8; 4]` (RGBA bytes) and waypoint colour fields are domain data, interoperable with OziExplorer (`COLORREF` import via PLT, Garmin named-colour GPX export). The theme system MUST NOT read or write these colours. The semantic-token map only covers UI chrome (`background`, `foreground`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `border`, `input`, `ring`, `popover`, `card`). Per-track colour pickers continue to write to the domain as bytes; Change 3 will keep the native `<input type="color">` for that reason.

### KD6 — `cn()` helper in `$lib/utils`

shadcn-svelte's generated primitives import `cn` from `$lib/utils`. We provide it as `twMerge(clsx(...))` — `clsx` handles conditional class composition, `tailwind-merge` dedupes conflicting Tailwind classes. The path `$lib/utils` is locked by `shadcn-svelte init`'s `components.json` and must match.

### KD7 — Primitives are scaffolded, not pre-styled

`shadcn-svelte init` and per-primitive `add` commands generate component sources under `src/lib/components/ui/<primitive>/`. We commit them as-is — no further visual tweaking in this change. Change 3 may adjust them when migrating panels.

## Token mapping table

Adapted from the parent design's "Catppuccin → semantic token mapping" section. Values reference `@catppuccin/palette` colour names; each entry resolves to the HSL triplet of that named colour in the active flavour.

| Semantic token | Light (Latte) | Dark (Frappé / Macchiato / Mocha) | Consumer |
|---|---|---|---|
| `--background` | `base` | `base` | Page background |
| `--foreground` | `text` | `text` | Default text |
| `--card` | `mantle` | `surface0` | Card surfaces |
| `--card-foreground` | `text` | `text` | Text on cards |
| `--popover` | `base` | `surface0` | Popover / dialog surfaces |
| `--popover-foreground` | `text` | `text` | Text on popovers |
| `--primary` | `blue` | `blue` | Primary actions |
| `--primary-foreground` | `base` | `crust` | Text on primary |
| `--secondary` | `surface0` | `surface1` | Secondary actions |
| `--secondary-foreground` | `text` | `text` | Text on secondary |
| `--muted` | `surface0` | `surface0` | Muted backgrounds |
| `--muted-foreground` | `subtext1` | `subtext0` | Muted text |
| `--accent` | `surface1` | `surface1` | Hover / accent surface |
| `--accent-foreground` | `text` | `text` | Text on accent |
| `--destructive` | `red` | `red` | Destructive actions |
| `--destructive-foreground` | `base` | `crust` | Text on destructive |
| `--border` | `surface1` | `surface2` | Default borders |
| `--input` | `surface1` | `surface2` | Form input borders |
| `--ring` | `lavender` | `lavender` | Focus rings |

The exact light/dark resolution per token is encoded in `SEMANTIC_MAP_LIGHT` and `SEMANTIC_MAP_DARK`. The parent design doc remains the canonical reference if disputes arise during implementation.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Tailwind bundle bloat enlarges the Tauri webview payload | Low | Tailwind v4 content-purge is on by default; verify `vite build --report` after install and document the size delta in tasks 8.x |
| shadcn `Popover` / `Select` clip or z-fight with the MapLibre canvas | Medium | bits-ui portals popovers into `<body>` by default. Smoke test "open a shadcn `Select` over `MapView`" is in the QA section to catch regressions before Change 3 starts using these primitives in panels |
| Semantic-token HSL parsing breaks because we accidentally emit `hsl(...)` instead of the triplet | Medium | Vitest test per flavour asserts the emitted value matches `/^\d+(\.\d+)?\s+\d+(\.\d+)?%\s+\d+(\.\d+)?%$/` |
| Track / waypoint colour inputs get inadvertently bound to semantic tokens later | Low | Spec requirement (this change) explicitly forbids it; Change 3 design will repeat the constraint per affected panel |
| `class="dark"` toggle desynchronises from the active flavour (e.g. on `auto` + OS flip) | Medium | The toggle lives inside `applySemanticTokens`, which is the single entry point called from both `applyTheme` and the `matchMedia` change listener; covered by a Vitest test that flips OS preference under `auto` and asserts `html.classList` |
| Existing components break because they read `--ctp-*` and a primitive overrides via Tailwind utility | Low | The two layers are namespaced (`--ctp-*` vs `--background` / `--foreground` / …) and never collide; no existing component is migrated in this change |
| `shadcn-svelte init` writes a `components.json` that conflicts with our directory layout | Low | Pre-set `componentDir` to `src/lib/components/ui` and `utils` to `$lib/utils` during init; verify the resulting `components.json` matches before installing primitives |
