/**
 * Where a recording is suspect: jumps and outliers, the two things an
 * operator hunts for when cleaning a track.
 *
 * A **jump** is a leg the GPS did not walk — the recording stopped, and the
 * line joins the gap straight. An **outlier** is a point nobody stood at: the
 * track goes out to it and comes straight back. The detachment's wiki finds
 * both in OziExplorer's point list by the Dist and KPH columns, and says to
 * delete only the outlier's apex — deleting a neighbour leaves the spike and
 * bends the track.
 *
 * The thresholds come from the owner's own cleaning of eleven tracks from a
 * real search (2026-10-06, `docs/field-notes/2026-10-06-track-processing-in-ozi.md`):
 * they put 27 of his 32 breaks and 15 of his 20 deleted points on the list, at
 * about eleven entries a track. The list is for review — nothing here edits a
 * track.
 */
import { distanceKm } from "./geo";
import type { Locale } from "./i18n";

export interface Leg {
  distanceM: number;
  /** Seconds between the two fixes, when both carry a time. */
  seconds: number | null;
  /** `null` without times, or when the clock did not move. */
  speedKmh: number | null;
}

interface PointLike {
  id: number;
  lat: number;
  lon: number;
  timestamp: string | null;
}

interface SegmentLike {
  id: number;
  points: PointLike[];
}

export type SuspectKind = "jump" | "outlier";

export interface Suspect {
  kind: SuspectKind;
  segmentId: number;
  /** A jump's point is the one after the gap; an outlier's is its apex. */
  pointId: number;
  /** The point before — where a jump is split. */
  previousPointId: number;
  lat: number;
  lon: number;
  before: Leg;
  after: Leg | null;
}

export interface SuspectThresholds {
  /** A jump is at least this long… */
  jumpMinM: number;
  /** …and at least this many of the track's median legs. */
  jumpMedians: number;
  /** Each leg of an outlier is at least this long… */
  outlierMinM: number;
  /** …and at least this many median legs… */
  outlierMedians: number;
  /**
   * …and its neighbours are closer together than this share of the two legs:
   * the track went out and came back.
   */
  outlierReturnRatio: number;
}

export const DEFAULT_SUSPECT_THRESHOLDS: SuspectThresholds = {
  jumpMinM: 100,
  jumpMedians: 10,
  outlierMinM: 30,
  outlierMedians: 3,
  outlierReturnRatio: 0.35,
};

function metres(a: PointLike, b: PointLike): number {
  return distanceKm(a, b) * 1000;
}

function leg(from: PointLike, to: PointLike): Leg {
  const distanceM = metres(from, to);
  const t0 = from.timestamp === null ? NaN : Date.parse(from.timestamp);
  const t1 = to.timestamp === null ? NaN : Date.parse(to.timestamp);
  const seconds =
    Number.isFinite(t0) && Number.isFinite(t1) ? (t1 - t0) / 1000 : null;
  const speedKmh =
    seconds !== null && seconds > 0 ? (distanceM / seconds) * 3.6 : null;
  return { distanceM, seconds, speedKmh };
}

/**
 * The leg into each point from the one before it; `null` for the first. Per
 * segment: the line between two segments is a gap, not a leg.
 */
export function legsOf(points: PointLike[]): Array<Leg | null> {
  return points.map((p, i) => (i === 0 ? null : leg(points[i - 1], p)));
}

function median(values: number[]): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? sorted[mid]
    : (sorted[mid - 1] + sorted[mid]) / 2;
}

/** The track's jumps and outliers, in track order. */
export function findSuspects(
  segments: SegmentLike[],
  thresholds: SuspectThresholds = DEFAULT_SUSPECT_THRESHOLDS,
): Suspect[] {
  const legsBySegment = segments.map((s) => legsOf(s.points));
  const allDistances = legsBySegment.flatMap((legs) =>
    legs.flatMap((l) => (l ? [l.distanceM] : [])),
  );
  if (allDistances.length === 0) return [];
  const step = median(allDistances);
  const jumpM = Math.max(thresholds.jumpMinM, thresholds.jumpMedians * step);
  const outlierM = Math.max(
    thresholds.outlierMinM,
    thresholds.outlierMedians * step,
  );

  const suspects: Suspect[] = [];
  segments.forEach((segment, s) => {
    const points = segment.points;
    const legs = legsBySegment[s];
    const isOutlier = (i: number): boolean => {
      if (i <= 0 || i >= points.length - 1) return false;
      const into = legs[i]!.distanceM;
      const out = legs[i + 1]!.distanceM;
      return (
        into >= outlierM &&
        out >= outlierM &&
        metres(points[i - 1], points[i + 1]) <
          thresholds.outlierReturnRatio * (into + out)
      );
    };
    for (let i = 1; i < points.length; i += 1) {
      const before = legs[i]!;
      const after = legs[i + 1] ?? null;
      const base = {
        segmentId: segment.id,
        pointId: points[i].id,
        previousPointId: points[i - 1].id,
        lat: points[i].lat,
        lon: points[i].lon,
        before,
        after,
      };
      if (isOutlier(i)) {
        suspects.push({ kind: "outlier", ...base });
      } else if (
        before.distanceM >= jumpM &&
        // A spike's two legs are long too; the apex already stands for them.
        !isOutlier(i - 1)
      ) {
        suspects.push({ kind: "jump", ...base });
      }
    }
  });
  return suspects;
}

const LEG_UNITS: Record<Locale, { m: string; km: string; kmh: string }> = {
  ru: { m: "м", km: "км", kmh: "км/ч" },
  en: { m: "m", km: "km", kmh: "km/h" },
};

/**
 * `351 м · 63 км/ч` — the Dist and KPH columns of OziExplorer's point list,
 * which is where the wiki tells an operator to look for an outlier.
 */
export function formatLeg(leg: Leg, locale: Locale): string {
  const u = LEG_UNITS[locale] ?? LEG_UNITS.en;
  const distance =
    leg.distanceM < 1000
      ? `${Math.round(leg.distanceM)}\u00a0${u.m}`
      : `${(leg.distanceM / 1000).toFixed(1)}\u00a0${u.km}`;
  return leg.speedKmh === null
    ? distance
    : `${distance} · ${Math.round(leg.speedKmh)}\u00a0${u.kmh}`;
}
