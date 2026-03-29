---
source: docs.rs and official GitHub releases
library: eframe / egui
package: eframe-egui
topic: eframe 0.34.x App trait shape and ui migration
fetched: 2026-03-28T08:25:00Z
official_docs: https://docs.rs/eframe/0.34.0/eframe/trait.App.html
---

# eframe 0.34.x `App` trait shape

Version context checked:
- `eframe` 0.34.0 trait docs on docs.rs
- `eframe` 0.33.0 trait docs on docs.rs for comparison
- `egui`/`eframe` 0.34.0 and 0.34.1 GitHub release notes

## Current trait shape in 0.34.x

`eframe::App` now requires `ui`, not `update`:

```rust
pub trait App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame);

    fn logic(&mut self, ctx: &egui::Context, frame: &mut Frame) { ... }

    #[deprecated = "Use Self::ui instead"]
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) { ... }

    fn save(&mut self, _storage: &mut dyn Storage) { ... }
    fn on_exit(&mut self, ...) { ... }
    fn auto_save_interval(&self) -> std::time::Duration { ... }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] { ... }
    fn persist_egui_memory(&self) -> bool { ... }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) { ... }
}
```

Key point for the compile error:
- In `eframe` 0.33.x, the required trait item was `update(&mut self, ctx: &egui::Context, frame: &mut Frame)`.
- In `eframe` 0.34.x, the required trait item is `ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame)`.
- `update` still exists, but only as a provided deprecated method, so implementing only `update` now triggers `not all trait items implemented, missing: ui`.

## Relevant migration note from 0.34.0 release

Release headline: **"More `Ui`, less `Context`"**.

The 0.34.0 release notes say:
- `eframe` deprecated `App::update` and replaced it with `App::ui`, which provides `&mut Ui` instead of `&Context`.
- `Ui` now dereferences to `Context`, so many former `ctx` calls can move to `ui` directly.
- `logic` was added for non-painting work that should run before `ui` and even when the UI is hidden but repaint was requested.

## Minimal adaptation guidance

If you have old 0.33-style code like:

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello");
        });
    }
}
```

Adapt to 0.34.x as:

```rust
impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.label("Hello");
    }
}
```

If you still want panel/frame styling around the app root, docs say the provided `Ui` has no margin or background color, so wrap it inside a panel/frame shown *inside* the root `Ui`, e.g. use APIs such as:
- `egui::CentralPanel` / `show_inside`
- `egui::Frame::central_panel(...)`

## Practical migration implications

- Replace `fn update(&mut self, ctx: &egui::Context, ...)` with `fn ui(&mut self, ui: &mut egui::Ui, ...)`.
- Move top-level widget building to use `ui` directly.
- For calls that used `ctx`, first try the same call on `ui`, because `Ui` derefs to `Context` in 0.34.
- Keep side effects or background orchestration that should happen without showing widgets in `fn logic(...)`.
- Do not rely on `update` as the trait requirement anymore; it is deprecated and not the required item.

## Notes on 0.34.1

- `0.34.1` release notes do not introduce a new `App`-trait migration; it is a small follow-up release.
- The trait-shape change relevant to the `missing: ui` error is the 0.34.0 migration.

## Source links

- 0.34.0 `eframe::App` docs: https://docs.rs/eframe/0.34.0/eframe/trait.App.html
- 0.34.0 `epi.rs` source on docs.rs: https://docs.rs/crate/eframe/0.34.0/source/src/epi.rs
- 0.33.0 `eframe::App` docs for comparison: https://docs.rs/eframe/0.33.0/eframe/trait.App.html
- 0.34.0 release notes: https://github.com/emilk/egui/releases/tag/0.34.0
- 0.34.1 release notes: https://github.com/emilk/egui/releases/tag/0.34.1
