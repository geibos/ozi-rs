/**
 * Holding a click on the map long enough to know it is not half of a double.
 *
 * Drawing a route finishes on a double-click, so a single click cannot be
 * committed the moment it arrives: the second half of a double-click would
 * already have put a point down. The application therefore held each click for
 * 220 ms before sending it.
 *
 * It held *one*. A second click inside that window cancelled the first and
 * replaced it — so a coordinator plotting a route at any normal pace, three or
 * four clicks a second, got roughly one point per burst and no sign that the
 * others had been dropped. The note in CJ-5 called it "быстрые клики теряются",
 * which reads as a delay; it was not a delay, the points were gone.
 *
 * Each click gets its own timer instead. Nothing cancels a click but a
 * double-click, which cancels every click still waiting — both halves of it are
 * inside the window by definition — and finishes the track.
 *
 * The commits are chained rather than fired in parallel: each one is told the
 * index to insert at, and two in flight at once would compute the same index.
 */

export interface PendingClicks<T> {
  /** Hold `payload`, committing it after the window unless a double-click comes. */
  hold(payload: T): void;
  /** Drop everything still waiting: a double-click, Esc, or leaving the mode. */
  cancel(): void;
  /** How many clicks are waiting — for a test, and for a readout. */
  waiting(): number;
}

export interface PendingClicksOptions<T> {
  /** How long a click waits before it counts as a single one. */
  windowMs?: number;
  /** Commit one click. Calls are chained, never overlapped. */
  commit: (payload: T) => Promise<void>;
  /** Injected in tests; `window.setTimeout` in the application. */
  setTimer?: (fn: () => void, ms: number) => number;
  clearTimer?: (id: number) => void;
}

/** The double-click window, in milliseconds. */
export const DOUBLE_CLICK_WINDOW_MS = 220;

export function pendingClicks<T>({
  windowMs = DOUBLE_CLICK_WINDOW_MS,
  commit,
  setTimer = (fn, ms) => window.setTimeout(fn, ms) as unknown as number,
  clearTimer = (id) => window.clearTimeout(id),
}: PendingClicksOptions<T>): PendingClicks<T> {
  let timers: number[] = [];
  let chain: Promise<void> = Promise.resolve();
  /**
   * Which draw these clicks belong to.
   *
   * Clearing the timers is not enough to cancel a click. A timer that has
   * already fired has left the list and put its commit on the chain, where it
   * waits behind whatever is running — and `cancel()` could not reach it. Two
   * ways that bit, both found by a reviewer on 2026-09-23:
   *
   * - A double-click that arrived while a commit was in flight still added the
   *   click before it.
   * - Leaving drawing mode between a click and its commit reordered the track:
   *   the mode's exit clears the preview, and the waiting commit then used the
   *   emptied preview's length as its insertion index, so a route drawn A, B
   *   and then abandoned mid-click came back as C, A, B. That is a changed
   *   route, not a late one.
   *
   * A commit checks the generation it was made in. Cancelling bumps it, and
   * everything from before is dropped wherever it had got to.
   */
  let generation = 0;

  return {
    hold(payload: T) {
      const madeIn = generation;
      const id = setTimer(() => {
        timers = timers.filter((other) => other !== id);
        if (madeIn !== generation) return;
        // Chained: the commit is told which index to insert at, and two in
        // flight would read the same one and write over each other.
        chain = chain
          .then(() => (madeIn === generation ? commit(payload) : undefined))
          .catch(() => {});
      }, windowMs);
      timers.push(id);
    },

    cancel() {
      generation += 1;
      for (const id of timers) clearTimer(id);
      timers = [];
    },

    waiting() {
      return timers.length;
    },
  };
}
