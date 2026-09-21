import { describe, expect, it } from "vitest";
import { formatBytes, formatOptionalBytes } from "../lib/format-bytes";

describe("formatBytes", () => {
  it("speaks the language the app is running in", () => {
    expect(formatBytes(185.1 * 1024 ** 2, "ru")).toBe("185.1 МиБ");
    expect(formatBytes(185.1 * 1024 ** 2, "en")).toBe("185.1 MiB");
  });

  it("picks the unit that keeps the number readable", () => {
    expect(formatBytes(742, "ru")).toBe("742 Б");
    expect(formatBytes(2048, "ru")).toBe("2 КиБ");
    expect(formatBytes(3 * 1024 ** 3, "ru")).toBe("3.0 ГиБ");
  });

  it("does not pretend an unknown size is zero", () => {
    expect(formatOptionalBytes(null, "ru")).toBeNull();
    expect(formatOptionalBytes(undefined, "ru")).toBeNull();
    expect(formatOptionalBytes(0, "ru")).toBe("0 Б");
  });

  it("refuses nonsense rather than printing NaN", () => {
    expect(formatBytes(Number.NaN, "ru")).toBe("—");
    expect(formatBytes(-1, "ru")).toBe("—");
  });
});
