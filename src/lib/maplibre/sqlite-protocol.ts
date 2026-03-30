import { addProtocol } from "maplibre-gl";
import { getSqliteTile } from "../api";

/**
 * Register a custom `sqlite://` protocol with MapLibre.
 *
 * URL format: `sqlite://<abs-path>/{z}/{x}/{y}`
 *
 * Example source definition:
 * ```json
 * {
 *   "type": "raster",
 *   "tiles": ["sqlite:///home/user/maps/bundle.sqlitedb/{z}/{x}/{y}"],
 *   "tileSize": 256
 * }
 * ```
 */
export function registerSqliteProtocol() {
  addProtocol("sqlite", async (params, _abortController) => {
    // Strip the "sqlite://" prefix
    const rest = params.url.slice("sqlite://".length);

    // Format: <path>/<base_zoom>/{z}/{x}/{y}
    const match = rest.match(/^(.+?)\/(\d+)\/(\d+)\/(\d+)\/(\d+)$/);
    if (!match) {
      throw new Error(`Invalid sqlite tile URL: ${params.url}`);
    }

    const [, filePath, baseZoomStr, zStr, xStr, yStr] = match;
    const baseZoom = parseInt(baseZoomStr, 10);
    const z = parseInt(zStr, 10);
    const x = parseInt(xStr, 10);
    const y = parseInt(yStr, 10);

    try {
      const data = await getSqliteTile(filePath, baseZoom, z, x, y);
      return { data };
    } catch {
      // Return empty response for missing tiles (coverage boundary)
      return { data: new ArrayBuffer(0) };
    }
  });
}
