import { describe, expect, it } from "vitest";
import { DOUBLE_CLICK_WINDOW_MS, pendingClicks } from "../lib/drawing-clicks";

/**
 * A fake clock, so a test about timing does not take a second per click.
 */
function fakeClock() {
  let now = 0;
  let nextId = 1;
  const timers = new Map<number, { at: number; fn: () => void }>();
  return {
    setTimer(fn: () => void, ms: number) {
      const id = nextId++;
      timers.set(id, { at: now + ms, fn });
      return id;
    },
    clearTimer(id: number) {
      timers.delete(id);
    },
    advance(ms: number) {
      now += ms;
      for (const [id, timer] of [...timers].sort((a, b) => a[1].at - b[1].at)) {
        if (timer.at <= now) {
          timers.delete(id);
          timer.fn();
        }
      }
    },
  };
}

describe("holding a click on the map", () => {
  it("keeps every click of a fast burst", async () => {
    // A coordinator plotting a route clicks three or four times a second. Each
    // click used to cancel the one before it, so this burst produced one point
    // out of five and said nothing about the other four.
    const clock = fakeClock();
    const committed: number[] = [];
    const clicks = pendingClicks<number>({
      commit: async (n) => {
        committed.push(n);
      },
      setTimer: clock.setTimer,
      clearTimer: clock.clearTimer,
    });

    for (let i = 1; i <= 5; i += 1) {
      clicks.hold(i);
      clock.advance(150); // faster than the double-click window
    }
    clock.advance(DOUBLE_CLICK_WINDOW_MS);
    await new Promise((resolve) => setTimeout(resolve, 10));

    expect(committed).toEqual([1, 2, 3, 4, 5]);
  });

  it("commits in the order the clicks were made", async () => {
    const clock = fakeClock();
    const committed: string[] = [];
    const clicks = pendingClicks<string>({
      commit: async (name) => {
        // A slow first commit must not let the second overtake it: each is
        // told the index to insert at, and two at once read the same one.
        await new Promise((resolve) =>
          setTimeout(resolve, name === "a" ? 5 : 0),
        );
        committed.push(name);
      },
      setTimer: clock.setTimer,
      clearTimer: clock.clearTimer,
    });

    clicks.hold("a");
    clicks.hold("b");
    clock.advance(DOUBLE_CLICK_WINDOW_MS);
    await new Promise((resolve) => setTimeout(resolve, 30));

    expect(committed).toEqual(["a", "b"]);
  });

  it("commits nothing for a double click", () => {
    // A double-click is two clicks and then `dblclick`, all inside the window.
    // The track finishes; neither half becomes a point.
    const clock = fakeClock();
    const committed: number[] = [];
    const clicks = pendingClicks<number>({
      commit: async (n) => {
        committed.push(n);
      },
      setTimer: clock.setTimer,
      clearTimer: clock.clearTimer,
    });

    clicks.hold(1);
    clock.advance(40);
    clicks.hold(2);
    clock.advance(20);
    expect(clicks.waiting()).toBe(2);

    clicks.cancel(); // what the dblclick handler does
    clock.advance(DOUBLE_CLICK_WINDOW_MS * 2);

    expect(committed).toEqual([]);
    expect(clicks.waiting()).toBe(0);
  });

  it("holds a lone click for the whole window before committing it", async () => {
    const clock = fakeClock();
    const committed: number[] = [];
    const clicks = pendingClicks<number>({
      commit: async (n) => {
        committed.push(n);
      },
      setTimer: clock.setTimer,
      clearTimer: clock.clearTimer,
    });

    clicks.hold(7);
    clock.advance(DOUBLE_CLICK_WINDOW_MS - 1);
    expect(committed).toEqual([]);

    clock.advance(1);
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(committed).toEqual([7]);
  });

  it("survives a commit that fails without stopping the ones after it", async () => {
    // One point that the backend refuses must not silence the rest of the
    // route: the operator is still drawing.
    const clock = fakeClock();
    const committed: number[] = [];
    const clicks = pendingClicks<number>({
      commit: async (n) => {
        if (n === 2) throw new Error("refused");
        committed.push(n);
      },
      setTimer: clock.setTimer,
      clearTimer: clock.clearTimer,
    });

    clicks.hold(1);
    clicks.hold(2);
    clicks.hold(3);
    clock.advance(DOUBLE_CLICK_WINDOW_MS);
    await new Promise((resolve) => setTimeout(resolve, 10));

    expect(committed).toEqual([1, 3]);
  });
});
