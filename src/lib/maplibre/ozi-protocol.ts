import { addProtocol } from "maplibre-gl";
import { getOziTile, getOziMetadata } from "../api";
import type { OziMetadataDto, OziLevelDto } from "../types";

/**
 * Register a custom `ozi://` protocol with MapLibre for OZF2 raster maps.
 *
 * OZF2 maps use their own tile grid (not Web Mercator), so this protocol
 * translates MapLibre z/x/y requests to OZF2 level/tileX/tileY coordinates.
 *
 * URL format: `ozi://<abs-path-to-.map-file>/{z}/{x}/{y}`
 *
 * The OZI map path (.map file) identifies both the metadata and the paired
 * .ozf2 raster file.
 */

interface OziMapState {
  meta: OziMetadataDto;
  georeference: AffineTransform;
}

interface AffineTransform {
  scaleX: number;
  offsetX: number;
  scaleY: number;
  offsetY: number;
}

const metadataCache = new Map<string, OziMapState>();

async function getOrLoadMeta(mapPath: string): Promise<OziMapState> {
  const cached = metadataCache.get(mapPath);
  if (cached) return cached;

  const meta = await getOziMetadata(mapPath);
  const georeference = parseGeoreference(meta.calibration_points);
  if (!georeference) {
    throw new Error(`Could not parse georeference for ${mapPath}`);
  }

  const state: OziMapState = { meta, georeference };
  metadataCache.set(mapPath, state);
  return state;
}

export function registerOziProtocol() {
  addProtocol("ozi", async (params, _abortController) => {
    const rest = params.url.slice("ozi://".length);
    const match = rest.match(/^(.+?)\/(\d+)\/(\d+)\/(\d+)$/);
    if (!match) {
      throw new Error(`Invalid ozi tile URL: ${params.url}`);
    }

    const [, mapPath, zStr, xStr, yStr] = match;
    const z = parseInt(zStr, 10);
    const x = parseInt(xStr, 10);
    const y = parseInt(yStr, 10);

    try {
      const { meta, georeference } = await getOrLoadMeta(mapPath);

      // Pick best level from available OZF2 levels
      const level = pickLevel(meta.levels, z);
      if (level === null) return { data: new ArrayBuffer(0) };

      // Convert Web Mercator tile to OZF2 tile coordinates
      const ozCoords = webMercatorTileToOzi(x, y, z, level, georeference);
      if (!ozCoords) return { data: new ArrayBuffer(0) };

      const data = await getOziTile(
        mapPath,
        level.level_index,
        ozCoords.tileX,
        ozCoords.tileY
      );
      return { data };
    } catch {
      return { data: new ArrayBuffer(0) };
    }
  });
}

// ── Coordinate helpers ────────────────────────────────────────────────────────

function webMercatorTileToOzi(
  x: number,
  y: number,
  z: number,
  level: OziLevelDto,
  geo: AffineTransform
): { tileX: number; tileY: number } | null {
  // Convert tile corner to lat/lon (top-left of tile)
  const { lat, lon } = tileToLatLon(x, y, z);

  // Project to OZF2 pixel coordinates using the georeference affine transform
  const pixelX = geo.scaleX * lon + geo.offsetX;
  const pixelY = geo.scaleY * lat + geo.offsetY;

  const tileX = Math.floor(pixelX / level.tile_width);
  const tileY = Math.floor(pixelY / level.tile_height);

  if (
    tileX < 0 ||
    tileY < 0 ||
    tileX >= level.tile_columns ||
    tileY >= level.tile_rows
  ) {
    return null;
  }

  return { tileX, tileY };
}

function tileToLatLon(
  x: number,
  y: number,
  z: number
): { lat: number; lon: number } {
  const n = Math.PI - (2 * Math.PI * y) / (1 << z);
  const lat = (180 / Math.PI) * Math.atan(Math.sinh(n));
  const lon = (x / (1 << z)) * 360 - 180;
  return { lat, lon };
}

function pickLevel(levels: OziLevelDto[], z: number): OziLevelDto | null {
  if (levels.length === 0) return null;
  // Level 0 is highest resolution; higher zoom = level 0, lower zoom = deeper level
  const idx = Math.max(0, Math.min(levels.length - 1, levels.length - 1 - z));
  return levels[idx] ?? null;
}

function parseGeoreference(
  calibrationPoints: string[]
): AffineTransform | null {
  // Parse lines like: "Point01,xy,  100,  200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N"
  const points: Array<{ px: number; py: number; lat: number; lon: number }> =
    [];

  for (const line of calibrationPoints) {
    const fields = line.split(",").map((f) => f.trim());
    if (fields.length < 12) continue;

    const px = parseFloat(fields[2]);
    const py = parseFloat(fields[3]);

    const latDeg = parseFloat(fields[6]);
    const latMin = parseFloat(fields[7]);
    const latHemi = fields[8];
    const lonDeg = parseFloat(fields[9]);
    const lonMin = parseFloat(fields[10]);
    const lonHemi = fields[11];

    if ([px, py, latDeg, latMin, lonDeg, lonMin].some(isNaN)) continue;

    const lat = (latDeg + latMin / 60) * (latHemi === "S" ? -1 : 1);
    const lon = (lonDeg + lonMin / 60) * (lonHemi === "W" ? -1 : 1);

    points.push({ px, py, lat, lon });
  }

  if (points.length < 2) return null;

  const scaleX = linearScale(points.map((p) => [p.lon, p.px]));
  const scaleY = linearScale(points.map((p) => [p.lat, p.py]));
  if (!scaleX || !scaleY) return null;

  return {
    scaleX: scaleX.scale,
    offsetX: scaleX.offset,
    scaleY: scaleY.scale,
    offsetY: scaleY.offset,
  };
}

function linearScale(
  pairs: Array<[number, number]>
): { scale: number; offset: number } | null {
  const n = pairs.length;
  let sumX = 0,
    sumY = 0,
    sumXY = 0,
    sumXX = 0;
  for (const [x, y] of pairs) {
    sumX += x;
    sumY += y;
    sumXY += x * y;
    sumXX += x * x;
  }
  const denom = n * sumXX - sumX * sumX;
  if (Math.abs(denom) < 1e-12) return null;

  const scale = (n * sumXY - sumX * sumY) / denom;
  const offset = (sumY - scale * sumX) / n;
  return { scale, offset };
}
