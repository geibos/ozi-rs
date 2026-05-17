## MODIFIED Requirements

### Requirement: System provides a Catppuccin theme selector with five options

The system SHALL apply one of the Catppuccin palettes — Auto (follow OS), Latte, Frappé, Macchiato, or Mocha — to the UI via two coexisting CSS custom-property layers on the root element:

1. **Palette layer** — hex variables `--ctp-<colour>` for every named colour in `@catppuccin/palette` (e.g. `--ctp-base`, `--ctp-red`, `--ctp-mauve`).
2. **Semantic layer** — HSL-triplet variables (no `hsl()` wrapper) for shadcn-svelte primitives: `--background`, `--foreground`, `--card`, `--card-foreground`, `--popover`, `--popover-foreground`, `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`, `--muted`, `--muted-foreground`, `--accent`, `--accent-foreground`, `--destructive`, `--destructive-foreground`, `--border`, `--input`, `--ring`.

The semantic-layer values SHALL be derived from the active flavour through two distinct mapping tables — `SEMANTIC_MAP_LIGHT` for Latte and `SEMANTIC_MAP_DARK` for Frappé / Macchiato / Mocha — to keep surface semantics correct in both light and dark modes (e.g. `--popover` resolves to `base` in light and `surface0` in dark).

The root element SHALL carry the class `dark` whenever the resolved flavour is not Latte, so shadcn-svelte's Tailwind dark-variant utilities apply correctly.

The Auto option SHALL continue to track the OS light/dark preference dynamically, re-running both layers on every change.

#### Scenario: Pick a manual theme

- **WHEN** the user selects "Mocha" from the theme picker
- **THEN** the UI re-renders with the Mocha palette applied via `--ctp-*` variables AND every semantic token from `SEMANTIC_MAP_DARK` is written to the root element as an HSL triplet AND the root element gains the `dark` class

#### Scenario: Auto follows OS

- **WHEN** the user selects "Auto" and the OS toggles between light and dark mode
- **THEN** the UI switches between Latte (light, root element has no `dark` class) and Mocha (dark, root element has `dark` class) accordingly, with both the palette and semantic layers re-applied on each transition

#### Scenario: Switching from Latte to Mocha updates semantic tokens

- **WHEN** the user switches the flavour from Latte to Mocha
- **THEN** the semantic CSS variables (e.g. `--background`, `--popover`, `--border`) on the root element change from the Latte-mapped HSL triplets to the Mocha-mapped HSL triplets in the same tick

#### Scenario: shadcn Select renders with theme-correct colours after flavour change

- **WHEN** a shadcn-svelte `Select` is open while the user switches the active flavour
- **THEN** the trigger, popover surface, and items pick up the new flavour's colours through Tailwind utilities reading the semantic CSS variables, with no remount required

#### Scenario: Track colour input is not affected by theme switch

- **WHEN** the user has set a per-track or per-waypoint colour via a domain colour input AND the user then switches the active Catppuccin flavour
- **THEN** the per-track / per-waypoint colour stored in the domain (RGBA bytes) and rendered on the map remains exactly the same value; the theme system SHALL NOT read or write any track or waypoint colour

#### Scenario: Semantic tokens are HSL triplets without `hsl()` wrapper

- **WHEN** the system writes any semantic token to the root element
- **THEN** the value matches the pattern `H S% L%` (e.g. `220 23% 95%`), not `hsl(H, S%, L%)`, so Tailwind utility classes such as `bg-background/80` can apply alpha via the configured `hsl(var(--background) / <alpha-value>)` pattern

## ADDED Requirements

### Requirement: UI primitives are sourced from the shadcn-svelte library

The frontend SHALL provide a `src/lib/components/ui/` directory containing shadcn-svelte primitives generated via `shadcn-svelte init` and `shadcn-svelte add`. The initial primitive set SHALL include: button, dialog, popover, select, tabs, tooltip, switch, separator, scroll-area, input, label, sonner, slider, card, table. Each primitive SHALL import the `cn` helper from `$lib/utils`.

This change introduces the primitives' source code; it does NOT migrate existing feature components (`Console`, `ThemePicker`, `SymbolPicker`, `SimplifyPanel`, `TracksPanel`, `WaypointsPanel`, `TrackPointsPanel`, `Sidebar`, `MapView`). Those keep their current styling until a follow-up change migrates them.

#### Scenario: A primitive renders against the active theme

- **WHEN** a shadcn-svelte `Button` is mounted anywhere in the app and the active flavour is Mocha
- **THEN** its computed styles resolve through the semantic CSS variables to Mocha-derived HSL colours, with no per-component theme wiring required

#### Scenario: `cn()` helper is available at `$lib/utils`

- **WHEN** any primitive or feature component imports `cn` from `$lib/utils`
- **THEN** the helper is defined as `twMerge(clsx(...))` and exists at that import path (locked by `components.json` from `shadcn-svelte init`)

### Requirement: Form panels use felte with zod resolvers

The frontend SHALL provide a `src/lib/forms/create-form.ts` module wrapping `felte`'s `createForm` with a `zod` validation resolver, plus a `src/lib/forms/schemas/` directory ready to hold per-feature zod schemas. Form-based panels added or migrated after this change SHALL build their forms through this module rather than ad-hoc input bindings.

`superforms` is explicitly NOT adopted — it requires SvelteKit form-action runtime which this application does not use.

#### Scenario: A feature panel composes a form

- **WHEN** a feature panel calls the helper exported from `src/lib/forms/create-form.ts` with a zod schema
- **THEN** it receives a typed form whose validation errors and submit handler are wired through felte and zod, without manual schema-to-form glue

### Requirement: Track and waypoint colours are isolated from the theme system

Per-track and per-waypoint colours are domain data (RGBA bytes, interoperable with OziExplorer `COLORREF` import and Garmin named-colour GPX export). The theme system SHALL NOT bind to, read, or write these colours. The semantic-token map SHALL cover only UI chrome (`background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `border`, `input`, `ring`). Per-track and per-waypoint colour inputs SHALL continue to operate on the domain RGBA representation directly.

#### Scenario: Track colour persists across flavour changes

- **WHEN** the user has assigned a specific RGBA colour to a track AND then switches the Catppuccin flavour
- **THEN** the track's stored colour value (in the domain and on disk) is unchanged AND the track renders on the map with that same RGBA colour

#### Scenario: Theme code does not reference track or waypoint colour fields

- **WHEN** the theme module (`src/lib/theme.ts`) is inspected statically
- **THEN** it imports no track or waypoint type and references no domain colour field; the semantic-token allow-list does not overlap with any track or waypoint colour name

### Requirement: Third-party dependencies are credited

The repository SHALL maintain a `THIRD_PARTY_LICENSES.md` file listing the new UI-kit dependencies and their licences (at minimum: Tailwind, shadcn-svelte, bits-ui, lucide-svelte, svelte-sonner, tailwind-variants, clsx, tailwind-merge, felte, zod, `@tailwindcss/typography`, `tailwindcss-animate`, `autoprefixer`, `postcss`, `prettier-plugin-tailwindcss`). The Tauri bundle configuration SHALL reference this file via `tauri.bundle.licenseFile`, and the project README SHALL include a Credits section linking to it.

#### Scenario: A new dependency lands

- **WHEN** a new UI-kit dependency is added to `package.json`
- **THEN** an entry for it exists in `THIRD_PARTY_LICENSES.md` before the change is merged

#### Scenario: The bundle ships the licence file

- **WHEN** `tauri build` produces a release bundle
- **THEN** the bundle includes `THIRD_PARTY_LICENSES.md` via the configured `tauri.bundle.licenseFile` pointer
