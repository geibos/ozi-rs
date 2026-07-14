/**
 * Tests for the bigint → number IPC argument serializer in `invokeIpc`
 * (hotfix until tauri-specta lands in Milestone 1).
 *
 * Tauri 2 serializes `invoke` args with `JSON.stringify`, which throws
 * `TypeError: Do not know how to serialize a BigInt` — so any bigint ID
 * passed through `invokeIpc` unmodified rejects before reaching Rust,
 * breaking every ID-carrying command (the entire editing surface).
 */
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";

describe("invokeIpc bigint argument serialization", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("converts bigint args (top-level, nested, arrays) to plain numbers", async () => {
    const invokeMock = vi.fn().mockResolvedValue(undefined);
    vi.doMock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
    vi.doMock("svelte-sonner", () => ({ toast: { error: vi.fn() } }));

    const { invokeIpc } = await import("../lib/ipc");
    await invokeIpc("cmd", {
      layerId: 1n,
      nested: { trackId: 2n },
      list: [3n],
    });

    expect(invokeMock).toHaveBeenCalledOnce();
    const [command, args] = invokeMock.mock.calls[0];
    expect(command).toBe("cmd");
    expect(args).toStrictEqual({
      layerId: 1,
      nested: { trackId: 2 },
      list: [3],
    });
    // Tauri's transport JSON.stringify must not throw on forwarded args.
    expect(() => JSON.stringify(args)).not.toThrow();
  });

  it("rejects bigints above Number.MAX_SAFE_INTEGER with a descriptive error", async () => {
    const invokeMock = vi.fn().mockResolvedValue(undefined);
    vi.doMock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
    vi.doMock("svelte-sonner", () => ({ toast: { error: vi.fn() } }));

    const { invokeIpc } = await import("../lib/ipc");
    const oversized = BigInt(Number.MAX_SAFE_INTEGER) + 1n;
    await expect(invokeIpc("cmd", { layerId: oversized })).rejects.toThrow(
      /MAX_SAFE_INTEGER/,
    );
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("createEmptyTrack converts the wire JSON number into a real bigint", async () => {
    const invokeMock = vi.fn().mockResolvedValue(7);
    vi.doMock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
    vi.doMock("svelte-sonner", () => ({ toast: { error: vi.fn() } }));

    const { createEmptyTrack } = await import("../lib/api");
    const trackId = await createEmptyTrack(1n, "New track");
    expect(typeof trackId).toBe("bigint");
    expect(trackId).toBe(7n);
  });
});
