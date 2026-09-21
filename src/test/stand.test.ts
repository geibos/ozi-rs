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

describe("the stand respects the arguments it is given", () => {
  it("answers waypoints for the layer that has them, and no other", async () => {
    // A handler that ignores its arguments is a mock that lies: answering
    // every layer with the same waypoints listed each of them twice in the
    // rail, which read as an app defect.
    await expect(invoke("get_waypoints", { layerId: 1 })).resolves.toEqual(
      waypointsFixture,
    );
    await expect(invoke("get_waypoints", { layerId: 2 })).resolves.toEqual([]);
  });

  it("answers the track detail only for the track it belongs to", async () => {
    const other = (await invoke("get_track_detail", {
      layerId: 1,
      trackId: 99,
    })) as { segments: unknown[] };
    expect(other.segments).toEqual([]);
  });
});
