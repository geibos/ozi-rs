/**
 * Capturing a moment, and asking about it afterwards.
 *
 * Two steps, and the order matters. The screen is photographed the instant the
 * operator asks — before a dialog can cover it, before the state moves on —
 * and the sentence about what happened is offered after, as an action on the
 * toast. A report with no sentence is still worth having; a sentence with no
 * screen is not.
 *
 * The owner asked for this on 2026-09-23, to hand a day's strangeness over as
 * one folder.
 */
import { get, writable } from "svelte/store";
import { toast } from "svelte-sonner";
import { addReportNote, revealPath, revealReports, saveReport } from "$lib/api";
import { t } from "$lib/i18n";

/**
 * The report waiting for its description, or `null`.
 *
 * A store rather than a dialog opened from here: the note box belongs in the
 * component tree, where Escape and focus work, and this only says which folder
 * it would be writing into.
 */
export const reportAwaitingNote = writable<string | null>(null);

/** Capture now; ask afterwards. */
export async function captureMoment(): Promise<void> {
  const translate = get(t);
  try {
    const report = await saveReport();
    toast.success(
      translate(
        report.screenshot_missing ? "report.savedNoShot" : "report.saved",
      ),
      {
        description: report.screenshot_missing
          ? translate("report.noShotWhy")
          : report.path,
        // Long enough to be caught by somebody who was looking at the map, not
        // at the corner where toasts appear.
        duration: 12_000,
        action: {
          label: translate("report.describe"),
          onClick: () => reportAwaitingNote.set(report.path),
        },
      },
    );
  } catch (error) {
    toast.error(translate("report.failed"), { description: String(error) });
  }
}

/** Write the description into the folder captured a moment ago. */
export async function describeMoment(
  path: string,
  note: string,
): Promise<void> {
  const translate = get(t);
  try {
    await addReportNote(path, note);
    toast.success(translate("report.saved"), {
      description: path,
      action: {
        label: translate("report.show"),
        onClick: () => void revealPath(path).catch(() => {}),
      },
    });
  } catch (error) {
    toast.error(translate("report.noteFailed"), { description: String(error) });
  } finally {
    reportAwaitingNote.set(null);
  }
}

/** Open the folder the reports go into. */
export async function showReportsFolder(): Promise<void> {
  try {
    await revealReports();
  } catch (error) {
    toast.error(get(t)("report.failed"), { description: String(error) });
  }
}
