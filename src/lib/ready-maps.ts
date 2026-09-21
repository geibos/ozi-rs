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
 * (`8-Android&iOS/..._z16.sqlitedb`), which is why this matches by suffix.
 */
export function readyMapName(
  packageName: string,
  maps: readonly LizaMapPackageDto[],
): string | null {
  const match = maps.find(
    (map) => map.name.length > 0 && packageName.endsWith(map.name),
  );
  return match ? match.name : null;
}
