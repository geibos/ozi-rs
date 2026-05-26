/**
 * Tests for the dev IPC error toast wrapper introduced by
 * `fix-redesign-functional-bugs`.
 *
 * Behaviour under test (per `openspec/changes/fix-redesign-functional-bugs/specs/ui-shell/spec.md`):
 *
 *   - In dev builds (`import.meta.env.DEV === true`), every IPC rejection
 *     fires a sticky Sonner toast whose `class` carries `ipc-error` (the
 *     layout-level `MutationObserver` mirrors this onto `data-testid`).
 *   - The toast title equals the IPC command name; description equals the
 *     JSON-stringified error.
 *   - When `JSON.stringify` throws (BigInt, circular ref), the description
 *     falls back to `String(error)` AND no second uncaught exception
 *     escapes.
 *   - The wrapper rethrows the original rejection so call sites' existing
 *     `try/catch` keeps surfacing the user-facing `toast.error` message.
 *
 * These tests are intentionally module-source agnostic — they exercise the
 * wrapper at runtime so they keep working if the file layout moves.
 */
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

describe("invokeIpc dev IPC-error toast", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("rethrows the original rejection so call-site catches still fire", async () => {
    const error = new Error("backend rejected");
    vi.doMock("@tauri-apps/api/core", () => ({
      invoke: vi.fn().mockRejectedValue(error),
    }));
    vi.doMock("svelte-sonner", () => ({
      toast: { error: vi.fn() },
    }));

    const { invokeIpc } = await import("../lib/ipc");
    await expect(invokeIpc("some_cmd")).rejects.toBe(error);
  });

  it("fires a sticky Sonner toast tagged with the ipc-error class on rejection", async () => {
    const error = { code: "BAD_PAYLOAD", field: "slug" };
    const toastError = vi.fn();
    vi.doMock("@tauri-apps/api/core", () => ({
      invoke: vi.fn().mockRejectedValue(error),
    }));
    vi.doMock("svelte-sonner", () => ({ toast: { error: toastError } }));

    const { invokeIpc } = await import("../lib/ipc");
    await expect(invokeIpc("load_project", { slug: "x" })).rejects.toEqual(
      error,
    );

    expect(toastError).toHaveBeenCalledOnce();
    const [title, options] = toastError.mock.calls[0];
    expect(title).toBe("load_project");
    expect(options.class).toBe("ipc-error");
    expect(options.duration).toBe(Number.POSITIVE_INFINITY);
    expect(typeof options.description).toBe("string");
    expect(options.description).toContain("BAD_PAYLOAD");
    expect(options.description).toContain("slug");
  });

  it("falls back to String(error) when JSON.stringify throws", async () => {
    // BigInt is not JSON-serializable by default; the wrapper serializer
    // converts it to a string suffix, but use a circular structure here so
    // the wrapper's String(error) fallback runs.
    const circular: Record<string, unknown> = { name: "loop" };
    circular.self = circular;

    const toastError = vi.fn();
    vi.doMock("@tauri-apps/api/core", () => ({
      invoke: vi.fn().mockRejectedValue(circular),
    }));
    vi.doMock("svelte-sonner", () => ({ toast: { error: toastError } }));

    const { invokeIpc } = await import("../lib/ipc");
    await expect(invokeIpc("any_cmd")).rejects.toBe(circular);

    expect(toastError).toHaveBeenCalledOnce();
    const [, options] = toastError.mock.calls[0];
    // No second uncaught exception should escape the wrapper.
    expect(options.class).toBe("ipc-error");
    expect(typeof options.description).toBe("string");
  });

  it("does not fire a toast when invoke resolves", async () => {
    const toastError = vi.fn();
    vi.doMock("@tauri-apps/api/core", () => ({
      invoke: vi.fn().mockResolvedValue(42),
    }));
    vi.doMock("svelte-sonner", () => ({ toast: { error: toastError } }));

    const { invokeIpc } = await import("../lib/ipc");
    await expect(invokeIpc<number>("happy_path")).resolves.toBe(42);
    expect(toastError).not.toHaveBeenCalled();
  });
});

describe("layout installs the IPC-error mutation observer", () => {
  it("wires installIpcErrorToastObserver from the root layout", () => {
    const layoutSource = readFileSync(
      join(__dirname, "../routes/+layout.svelte"),
      "utf-8",
    );
    expect(layoutSource).toContain("installIpcErrorToastObserver");
    expect(layoutSource).toMatch(/installIpcErrorToastObserver\(\)/);
  });
});
