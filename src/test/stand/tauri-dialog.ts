/**
 * The stand's replacement for `@tauri-apps/plugin-dialog`.
 *
 * File pickers cannot open in a browser. Every dialog answers "cancelled" by
 * default, which is the branch a screen must handle anyway, and the call is
 * recorded so a stand session can show what would have been asked.
 *
 * Cancelling every time also meant that no flow behind a file dialog — import,
 * export, save-as, open a project — could be looked at on the stand at all,
 * which is a large hole in a stand whose purpose is to be looked at. So the
 * answer can be set: `standAnswerDialogsWith("/tmp/day.gpx")` makes the next
 * pickers answer that path, and `standCancelDialogs()` puts it back.
 */
export const standDialogCalls: Array<{ kind: string; options: unknown }> = [];

let plannedAnswer: string | null = null;
let plannedConfirm = false;

/** Answer the file pickers with this path instead of cancelling. */
export function standAnswerDialogsWith(
  path: string,
  confirmAnswer = true,
): void {
  plannedAnswer = path;
  plannedConfirm = confirmAnswer;
}

/** Back to cancelling, which is the default. */
export function standCancelDialogs(): void {
  plannedAnswer = null;
  plannedConfirm = false;
}

export async function open(options?: unknown): Promise<string | null> {
  standDialogCalls.push({ kind: "open", options });
  return plannedAnswer;
}

export async function save(options?: unknown): Promise<string | null> {
  standDialogCalls.push({ kind: "save", options });
  return plannedAnswer;
}

export async function confirm(
  message: string,
  options?: unknown,
): Promise<boolean> {
  standDialogCalls.push({ kind: "confirm", options: { message, options } });
  return plannedConfirm;
}

export async function message(text: string, options?: unknown): Promise<void> {
  standDialogCalls.push({ kind: "message", options: { text, options } });
}
