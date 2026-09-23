import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from "node:url";

/**
 * The stand: the real frontend, served in a browser, with the Tauri transport
 * answered from the fixtures the Rust core writes.
 *
 * Every visual check so far has cost a full `just build`, an Appium session
 * that owns the screen, and a window that has to be caught in the right state.
 * This opens the same screens in a browser tab in a second, at any size, with
 * devtools — and the data is the same bytes the backend sends, because the
 * fixtures come from the same mappers the commands use.
 *
 * It is not a replacement for the smoke gate: nothing here proves the Rust
 * side works. It proves what a screen looks like and how it behaves with a
 * given state, which is the part that kept regressing unseen.
 */
const stand = (name: string) =>
  fileURLToPath(new URL(`./src/test/stand/${name}.ts`, import.meta.url));

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  clearScreen: false,
  resolve: {
    alias: [
      { find: "@tauri-apps/api/core", replacement: stand("tauri-core") },
      { find: "@tauri-apps/api/event", replacement: stand("tauri-event") },
      { find: "@tauri-apps/api/window", replacement: stand("tauri-window") },
      {
        find: "@tauri-apps/api/webview",
        replacement: stand("tauri-webview"),
      },
      { find: "@tauri-apps/plugin-dialog", replacement: stand("tauri-dialog") },
    ],
  },
  server: {
    port: 5273,
    strictPort: true,
  },
});
