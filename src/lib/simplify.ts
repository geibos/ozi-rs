/**
 * The tolerance the simplify panel opens at, and the one way to open it.
 *
 * 2 m is OziExplorer's track filter at index 4 — the index the detachment's
 * standard prescribes for every foot patrol's track. Nobody could say what
 * the index meant; measured on eleven tracks from a real search on
 * 2026-10-06, Ozi keeps the points Douglas–Peucker keeps at about 2 m
 * (`docs/field-notes/2026-10-06-track-processing-in-ozi.md`). Only index 4 is
 * established; what other indices mean in metres is not.
 */
import { simplifyState } from "$lib/stores";

export const OZI_INDEX_4_TOLERANCE_M = 2;

/**
 * Open the panel for one track, at the index-4 tolerance.
 *
 * One function for the Tracks tab's menu and the inspector: the two used to
 * set the state themselves, and one of them opened a slider with no numbers
 * under it.
 */
export function openSimplify(layerId: bigint, trackId: bigint): void {
  simplifyState.set({
    active: true,
    layerId,
    trackId,
    toleranceM: OZI_INDEX_4_TOLERANCE_M,
    preview: null,
  });
}

/** Put the slider back at index 4; clearing the preview asks for a new one. */
export function resetToOziIndex4(): void {
  simplifyState.update((s) => ({
    ...s,
    toleranceM: OZI_INDEX_4_TOLERANCE_M,
    preview: null,
  }));
}
