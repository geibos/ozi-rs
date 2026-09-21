import { get } from "svelte/store";
import { toast } from "svelte-sonner";
import { t, type MessageKey } from "./i18n";

/**
 * Tell the operator that an edit on the map did not happen.
 *
 * Dragging a track point, deleting one, inserting one and dragging a waypoint
 * all reported their failures to `console.error` and nowhere else, and the
 * error reporter is disabled outside dev builds. In a release build a refused
 * edit was therefore entirely silent: the operator dragged a point, the
 * backend declined, and the only sign was that the next reload put it back.
 *
 * The loader was given this treatment in `honest-bundle-flow`; the map's
 * editing path never was. The backend's own message goes in the description,
 * verbatim, because it is the only thing that says *why*.
 */
export function reportEditFailure(key: MessageKey, error: unknown): void {
  console.error(key, error);
  toast.error(get(t)(key), { description: String(error) });
}
