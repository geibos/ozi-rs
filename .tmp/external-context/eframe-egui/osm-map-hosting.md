---
source: Official docs and docs.rs
library: eframe / egui
package: eframe-egui
topic: native egui app hosting for map widget
fetched: 2026-03-22T20:17:19Z
official_docs: https://docs.rs/eframe/latest/eframe/
---

Relevant host-app facts
- `eframe` is the recommended crate when writing an app for native or web and using `egui` for everything.
- Native startup is via implementing `eframe::App` and calling `eframe::run_native`; `run_simple_native` exists for simpler native-only apps.
- `eframe` provides native desktop support through Rust-native backends and is documented for desktop/native use.

Integration fit for maps
- The `walkers` quick start is built directly on `eframe::App` and `egui` widgets.
- This means embedding the map is normal widget composition inside `CentralPanel` or any other `egui` container.
