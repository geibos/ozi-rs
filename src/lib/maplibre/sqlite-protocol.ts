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

    // Format: <path>/{z}/{x}/{y}
    const match = rest.match(/^(.+?)\/(\d+)\/(\d+)\/(\d+)$/);
    if (!match) {
      throw new Error(`Invalid sqlite tile URL: ${params.url}`);
    }

    const [, filePath, zStr, xStr, yStr] = match;
    const z = parseInt(zStr, 10);
    const x = parseInt(xStr, 10);
    const y = parseInt(yStr, 10);

    try {
      const data = await getSqliteTile(filePath, z, x, y);
      return { data };
    } catch {
      // Return empty response for missing tiles (coverage boundary)
      return { data: new ArrayBuffer(0) };
    }
  });
}
