/**
 * Vitest global setup. Polyfills `localStorage` because Node's experimental
 * implementation surfaces an incomplete object when started without a
 * `--localstorage-file` path, which breaks library code that calls
 * `localStorage.getItem` at module load (notably `mode-watcher`, the peer
 * dependency of `svelte-sonner`).
 */
import { afterEach } from "vitest";

const storeMap = new Map<string, string>();

const inMemoryStorage: Storage = {
  get length() {
    return storeMap.size;
  },
  clear: () => storeMap.clear(),
  getItem: (key) => (storeMap.has(key) ? storeMap.get(key)! : null),
  key: (index) => Array.from(storeMap.keys())[index] ?? null,
  removeItem: (key) => {
    storeMap.delete(key);
  },
  setItem: (key, value) => {
    storeMap.set(key, String(value));
  },
};

if (
  typeof globalThis.localStorage === "undefined" ||
  typeof globalThis.localStorage.getItem !== "function"
) {
  Object.defineProperty(globalThis, "localStorage", {
    value: inMemoryStorage,
    configurable: true,
    writable: true,
  });
}

afterEach(() => {
  storeMap.clear();
});
