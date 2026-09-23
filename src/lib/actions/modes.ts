/**
 * What the map does with a click, in one place.
 *
 * The workspace has four modes — look, draw, edit, measure — and until
 * 2026-09-23 the strip of chips that names them at the top of the map was
 * inert. Not disabled-looking: four buttons marked `aria-disabled`, carrying
 * the four mode names, doing nothing at all when pressed. Every one of those
 * modes existed and worked; each was reachable from somewhere else — drawing
 * from the Tracks tab, editing from the segments table, measuring from the
 * command palette — and the control that says what mode you are in was a
 * placeholder from a redesign that never came back to it.
 *
 * The chips are not the problem this file solves, though. Entering drawing
 * mode is not a flag: a track has to be created in the active layer first, its
 * first segment found, the counters reset. That routine lived inside the
 * Tracks tab, so wiring the chips would have meant a second copy of it —
 * which is how the import surfaces came to accept different file types from
 * each other, and how this repository learned to put the dispatch in one
 * place.
 *
 * So the mode is the function, and every surface asks for a mode.
 */
import { derived, get, type Readable } from "svelte/store";
import { toast } from "svelte-sonner";
import {
  activeTrackLayerId,
  addWaypointMode,
  drawingModeActive,
  drawingPointCount,
  drawingSegmentId,
  drawingTrackId,
  drawingTrackLayerId,
  editModeActive,
  measuringActive,
  setMeasuring,
} from "$lib/stores";
import { createEmptyTrack, getTrackDetail } from "$lib/api";
import { t } from "$lib/i18n";

export type InteractionMode = "view" | "draw" | "edit" | "measure";

/**
 * Which mode the workspace is in, read from the flags that have always held
 * it.
 *
 * The flags stay because a dozen components read them; this is the single
 * reading of them, so a screen that shows the mode and a screen that sets it
 * cannot disagree about what "edit" means.
 */
export const workspaceMode: Readable<InteractionMode> = derived(
  [drawingModeActive, editModeActive, measuringActive],
  ([drawing, editing, measuring]): InteractionMode => {
    if (drawing) return "draw";
    if (editing) return "edit";
    if (measuring) return "measure";
    return "view";
  },
);

/** The same reading, for code that is not in a component. */
export function currentMode(): InteractionMode {
  return get(workspaceMode);
}

/** Leave every mode. Called before entering one, and by "view". */
function leaveAll(): void {
  if (get(drawingModeActive)) drawingModeActive.set(false);
  if (get(editModeActive)) editModeActive.set(false);
  if (get(measuringActive)) setMeasuring(false);
  if (get(addWaypointMode)) addWaypointMode.set(false);
}

/**
 * Enter a mode, or leave it when it is already the current one.
 *
 * Answers the mode the workspace ended up in, which is not always the one
 * asked for: drawing needs a track layer to draw into, and saying so is better
 * than a chip that lights up over a map that ignores the clicks.
 */
export async function setInteractionMode(
  mode: InteractionMode,
): Promise<InteractionMode> {
  if (mode !== "view" && currentMode() === mode) {
    leaveAll();
    return "view";
  }

  leaveAll();
  if (mode === "view") return "view";

  if (mode === "edit") {
    editModeActive.set(true);
    return "edit";
  }

  if (mode === "measure") {
    // Distance first: it is the commoner question by a wide margin — how far
    // a crew still has to walk — and the area tool is one click away.
    setMeasuring(true, "distance");
    return "measure";
  }

  const layerId = get(activeTrackLayerId);
  if (layerId === null) {
    toast.error(get(t)("mode.drawNeedsLayer"));
    return "view";
  }

  try {
    const trackId = await createEmptyTrack(layerId, "New Track");
    const detail = await getTrackDetail(layerId, trackId);
    drawingTrackLayerId.set(layerId);
    drawingTrackId.set(trackId);
    drawingSegmentId.set(BigInt(detail.segments[0].id));
    drawingPointCount.set(0);
    drawingModeActive.set(true);
    return "draw";
  } catch (error) {
    // The scratch track may or may not have been created; either way the mode
    // must not come on over a map that has nothing to draw into.
    console.error("Failed to start track drawing mode", error);
    toast.error(get(t)("tracksTab.drawStartFailed"), {
      description: String(error),
    });
    return "view";
  }
}
