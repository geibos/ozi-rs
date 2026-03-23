---
source: Official docs and docs.rs
library: eframe / egui
package: eframe-egui
topic: rust desktop map editor evaluation
fetched: 2026-03-22T12:08:31Z
official_docs: https://docs.rs/eframe/latest/eframe/
---

Version context
- `eframe` 0.33.3 and `egui` 0.33.3 on docs.rs.

Startup complexity
- `eframe` native startup is minimal: implement `eframe::App` and call `eframe::run_native`, or use `run_simple_native` for native-only apps.
- The docs position `eframe` as the path to use when you want `egui` for everything on web or native.

Native desktop support
- `eframe` supports native desktop using `winit` with `glow` by default, and optional `wgpu`.
- Official integrations include `eframe`, `egui_glow`, `egui_wgpu`, and `egui_winit`.
- `egui`/`eframe` support Linux, macOS, Windows, and Android; `egui` also supports multiple native viewports where the backend exposes them.

Iteration speed / prototyping
- `egui` describes itself as an "easy-to-use immediate mode GUI" and aims to be the easiest-to-use Rust GUI library.
- Immediate mode keeps UI logic in self-contained functions like `if ui.button("Save").clicked() { ... }`, avoiding callback-heavy wiring.
- `egui` explicitly targets responsiveness in debug builds and says it is well suited for highly interactive applications.

Architecture implications
- `egui` docs say the library does not store your application state; the user stores state and `egui` only reads and mutates it frame to frame.
- `egui` is a library, not a framework, and can be integrated into an existing engine or backend by feeding `RawInput` and handling `FullOutput`.
- This supports keeping domain/application state outside the UI layer, but immediate-mode code can still tempt you to mix commands and drawing unless you enforce boundaries yourself.

Custom canvas / rendering suitability
- `egui` aims to provide a simple 2D graphics API via `epaint`.
- Integrations can render `egui` anywhere you can draw textured triangles.
- `PaintCallback` / `Shape::Callback` lets you render custom content in an egui region using the backend rendering context.
- Render-to-texture is also documented as an option for embedding custom rendering inside the UI.

Packaging / runtime model
- `eframe` is a pure Rust desktop app path; there is no separate JS/web runtime in the desktop path.
- Backend choice is Rust-native (`glow` or optional `wgpu`) and packaging follows normal Rust desktop-app distribution practices.

Likely downsides / lock-in risks
- `egui` says it is in heavy development, with breaking changes in new versions.
- The README says it is still work in progress and not the right choice if you want something that does not break on upgrade yet.
- Immediate mode has documented downsides: harder layout in some cases, possible first-frame jitter, and potential CPU cost when laying out large/complex UIs each frame.
- `egui` explicitly does not target native-looking UI.
