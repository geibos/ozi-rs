import { addProtocol } from "maplibre-gl";
import { getOziTileProjected } from "../api";

/**
 * Register a custom `ozi://` protocol with MapLibre for OZF2 raster maps.
 *
 * For each MapLibre tile request the Rust backend reprojects the OZF2 raster
 * into a 256×256 Web Mercator tile — no coordinate math is done in JS.
 *
 * URL format: `ozi://<abs-path-to-.map-file>/{z}/{x}/{y}`
 */
export function registerOziProtocol() {
  addProtocol("ozi", async (params, _abortController) => {
    const rest = params.url.slice("ozi://".length);
    const match = rest.match(/^(.+?)\/(\d+)\/(\d+)\/(\d+)$/);
    if (!match) {
      throw new Error(`Invalid ozi tile URL: ${params.url}`);
    }

    const [, mapPath, zStr, xStr, yStr] = match;
    const tz = parseInt(zStr, 10);
    const tx = parseInt(xStr, 10);
    const ty = parseInt(yStr, 10);

    try {
      const data = await getOziTileProjected(mapPath, tx, ty, tz);
      return { data };
    } catch {
      // Out-of-bounds or missing tiles are expected at map edges — return empty.
      return { data: new ArrayBuffer(0) };
    }
  });
}
