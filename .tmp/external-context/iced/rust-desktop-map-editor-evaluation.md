---
source: Official docs, docs.rs, and iced book
library: iced
package: iced
topic: rust desktop map editor evaluation
fetched: 2026-03-22T12:08:31Z
official_docs: https://docs.rs/iced/latest/iced/
---

Version context
- `iced` 0.14.0 on docs.rs.

Startup complexity
- The shortest path is `iced::run(update, view)`.
- More customization uses `iced::application(new, update, view).theme(...).subscription(...).run()`.
- The crate is documented as inspired by Elm and centers everything around state, messages, update logic, and view logic.

Native desktop support
- `iced` docs call it a cross-platform GUI library and a native, cross-platform, multi-windowed application framework.
- The default renderer re-exports `wgpu`; the widget module also exposes a `shader` widget for `wgpu` applications.

Iteration speed / prototyping
- Basic startup is small, but the crate warns that `iced` is experimental and can be frustrating if you expect hand-holding.
- The docs say it leverages ownership, borrowing, lifetimes, futures, streams, trait bounds, and closures heavily.
- That suggests a slower initial ramp than `egui`, especially for a team wanting a working window quickly.

Architecture implications
- The iced book explicitly says one of the main advantages of the Elm Architecture is that state, messages, and update logic do not need to know about the UI library at all.
- The docs show `update`/`view`/`Message` composition and scaling via nested screens and mapped actions/tasks.
- For clean domain/application separation, this is the strongest out-of-the-box architectural fit of the three options.

Custom canvas / rendering suitability
- `iced::widget::canvas` is available behind the `canvas` feature and is specifically for interactive 2D graphics.
- `Canvas` uses a `Program` trait and can draw paths, text, images, and cached geometry.
- `iced::widget::shader` is available for `wgpu` applications when lower-level custom rendering is needed.
- This makes iced a strong technical fit for a map canvas/editor surface.

Packaging / runtime model
- `iced` is a pure Rust native GUI stack, without a JS/webview frontend requirement in the desktop case.
- Runtime and rendering stay in the Rust process model.

Likely downsides / lock-in risks
- The crate explicitly labels itself experimental.
- The docs state the library is unforgiving and assumes strong Rust knowledge.
- The Elm-style message architecture is beneficial for separation, but it is also a structural commitment that can feel heavy for fast UI sketching.
- Feature-gated canvas/shader paths and advanced type signatures can raise complexity for early MVP iteration.
