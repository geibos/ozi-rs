export interface SqliteTileUrl {
  filePath: string;
  baseZoom: number;
  z: number;
  x: number;
  y: number;
}

/** Parse `sqlite://<path>/<base_zoom>/<z>/<x>/<y>` → params, or null if malformed. */
export function parseSqliteTileUrl(url: string): SqliteTileUrl | null {
  const rest = url.slice("sqlite://".length);
  const match = rest.match(/^(.+?)\/(\d+)\/(\d+)\/(\d+)\/(\d+)$/);
  if (!match) return null;
  const [, filePath, baseZoomStr, zStr, xStr, yStr] = match;
  return {
    filePath,
    baseZoom: parseInt(baseZoomStr, 10),
    z: parseInt(zStr, 10),
    x: parseInt(xStr, 10),
    y: parseInt(yStr, 10),
  };
}
