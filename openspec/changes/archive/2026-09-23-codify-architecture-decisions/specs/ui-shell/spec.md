## ADDED Requirements

### Requirement: MapLibre GL is the sole map engine in the frontend

The map canvas SHALL be rendered by MapLibre GL JS (major version 4). Map-specific integration code — custom tile protocols, track layer helpers, tile URL helpers — SHALL live under `src/lib/maplibre/`, and no second web-map library SHALL be introduced.

#### Scenario: Dependency set names one map engine

- **WHEN** `package.json` dependencies are read
- **THEN** `maplibre-gl` is present and none of `leaflet`, `ol` or `openlayers` is

#### Scenario: Map integration lives in one module

- **WHEN** `src/lib/maplibre/` is listed
- **THEN** it contains `sqlite-protocol.ts`, `ozi-protocol.ts` and `tracks-layer.ts`

#### Scenario: MapView constructs the map from MapLibre

- **WHEN** `src/components/MapView.svelte` is inspected
- **THEN** the map instance is created from `maplibre-gl` and no other map constructor is imported

### Requirement: Icons come from Lucide via `@lucide/svelte`

Iconography SHALL use the Lucide set imported from `@lucide/svelte`, the `iconLibrary` declared in `components.json`. Components SHALL NOT import the legacy `lucide-svelte` package and SHALL NOT use emoji or text glyphs in place of an interface icon — a button, a menu entry, a status indicator.

A waypoint's symbol is not an interface icon. The symbols a crew picks from — a flag, a pin, a cross — are the mark's own meaning, they travel with it to the next headquarters through `.wpt` and GPX, and they are drawn on the map rather than in the interface. They are glyphs by choice (`src/lib/waypoint-symbols.ts`), which is why this requirement says "in place of an interface icon" rather than "anywhere".

#### Scenario: Registry declares Lucide

- **WHEN** `components.json` is read
- **THEN** `iconLibrary` is `lucide`

#### Scenario: Only the current package is imported

- **WHEN** `src/` is searched for icon imports
- **THEN** every icon import resolves to `@lucide/svelte` and no file imports from `lucide-svelte`

#### Scenario: A waypoint's symbol is the mark's, not the interface's

- **WHEN** a waypoint carries the `flag` symbol
- **THEN** the map draws its glyph, and that is not a breach of this
  requirement

### Requirement: Toasts and tooltips are hosted once in the root layout

`src/routes/+layout.svelte` SHALL mount exactly one `<Toaster>` (the shadcn-svelte `sonner` wrapper over `svelte-sonner`) and SHALL wrap the application in a single `Tooltip.Provider`. Feature components SHALL emit notifications through `toast()` and use the `Tooltip` primitives without mounting their own hosts.

#### Scenario: Single Toaster host

- **WHEN** `src/` is searched for `<Toaster`
- **THEN** the only match is in `src/routes/+layout.svelte`

#### Scenario: Tooltip provider wraps the shell

- **WHEN** `src/routes/+layout.svelte` is inspected
- **THEN** a `Tooltip.Provider` element encloses the routed content and the `Toaster`
