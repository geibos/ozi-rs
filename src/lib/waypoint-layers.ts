import type { LayerSummaryDto } from "./types";

/**
 * Pure selector: given a list of waypoint layer summaries, returns those whose
 * per-layer `visible` flag is not explicitly `false`. Layers without the flag
 * default to visible (current backend does not emit the flag). Lives in a
 * dependency-free module so tests can import it without dragging in browser
 * globals such as `localStorage`.
 */
export function selectVisibleWaypointLayers(
  layers: LayerSummaryDto[] | undefined | null,
): LayerSummaryDto[] {
  if (!layers) return [];
  return layers.filter((layer) => layer.visible !== false);
}
