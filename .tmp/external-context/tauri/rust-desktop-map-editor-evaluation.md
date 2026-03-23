---
source: Official Tauri docs
library: Tauri
package: tauri
topic: rust desktop map editor evaluation
fetched: 2026-03-22T12:08:31Z
official_docs: https://v2.tauri.app/start/
---

Version context
- Tauri v2 docs.

Startup complexity
- Tauri project creation uses `create-tauri-app` or `tauri init`.
- The standard setup expects a frontend stack plus Rust backend setup, often with package manager, frontend template, dev server URL, frontend dev command, and frontend build command.
- The project structure docs show a typical two-part app: top-level JS project plus `src-tauri/` Rust project.

Native desktop support
- Tauri targets desktop apps across major platforms and builds platform-specific installers and bundles.
- The core uses Rust plus TAO for windows and WRY for webview rendering.
- The process model docs say it uses WebView2 on Windows, WKWebView on macOS, and webkitgtk on Linux.

Iteration speed / prototyping
- If the team is already moving quickly in web UI tech, Tauri can be fast.
- For a Rust-first desktop MVP, it adds extra moving parts: frontend tooling, web assets, Tauri config, capability files, and Rust/JS integration.
- It is less direct than a pure Rust GUI if the immediate goal is "show a working window quickly" from this repo.

Architecture implications
- Tauri's process model explicitly separates a Rust core process from webview processes.
- The docs recommend keeping global state and business-sensitive data in the core process and deferring as much business logic as possible to the Core.
- This separation supports clean boundaries very well, but it introduces IPC/message-passing and a split Rust/web frontend boundary even when the product is desktop-first.

Custom canvas / rendering suitability
- Tauri's UI is HTML/CSS/JS in a webview.
- That gives access to browser canvas/WebGL/WebGPU style rendering approaches, but the map editor canvas becomes a web frontend concern instead of a Rust-native rendering surface.
- For a Rust desktop map editor, that increases cross-stack coordination and can dilute the repo's desired domain/application/UI layering inside one Rust codebase.

Packaging / runtime model
- Tauri apps use the system webview instead of bundling a browser engine; docs say a minimal app can be less than 600KB.
- Apps are dynamically linked to platform webviews rather than shipping the webview runtime inside the final executable.
- Distribution is broad: DMG, Windows installer, Debian, RPM, AppImage, Snap, store paths, and signing/notarization guidance.

Likely downsides / lock-in risks
- Platform webview differences matter because the app depends on system webviews at runtime.
- The app architecture is intrinsically polyglot for the common case: web frontend plus Rust core.
- Tauri-specific config, capability files, CLI/build flow, and IPC conventions are additional framework surface area.
- For a graphics-heavy map editor, webview rendering and browser-engine behavior become part of the long-term constraints.
