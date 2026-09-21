import { describe, expect, it } from "vitest";
import { formatCatalogueAge } from "../lib/catalogue-age";

/**
 * The catalogue cache has carried an ISO timestamp since it was written, and
 * nothing ever showed it. Offline — which is the state this cache exists for —
 * a crew reads a list without knowing whether it is from this morning or from
 * three weeks ago, and a search created yesterday looks like a search that
 * does not exist.
 *
 * Absolute, not relative: "3 часа назад" needs Russian plural rules to be
 * written correctly, and a clock time is what a crew compares against "when
 * did the coordinator say it was published".
 */
describe("formatCatalogueAge", () => {
  it("is null when nothing was ever cached", () => {
    expect(formatCatalogueAge(null, "ru")).toBeNull();
  });

  it("is null for a timestamp that is not a date", () => {
    // A hand-edited or truncated localStorage entry must not print "Invalid
    // Date" into the one line that is supposed to build trust.
    expect(formatCatalogueAge("not a date", "ru")).toBeNull();
    expect(formatCatalogueAge("", "ru")).toBeNull();
  });

  it("gives day, month and time", () => {
    const formatted = formatCatalogueAge("2026-09-21T11:32:00.000Z", "ru");
    expect(formatted).toMatch(/\d{2}\.\d{2}/);
    expect(formatted).toMatch(/\d{2}:\d{2}/);
  });

  it("does not depend on the locale to stay readable", () => {
    const ru = formatCatalogueAge("2026-09-21T11:32:00.000Z", "ru");
    const en = formatCatalogueAge("2026-09-21T11:32:00.000Z", "en");
    expect(ru).toBeTruthy();
    expect(en).toBeTruthy();
  });

  it("formats a timestamp from the future like any other", () => {
    // Clock skew between machines is ordinary; it is still a real timestamp,
    // and printing it lets the crew see that something is off.
    expect(formatCatalogueAge("2030-01-02T10:00:00.000Z", "ru")).toBeTruthy();
  });
});
