import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      // Tauri loads `build/index.html` at startup, which is the prerendered
      // `/` route. The fallback (used for unknown paths during dev/preview)
      // lives at `200.html` so it does not overwrite the prerendered entry.
      fallback: "200.html",
    }),
    alias: {
      $lib: "src/lib",
    },
  },
};
