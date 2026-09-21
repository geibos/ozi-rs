import { beforeEach, describe, expect, it, vi } from "vitest";

const { error } = vi.hoisted(() => ({ error: vi.fn() }));
vi.mock("svelte-sonner", () => ({ toast: { error } }));

import { reportEditFailure } from "../lib/edit-failure";
import { locale } from "../lib/i18n";

/**
 * A refused edit on the map used to reach `console.error` and nothing else,
 * and the error reporter is disabled outside dev builds — so in a release
 * build it was silent. The operator dragged a point, the backend declined, and
 * the only sign was that the next reload put it back.
 */
describe("a refused edit on the map", () => {
  beforeEach(() => {
    error.mockClear();
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  it("reaches the operator in their language, carrying the backend's reason", () => {
    locale.set("ru");
    reportEditFailure("map.movePointFailed", "layer 3 not found");

    expect(error).toHaveBeenCalledTimes(1);
    const [message, options] = error.mock.calls[0];
    expect(message).toBe("Не удалось переместить точку");
    expect(options.description).toBe("layer 3 not found");
  });

  it("passes an Error through as its message rather than as [object Object]", () => {
    locale.set("en");
    reportEditFailure("map.deletePointFailed", new Error("point is locked"));

    const [, options] = error.mock.calls[0];
    expect(options.description).toContain("point is locked");
  });
});
