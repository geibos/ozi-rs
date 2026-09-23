/**
 * One implementation of "take these files into the project".
 *
 * Two surfaces ask for it — the Import… picker in the Tracks tab and, since
 * 2026-09-23, dropping files on the window — and CJ-3 is the same journey
 * either way: a day's recordings arrive as a handful of files from several
 * navigators. Duplicating the dispatch would let the two drift, which is how
 * the picker ended up accepting `.wpt` while nothing else did.
 *
 * Routing is by extension because that is what the operator has: a file from
 * a Garmin, a file from an old navigator, a ZIP from someone who zipped the
 * folder, and the `.wpt` the штаб next door exports. A directory dropped on
 * the window goes through the recursive folder import instead.
 */
import {
  importGpx,
  importPlt,
  importTracksDirectory,
  importWpt,
} from "$lib/api";

/** Extensions the file surfaces accept, lower-case and without the dot. */
export const IMPORTABLE_EXTENSIONS = ["gpx", "plt", "wpt", "zip"] as const;

export interface ImportFailure {
  /** The file's own name, which is what a toast has room for. */
  name: string;
  /** What the backend said, as it said it. */
  reason: string;
}

export interface ImportOutcome {
  imported: number;
  /**
   * The files that would not come in, each with what the backend said.
   *
   * The name alone was not enough. A coordinator told "20260708_Ветер2.plt did
   * not import" cannot tell whether to go back to the crew for another export
   * or whether the application is broken, and at four in the morning they will
   * assume the second. The reason is one line and it decides what they do
   * next.
   */
  failed: ImportFailure[];
  /** True when at least one path was a folder, so the summary can say so. */
  usedFolderImport: boolean;
}

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

/** Whether this path is one the import surfaces will take. */
export function isImportablePath(path: string): boolean {
  const lower = path.toLowerCase();
  return IMPORTABLE_EXTENSIONS.some((ext) => lower.endsWith(`.${ext}`));
}

/**
 * The failures as a coordinator reads them: name, then reason.
 *
 * Three at most. A folder where everything failed is a folder with the wrong
 * files in it, and twenty lines of the same message says that no better than
 * three do.
 */
export function describeFailures(failures: readonly ImportFailure[]): string {
  const shown = failures
    .slice(0, 3)
    .map(({ name, reason }) => `${name} — ${reason}`);
  if (failures.length > 3) shown.push(`…и ещё ${failures.length - 3}`);
  return shown.join("\n");
}

/**
 * Import each path, never stopping at the first failure.
 *
 * A folder of a day's recordings where one navigator's file is unreadable
 * still has the rest of the day in it, and a crew at four in the morning
 * needs the nine files that worked more than it needs to know which one did
 * not — so failures are collected and reported after the loop, not thrown.
 */
export async function importPaths(paths: string[]): Promise<ImportOutcome> {
  const outcome: ImportOutcome = {
    imported: 0,
    failed: [],
    usedFolderImport: false,
  };

  for (const path of paths) {
    try {
      const lower = path.toLowerCase();
      if (lower.endsWith(".plt")) {
        await importPlt(path);
      } else if (lower.endsWith(".wpt")) {
        // OziExplorer's own waypoint format, from the штаб next door.
        await importWpt(path);
      } else if (lower.endsWith(".gpx") || lower.endsWith(".zip")) {
        // ZIP goes through the GPX command, which unpacks it.
        await importGpx(path);
      } else {
        // Anything else that was dropped is treated as a folder: the recursive
        // import walks per-date subfolders, which is how a day arrives when
        // somebody hands over a memory card rather than files.
        await importTracksDirectory(path);
        outcome.usedFolderImport = true;
      }
      outcome.imported += 1;
    } catch (error) {
      console.error("Failed to import", path, error);
      outcome.failed.push({ name: baseName(path), reason: String(error) });
    }
  }

  return outcome;
}
