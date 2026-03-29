---
source: docs.rs and GitHub tag/commit pages
library: walkers
package: walkers
topic: walkers 0.53.x compatibility with eframe 0.34.x
fetched: 2026-03-28T08:20:03Z
official_docs: https://docs.rs/crate/walkers/0.53.0
---

# walkers 0.53.x + eframe 0.34.x compatibility

## Current version checked

- Latest `walkers` on docs.rs/crates.io: `0.53.0` (published 2026-03-26).
- No newer `0.53.x` patch release was listed on docs.rs at fetch time.

## Compatibility with `eframe` 0.34.x

- `walkers 0.53.0` depends on `egui = 0.34.0`.
- `walkers 0.53.0` uses `egui_extras = 0.34.0`.
- `walkers 0.53.0` lists `eframe = 0.34.0` as a dev-dependency and its quick-start example embeds the widget in an `eframe::App`.
- `eframe 0.34.0` itself depends on `egui = 0.34.0`.

## Release-note / changelog signal for 0.53.0

From the `0.53.0` tag commit (`e91f75e`):

- `egui` updated to `0.34`.
- MSRV updated to `1.92`.

## Practical compatibility notes

- If your app is already on `eframe 0.34.x` / `egui 0.34.x`, `walkers 0.53.0` is aligned with that stack.
- If your app is still on `eframe 0.33.x` / `egui 0.33.x`, upgrading to `walkers 0.53.0` will pull you onto the newer egui generation.
- Even though `walkers` only declares `eframe` as a dev-dependency, its examples and docs are now written against `eframe 0.34.0`, so integration assumptions should be considered `egui/eframe 0.34`-era.
- Watch the Rust toolchain too: `walkers` moved MSRV from `1.88` in `0.52.0` to `1.92` in `0.53.0`.

## Source links

- walkers crate page: https://docs.rs/crate/walkers/0.53.0
- walkers 0.53.0 Cargo.toml: https://docs.rs/crate/walkers/0.53.0/source/Cargo.toml
- walkers API docs: https://docs.rs/walkers/0.53.0/walkers/
- walkers 0.52.0 Cargo.toml (for comparison): https://docs.rs/crate/walkers/0.52.0/source/Cargo.toml
- walkers 0.53.0 tag page: https://github.com/podusowski/walkers/releases/tag/0.53.0
- walkers 0.53.0 tag commit: https://github.com/podusowski/walkers/commit/e91f75e3d58779616d68b3d790f842696ffd42b5
- eframe 0.34.0 crate page: https://docs.rs/crate/eframe/0.34.0
- eframe 0.34.0 Cargo.toml: https://docs.rs/crate/eframe/0.34.0/source/Cargo.toml
