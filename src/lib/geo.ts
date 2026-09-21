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
