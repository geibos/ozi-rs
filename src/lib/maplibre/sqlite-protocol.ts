import { addProtocol } from "maplibre-gl";
import { getSqliteTile } from "../api";
import { parseSqliteTileUrl } from "./tile-url";

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
    const parsed = parseSqliteTileUrl(params.url);
    if (!parsed) {
      throw new Error(`Invalid sqlite tile URL: ${params.url}`);
    }

    const { filePath, baseZoom, z, x, y } = parsed;

    try {
      const data = await getSqliteTile(filePath, baseZoom, z, x, y);
      return { data };
    } catch {
      // Return empty response for missing tiles (coverage boundary)
      return { data: new ArrayBuffer(0) };
    }
  });
}
