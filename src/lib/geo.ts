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

/**
 * The area of the polygon these points close, in square kilometres.
 *
 * OziExplorer's measuring tool is called "Distance & Area", and the area is
 * the half a coordinator actually writes down: a sector is handed to a crew as
 * "прочесать 2.4 км²", and the number decides how many people it takes and how
 * long it will run. Measuring it by eye off a scale bar is what the tool
 * exists to stop.
 *
 * The polygon is treated as closed whether or not the last point repeats the
 * first — a crew clicking round a sector stops when the shape is obvious, not
 * when it is arithmetically shut.
 *
 * Equirectangular projection about the polygon's own mean latitude, then the
 * shoelace formula. Over a search sector — kilometres, not hundreds of them —
 * this is within a fraction of a percent of the spherical answer, and unlike
 * the spherical excess formula it does not lose precision on the small
 * shapes that are the normal case. Fewer than three points enclose nothing.
 */
export function polygonAreaSqKm(points: LatLon[]): number {
  // A crew that clicked back to the start has not drawn a different sector.
  // The repeated point would otherwise be counted twice in the mean latitude
  // that sets the projection, and the same shape would measure differently
  // depending on where the clicking stopped.
  const ring =
    points.length > 1 &&
    points[0].lat === points[points.length - 1].lat &&
    points[0].lon === points[points.length - 1].lon
      ? points.slice(0, -1)
      : points;
  if (ring.length < 3) return 0;

  const meanLat = ring.reduce((sum, point) => sum + point.lat, 0) / ring.length;
  const cosLat = Math.cos((meanLat * Math.PI) / 180);
  const kmPerDegree = (Math.PI * EARTH_RADIUS_KM) / 180;

  let twiceArea = 0;
  for (let i = 0; i < ring.length; i += 1) {
    const a = ring[i];
    const b = ring[(i + 1) % ring.length];
    const ax = a.lon * cosLat * kmPerDegree;
    const ay = a.lat * kmPerDegree;
    const bx = b.lon * cosLat * kmPerDegree;
    const by = b.lat * kmPerDegree;
    twiceArea += ax * by - bx * ay;
  }
  return Math.abs(twiceArea) / 2;
}

/**
 * An area in the unit a coordinator would say it in.
 *
 * Hectares in the middle: a sector of 40 га is a sentence a Russian search
 * coordinator says, where "0.4 км²" is one they would have to convert.
 */
export function formatMeasuredArea(sqKm: number, locale: "ru" | "en"): string {
  if (sqKm <= 0) return "";
  if (sqKm < 0.01) {
    const sqMetres = Math.round(sqKm * 1_000_000);
    return locale === "ru" ? `${sqMetres} м²` : `${sqMetres} m²`;
  }
  if (sqKm < 1) {
    const hectares = sqKm * 100;
    const rounded = hectares.toFixed(hectares < 10 ? 1 : 0);
    return locale === "ru" ? `${rounded} га` : `${rounded} ha`;
  }
  const rounded = sqKm.toFixed(sqKm < 10 ? 2 : 1);
  return locale === "ru" ? `${rounded} км²` : `${rounded} km²`;
}
