/**
 * What the operator is told after a file has been written.
 *
 * CJ-6 ends with a file handed to a group, and until 2026-09-23 four of the
 * export paths said nothing at all on success: the file appeared somewhere the
 * operator had chosen a moment earlier, and the interface gave no sign it had
 * happened. An export that says nothing is indistinguishable from one that did
 * not run — at four in the morning, sending a crew out, that is a question
 * nobody can answer without going to look.
 *
 * The day export had a success toast from the start. The single-file exports,
 * which are the ones used to hand one crew's track over, did not.
 *
 * One function so the two cannot drift again, and so the "show it to me"
 * action is attached everywhere rather than in whichever place someone
 * remembered.
 */
import { toast } from "svelte-sonner";
import { get } from "svelte/store";
import { revealPath } from "$lib/api";
import { t } from "$lib/i18n";

/** The file's name, without the directory nobody reads in a toast. */
export function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

/**
 * Say that a file was written, and offer to show it.
 *
 * The toast names the file, not the path: a path in a corner is not something
 * anybody retypes, which is what the action is for.
 */
export function reportExported(path: string): void {
  const translate = get(t);
  toast.success(translate("export.done").replace("{file}", baseName(path)), {
    description: path,
    action: {
      label: translate("export.reveal"),
      onClick: () => {
        // A failure here is worth a word: the operator asked for a window and
        // did not get one, and the reason is usually that the file was moved
        // between writing it and asking.
        void revealPath(path).catch((error) => {
          toast.error(translate("export.revealFailed"), {
            description: String(error),
          });
        });
      },
    },
  });
}
