/**
 * Recent-files localStorage helper for the Cmd-K command palette.
 *
 * Persists the last `MAX_RECENT_FILES` map opens under the versioned key
 * `ozi:recent-files:v1`. Pure UX convenience, per-machine — NOT part of the
 * Rust session file (see `project-persistence`).
 *
 * Records are most-recent-first. Adding an existing `mapPath` moves it to
 * the front rather than duplicating it. Failure modes (quota exceeded,
 * unparseable storage) are absorbed silently and logged via `console.warn`.
 */

export const RECENT_FILES_KEY = "ozi:recent-files:v1";
export const MAX_RECENT_FILES = 8;

export interface RecentFile {
  projectSlug: string;
  mapPath: string;
  mapName: string;
  /** Epoch milliseconds at the time `appendRecentFile` was called. */
  openedAt: number;
}

function isValidRecord(value: unknown): value is RecentFile {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.projectSlug === "string" &&
    typeof v.mapPath === "string" &&
    typeof v.mapName === "string" &&
    typeof v.openedAt === "number"
  );
}

/**
 * Read and validate the recent-files list from localStorage. Returns an
 * empty list on any failure (missing, corrupt, wrong shape, storage
 * unavailable). Never throws.
 */
export function getRecentFiles(): RecentFile[] {
  try {
    if (typeof localStorage === "undefined") return [];
    const raw = localStorage.getItem(RECENT_FILES_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      console.warn("getRecentFiles: stored value is not an array, ignoring");
      return [];
    }
    const valid = parsed.filter(isValidRecord);
    return valid.slice(0, MAX_RECENT_FILES);
  } catch (err) {
    console.warn("getRecentFiles: read/parse failed", err);
    return [];
  }
}

function writeOrTrim(list: RecentFile[]): void {
  try {
    localStorage.setItem(RECENT_FILES_KEY, JSON.stringify(list));
  } catch (err) {
    // Quota exceeded — drop the oldest record and retry once. If the retry
    // still throws, give up silently per spec.
    console.warn("appendRecentFile: storage write failed, retrying", err);
    if (list.length <= 1) return;
    const trimmed = list.slice(0, list.length - 1);
    try {
      localStorage.setItem(RECENT_FILES_KEY, JSON.stringify(trimmed));
    } catch (retryErr) {
      console.warn("appendRecentFile: retry failed, giving up", retryErr);
    }
  }
}

/**
 * Prepend `record` to the recent-files list. If a record with the same
 * `mapPath` already exists, it is removed first (move-to-front, not
 * duplicate). The list is capped at `MAX_RECENT_FILES`.
 */
export function appendRecentFile(record: RecentFile): void {
  if (typeof localStorage === "undefined") return;
  const current = getRecentFiles();
  const deduped = current.filter((r) => r.mapPath !== record.mapPath);
  const next = [record, ...deduped].slice(0, MAX_RECENT_FILES);
  writeOrTrim(next);
}

/**
 * Test/dev helper — wipe the recent-files key. Not called by production
 * code paths but useful for vitest and manual smoke flows.
 */
export function clearRecentFiles(): void {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.removeItem(RECENT_FILES_KEY);
  } catch {
    // ignore
  }
}
