# Third-Party Licenses

`ozi-rs` builds on the open-source projects listed below. Each is used under
its respective licence; the full licence text travels with the project's
source in `node_modules/<package>/LICENSE` (or equivalent) after `npm install`.

## UI kit (frontend)

| Package | Licence | Project |
|---|---|---|
| [`tailwindcss`](https://tailwindcss.com) | MIT | Utility-first CSS engine (v4) |
| [`@tailwindcss/vite`](https://tailwindcss.com) | MIT | Vite plugin that wires Tailwind v4 |
| [`@tailwindcss/typography`](https://github.com/tailwindlabs/tailwindcss-typography) | MIT | Prose plugin |
| [`tw-animate-css`](https://github.com/Wombosvideo/tw-animate-css) | MIT | Tailwind v4 animation utilities (successor to `tailwindcss-animate`) |
| [`prettier-plugin-tailwindcss`](https://github.com/tailwindlabs/prettier-plugin-tailwindcss) | MIT | Class sorter for Prettier |
| [`shadcn-svelte`](https://shadcn-svelte.com) | MIT | Primitive scaffolding CLI |
| [`bits-ui`](https://bits-ui.com) | MIT | Headless primitives consumed by shadcn-svelte |
| [`tailwind-variants`](https://www.tailwind-variants.org) | MIT | Variant authoring API used by primitives |
| [`tailwind-merge`](https://github.com/dcastil/tailwind-merge) | MIT | Class-conflict deduper inside `cn()` |
| [`clsx`](https://github.com/lukeed/clsx) | MIT | Conditional class composer inside `cn()` |
| [`@lucide/svelte`](https://lucide.dev) / [`lucide-svelte`](https://lucide.dev) | ISC | Icon set used by primitives |
| [`svelte-sonner`](https://github.com/wobsoriano/svelte-sonner) | MIT | Toast primitive (sonner port) |
| [`mode-watcher`](https://github.com/svecosystem/mode-watcher) | MIT | Peer dep of `svelte-sonner`; not used as a theme source — `$lib/theme` remains canonical |
| [`@internationalized/date`](https://react-spectrum.adobe.com/internationalized/date/) | Apache-2.0 | Calendar/date primitives required by some bits-ui components |
| [`felte`](https://felte.dev) | MIT | Form runtime |
| [`@felte/validator-zod`](https://felte.dev/docs/svelte/validators) | MIT | Zod adapter for felte |
| [`zod`](https://zod.dev) | MIT | Schema validation |
| [`csstype`](https://github.com/frenic/csstype) | MIT | CSS property types (transitive but listed here because we install it explicitly to satisfy bits-ui's type declarations) |

## Existing dependencies acknowledged (not introduced by this change)

| Package | Licence | Notes |
|---|---|---|
| [`@catppuccin/palette`](https://github.com/catppuccin/palette) | MIT | Canonical Catppuccin colour source (4 flavours) |
| [`svelte`](https://svelte.dev) | MIT | UI framework |
| [`@sveltejs/kit`](https://kit.svelte.dev) | MIT | App framework |
| [`maplibre-gl`](https://maplibre.org) | BSD-3-Clause | Map renderer |
| [`@tauri-apps/api`](https://tauri.app) | Apache-2.0 OR MIT | Tauri runtime API |

## Rust crates (backend)

Rust dependencies are tracked in `Cargo.toml` / `Cargo.lock`; their licences
are bundled into the produced binary via the standard Cargo licence-tracking
machinery and are not duplicated here.
