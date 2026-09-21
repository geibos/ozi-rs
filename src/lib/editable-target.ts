/**
 * Whether a keyboard event landed in something the operator is typing into.
 *
 * A global chord must not hijack native text editing: Backspace while renaming
 * a track deletes a character, not the last measured point, and Cmd+Z there is
 * the input's own undo.
 *
 * Shared rather than copied — it lived in the layout, and the second caller
 * (the measuring tool's Backspace) would otherwise have had its own idea of
 * what "editable" means, which is how two handlers come to disagree about the
 * same keypress.
 */
export function isEditableTarget(target: EventTarget | null): boolean {
  const el = target instanceof HTMLElement ? target : null;
  return Boolean(
    el?.closest(
      'input, textarea, select, [contenteditable="true"], [contenteditable=""]',
    ),
  );
}
