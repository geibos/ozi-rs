import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";
import { fileURLToPath } from "node:url";
import * as fs from "node:fs";
import * as path from "node:path";

// When `vitest` runs from a git worktree, the worktree's project root has
// no `node_modules` of its own — packages are resolved up the directory
// tree to the parent repo. Vite's default `server.fs.allow` is the project
// root only, so `/@fs/...` lookups for `@testing-library/svelte` fail with
// "Cannot find module" even though the file exists on disk. Walk up until
// we find a `node_modules` directory and grant Vite filesystem access to
// it. No-op when the worktree itself has a `node_modules` (the parent path
// is added redundantly but harmlessly).
function resolveNodeModulesAncestors(start: string): string[] {
  const results: string[] = [];
  let dir = start;
  for (let i = 0; i < 8; i += 1) {
    const candidate = path.join(dir, "node_modules");
    if (fs.existsSync(candidate)) results.push(candidate);
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return results;
}

const projectRoot = fileURLToPath(new URL(".", import.meta.url));
const nodeModulesAllow = resolveNodeModulesAncestors(projectRoot);

export default defineConfig({
  plugins: [svelte({ hot: false }), svelteTesting()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
      $app: fileURLToPath(new URL("./src/test/stubs/app", import.meta.url)),
    },
  },
  server: {
    fs: {
      allow: [projectRoot, ...nodeModulesAllow],
    },
  },
  test: {
    environment: "node",
    include: ["src/test/**/*.test.ts"],
    setupFiles: ["./src/test/setup.ts"],
  },
});
