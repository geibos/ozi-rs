import { setWorkerUrl } from "maplibre-gl";
import workerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url";

/**
 * Point MapLibre at its worker, as the bundler emitted it.
 *
 * MapLibre 6 ships as ES modules and finds its worker from its own
 * `import.meta.url` — and only when that is an http(s) URL. Bundled, the
 * module is a chunk under `_app/immutable/`, not beside the worker; in the
 * packaged application the page is `tauri://localhost`, which is not http at
 * all. Either way the default resolves to nothing, and a map without workers
 * draws no GeoJSON: no tracks, no drawing, no labels. The worker also imports
 * a shared chunk, so copying the file as a plain asset would lose half of it;
 * `?worker&url` has Vite bundle it with its imports and hand back the URL.
 */
export function configureMapLibreWorker(): void {
  setWorkerUrl(workerUrl);
}
