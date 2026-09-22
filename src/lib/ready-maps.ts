import type { LizaMapPackageDto } from "./bindings";

/**
 * Whether a file that just landed during a bundle download is a map the crew
 * can open right now.
 *
 * A bundle is fetched whole — the 16 MiB topo layer arrives long before the
 * 185 MiB satellite one — and a map that has landed is openable while the rest
 * is still coming. The backend has always set the map's local path when its
 * file lands; nothing ever said so, so the crew waited for the whole bundle.
 *
 * `packageName` is the file's path relative to the bundle root
 * (`8-Android&iOS/..._z16.sqlitedb`), so the two meet at the last component,
 * compared whole. Comparing with `endsWith` over the entire path instead made
 * `bigmap.ozf2` a match for `map.ozf2`, and the crew was told a map was ready
 * that had not been downloaded. The Rust side had the same defect, found by
 * external review on 2026-09-22; this copy was missed because the review read
 * only the Rust.
 */
export function readyMapName(
  packageName: string,
  maps: readonly LizaMapPackageDto[],
): string | null {
  const landed = packageName.split("/").pop() ?? packageName;
  const match = maps.find((map) => map.name.length > 0 && landed === map.name);
  return match ? match.name : null;
}
