/**
 * The projects a crew opened or saved recently, so reopening yesterday's work
 * is not a hunt through a file dialog.
 *
 * ADR-0020 declares a recent-projects list in scope; what existed was
 * `recentFiles.ts`, which records recent *maps* — a bundle and one of its
 * tiles, not a saved `.ozp`. The two lists answer different questions and both
 * belong in the palette.
 *
 * Deliberately not merged with `recentFiles.ts`: the records differ, and the
 * shared part is forty lines of localStorage plumbing around a working feature
 * with its own tests. If a third list ever appears, extract then.
 *
 * Per-machine convenience, like the other one, and explicitly not part of the
 * Rust session file: the session remembers one project, this remembers where
 * the others are.
 */
export const RECENT_PROJECTS_KEY = "ozi:recent-projects:v1";
export const MAX_RECENT_PROJECTS = 8;

export interface RecentProject {
  /** Absolute path to the `.ozp` file. */
  path: string;
  /** What to show: the file's own name, since a project carries no title. */
  name: string;
  /** Epoch milliseconds when it was last opened or saved. */
  openedAt: number;
}

function isValidRecord(value: unknown): value is RecentProject {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.path === "string" &&
    v.path !== "" &&
    typeof v.name === "string" &&
    typeof v.openedAt === "number"
  );
}

/** The file's own name, which is all a project has to be called. */
export function projectDisplayName(path: string): string {
  const last = path.split(/[/\\]/).pop() ?? path;
  return last === "" ? path : last;
}

/**
 * The list, most recent first. Returns empty on any failure — missing,
 * corrupt, wrong shape, storage unavailable — and never throws: a broken
 * convenience must not stop the palette opening.
 */
export function getRecentProjects(): RecentProject[] {
  try {
    if (typeof localStorage === "undefined") return [];
    const raw = localStorage.getItem(RECENT_PROJECTS_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isValidRecord).slice(0, MAX_RECENT_PROJECTS);
  } catch {
    return [];
  }
}

/**
 * Record a project path, moving it to the front if it is already known.
 *
 * Called on open and on save alike: a project a crew has just written is the
 * one they are most likely to want next, and a save is where a never-saved
 * project first gets a path at all.
 */
export function rememberProject(path: string, now = Date.now()): void {
  if (path === "") return;
  try {
    if (typeof localStorage === "undefined") return;
    const existing = getRecentProjects().filter((p) => p.path !== path);
    const next: RecentProject[] = [
      { path, name: projectDisplayName(path), openedAt: now },
      ...existing,
    ].slice(0, MAX_RECENT_PROJECTS);
    localStorage.setItem(RECENT_PROJECTS_KEY, JSON.stringify(next));
  } catch {
    // A convenience list is not worth a toast, and the quota it could exceed
    // is shared with the catalogue cache, which matters more.
  }
}

/** Forget one project — for a path that no longer opens. */
export function forgetProject(path: string): void {
  try {
    if (typeof localStorage === "undefined") return;
    const next = getRecentProjects().filter((p) => p.path !== path);
    localStorage.setItem(RECENT_PROJECTS_KEY, JSON.stringify(next));
  } catch {
    // As above.
  }
}
