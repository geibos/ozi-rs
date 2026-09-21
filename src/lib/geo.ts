/**
 * Distance on the globe, for the on-map measuring tool.
 *
 * The same haversine the Rust side uses for track statistics
 * (`domain/track.rs::haversine_km`), with the same earth radius, so a measured
 * leg and a track's length are the same number for the same two points. A
 * measurement that disagreed with the track beside it would be worse than no
 * measurement.
 *
 * Computed here rather than over IPC: a scratch measurement changes on every
 * click and every drag, and a round trip per click is what makes a tool feel
 * slow.
 */
const EARTH_RADIUS_KM = 6371;

export interface LatLon {
  lat: number;
  lon: number;
}

/** Great-circle distance between two points, in kilometres. */
export function distanceKm(a: LatLon, b: LatLon): number {
  const dLat = toRadians(b.lat - a.lat);
  const dLon = toRadians(b.lon - a.lon);
  const h =
    Math.sin(dLat / 2) ** 2 +
    Math.cos(toRadians(a.lat)) *
      Math.cos(toRadians(b.lat)) *
      Math.sin(dLon / 2) ** 2;
  return 2 * EARTH_RADIUS_KM * Math.asin(Math.sqrt(h));
}

/** The total length of a path through the given points, in kilometres. */
export function pathLengthKm(points: LatLon[]): number {
  let total = 0;
  for (let i = 1; i < points.length; i += 1) {
    total += distanceKm(points[i - 1], points[i]);
  }
  return total;
}

function toRadians(degrees: number): number {
  return (degrees * Math.PI) / 180;
}

/**
 * A measured distance, as a person reads it.
 *
 * Metres under a kilometre: a crew measuring the width of a clearing wants
 * "180 м", not "0.2 км", and the difference between 40 and 140 metres is the
 * difference between two sides of a road.
 */
export function formatMeasuredDistance(
  km: number,
  locale: "ru" | "en",
): string {
  if (km < 1) {
    const metres = Math.round(km * 1000);
    return locale === "ru" ? `${metres} м` : `${metres} m`;
  }
  const rounded = km.toFixed(km < 10 ? 2 : 1);
  return locale === "ru" ? `${rounded} км` : `${rounded} km`;
}

/**
 * The point `distanceKm` away from `from` on the given bearing.
 *
 * Used to build a radius ring. A ring drawn as a flat circle in screen pixels
 * is wrong everywhere except the equator and gets worse the further north the
 * search is — at 60°, which is where these searches happen, a "500 m" circle
 * drawn flat is half a kilometre north-south and a kilometre east-west. The
 * crew standing in it would be looking in the wrong place.
 */
export function destinationPoint(
  from: LatLon,
  bearingDegrees: number,
  km: number,
): LatLon {
  const angular = km / EARTH_RADIUS_KM;
  const bearing = toRadians(bearingDegrees);
  const lat1 = toRadians(from.lat);
  const lon1 = toRadians(from.lon);

  const lat2 = Math.asin(
    Math.sin(lat1) * Math.cos(angular) +
      Math.cos(lat1) * Math.sin(angular) * Math.cos(bearing),
  );
  const lon2 =
    lon1 +
    Math.atan2(
      Math.sin(bearing) * Math.sin(angular) * Math.cos(lat1),
      Math.cos(angular) - Math.sin(lat1) * Math.sin(lat2),
    );

  return {
    lat: toDegrees(lat2),
    // Back into −180..180, so a ring drawn across the antimeridian does not
    // come out as a band around the world.
    lon: ((toDegrees(lon2) + 540) % 360) - 180,
  };
}

/**
 * A ring of `km` around `centre`, as a closed path.
 *
 * `steps` points, evenly spaced by bearing; 64 is smooth at any zoom a crew
 * uses and cheap enough to rebuild on every drag.
 */
export function ringAround(centre: LatLon, km: number, steps = 64): LatLon[] {
  if (km <= 0) return [];
  const ring: LatLon[] = [];
  for (let i = 0; i < steps; i += 1) {
    ring.push(destinationPoint(centre, (360 * i) / steps, km));
  }
  ring.push(ring[0]);
  return ring;
}

function toDegrees(radians: number): number {
  return (radians * 180) / Math.PI;
}
