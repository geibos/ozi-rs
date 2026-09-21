/**
 * The stand's transport is the only thing between a screen and nothing, so
 * the rules it enforces are worth testing: it answers from the fixtures, and
 * it refuses loudly when it cannot.
 */
import { beforeEach, describe, expect, it } from "vitest";
import { invoke, standCalls } from "./stand/tauri-core";
import { listen, standEmit } from "./stand/tauri-event";
import { appStateFixture, waypointsFixture } from "./fixtures";

beforeEach(() => {
  standCalls.length = 0;
});

describe("the stand's transport", () => {
  it("answers a query from the fixtures", async () => {
    await expect(invoke("get_app_state")).resolves.toEqual(appStateFixture);
    await expect(invoke("get_waypoints", { layerId: 1 })).resolves.toEqual(
      waypointsFixture,
    );
  });

  it("accepts a mutation without inventing data", async () => {
    await expect(
      invoke("set_all_tracks_visible", { visible: false }),
    ).resolves.toBeNull();
  });

  it("refuses a command it has no answer for, naming it", async () => {
    // The failure this guards: a screen that renders because a mock quietly
    // returned undefined, which is how a broken rail passed every test.
    await expect(invoke("no_such_command")).rejects.toThrow(/no_such_command/);
  });

  it("records what a screen asked for", async () => {
    await invoke("get_app_state");
    await invoke("get_waypoints", { layerId: 2 });

    expect(standCalls.map((c) => c.command)).toEqual([
      "get_app_state",
      "get_waypoints",
    ]);
    expect(standCalls[1].args).toEqual({ layerId: 2 });
  });
});

describe("the stand's events", () => {
  it("registers a listener and delivers a hand-sent event", async () => {
    const seen: unknown[] = [];
    const unlisten = await listen<string>("state-changed", (e) =>
      seen.push(e.payload),
    );

    standEmit("state-changed", "first");
    unlisten();
    standEmit("state-changed", "after unsubscribe");

    expect(seen).toEqual(["first"]);
  });
});
