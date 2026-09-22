/**
 * Whether the application may reach for the network on its own initiative.
 *
 * CJ-2 ("полевой старт: открыть карты без интернета") promises zero network
 * requests from a cold launch in the field. Two paths broke that promise: the
 * map added an OpenStreetMap raster source unconditionally, and the launch
 * sequence always started the LizaAlert catalogue walk. Offline both merely
 * fail — but they fail slowly, they burn battery, and on a phone hotspot they
 * spend the crew's data on a basemap nobody can see under the local raster.
 *
 * `navigator.onLine` is only worth trusting in one direction. `true` means
 * "there is an interface with a link", which on a field laptop plugged into a
 * dead hotspot is a lie; `false` means the machine has no way out at all, and
 * that is the case this guards. So the rule is deliberately one-sided: only a
 * definite `false` stops us. Anything we cannot determine behaves exactly as
 * it did before.
 *
 * This governs only what the app does by itself. Everything the operator asks
 * for — the refresh button, a download, opening a bundle — runs regardless:
 * they can see the link state better than `navigator` can.
 */
export function mayReachNetwork(online: boolean | undefined): boolean {
  return online !== false;
}

/** `mayReachNetwork` against the live browser state. */
export function mayReachNetworkNow(): boolean {
  return mayReachNetwork(
    typeof navigator === "undefined" ? undefined : navigator.onLine,
  );
}

/**
 * What the catalogue shows when the launch walk was skipped for want of a
 * link. Never rendered: `BundleLoader` and the status line translate any
 * non-null `catalogueError` into "showing the saved list from {when}". It is
 * here so the reason survives into a log.
 */
export const OFFLINE_CATALOGUE_REASON =
  "offline: the machine reports no network, the saved list is what there is";
