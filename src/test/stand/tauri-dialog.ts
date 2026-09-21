/**
 * The stand's replacement for `@tauri-apps/plugin-dialog`.
 *
 * File pickers cannot open in a browser. Every dialog answers "cancelled",
 * which is the branch a screen must handle anyway, and the call is recorded so
 * a stand session can show what would have been asked.
 */
export const standDialogCalls: Array<{ kind: string; options: unknown }> = [];

export async function open(options?: unknown): Promise<string | null> {
  standDialogCalls.push({ kind: "open", options });
  return null;
}

export async function save(options?: unknown): Promise<string | null> {
  standDialogCalls.push({ kind: "save", options });
  return null;
}

export async function confirm(
  message: string,
  options?: unknown,
): Promise<boolean> {
  standDialogCalls.push({ kind: "confirm", options: { message, options } });
  return false;
}

export async function message(text: string, options?: unknown): Promise<void> {
  standDialogCalls.push({ kind: "message", options: { text, options } });
}
