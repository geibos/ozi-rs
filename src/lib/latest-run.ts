/**
 * "Only the newest run may write."
 *
 * Reloads that overlap have been wrong in four places now: the bundle preview,
 * both library tabs and the map's marker refresh. Each reloads on
 * `state-changed`, which during a bundle download fires once per file, so a
 * reload routinely overlaps itself — and whichever request answered last won,
 * even when it was the older one. Lists went backwards; a waypoint just added
 * could disappear.
 *
 * The rule is the same every time and small enough to get wrong differently
 * each time, so it lives here: take a token before the first await, and check
 * it before writing anything, on the failure path as well as the success one.
 *
 *     const runs = createLatestRun();
 *     ...
 *     const run = runs.begin();
 *     const rows = await fetchRows();
 *     if (!runs.isCurrent(run)) return;
 *     apply(rows);
 */
export interface LatestRun {
  /** Start a run and take its token. Any earlier run is now stale. */
  begin(): number;
  /** Whether this token is still the newest — check before writing. */
  isCurrent(token: number): boolean;
}

export function createLatestRun(): LatestRun {
  let newest = 0;
  return {
    begin(): number {
      newest += 1;
      return newest;
    },
    isCurrent(token: number): boolean {
      return token === newest;
    },
  };
}
