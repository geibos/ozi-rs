import { describe, expect, it } from "vitest";
import { bundleSlugFromUrl } from "../lib/bundle-url";

/**
 * A link to a search arrives over a messenger, and by the time it is pasted it
 * has been through a link preview, an autolinker and a clipboard. Typing the
 * name back in instead is error-prone: the names are latin transliterations of
 * Russian place names and the one in the link is exactly right.
 */
describe("the slug in a catalogue link", () => {
  it("is read from the link a coordinator sends", () => {
    expect(
      bundleSlugFromUrl("https://maps.lizaalert.ru/maps/2026-09-21_Vesta/"),
    ).toBe("2026-09-21_Vesta");
  });

  it("survives a lost scheme, a missing trailing slash and tracking junk", () => {
    expect(bundleSlugFromUrl("maps.lizaalert.ru/maps/2026-09-21_Vesta")).toBe(
      "2026-09-21_Vesta",
    );
    expect(
      bundleSlugFromUrl(
        "https://maps.lizaalert.ru/maps/2026-09-21_Vesta/?utm_source=tg",
      ),
    ).toBe("2026-09-21_Vesta");
    expect(bundleSlugFromUrl("  https://maps.lizaalert.ru/maps/A/  ")).toBe(
      "A",
    );
  });

  it("decodes a name that was percent-encoded on the way", () => {
    expect(
      bundleSlugFromUrl(
        "https://maps.lizaalert.ru/maps/2026-09-21_%D0%92%D0%B5%D1%81%D1%82%D0%B0/",
      ),
    ).toBe("2026-09-21_Веста");
  });

  /**
   * Null rather than a guess: a filter box that jumped to a project because
   * someone typed a word with a slash in it would be worse than one that did
   * nothing.
   */
  it("is nothing for text that is not a catalogue link", () => {
    expect(bundleSlugFromUrl("Веста")).toBeNull();
    expect(bundleSlugFromUrl("")).toBeNull();
    expect(bundleSlugFromUrl("   ")).toBeNull();
    expect(bundleSlugFromUrl("2026-09-21 / Веста")).toBeNull();
  });

  it("is nothing for another host, however similar", () => {
    expect(
      bundleSlugFromUrl("https://maps.google.com/maps/2026-09-21_Vesta/"),
    ).toBeNull();
    expect(
      bundleSlugFromUrl("https://evil.example/maps.lizaalert.ru/maps/x/"),
    ).toBeNull();
  });

  it("is nothing for the catalogue root, which names no search", () => {
    expect(bundleSlugFromUrl("https://maps.lizaalert.ru/maps/")).toBeNull();
    expect(bundleSlugFromUrl("https://maps.lizaalert.ru/")).toBeNull();
  });
});
