import { describe, expect, it } from "vitest";
import { get } from "svelte/store";
import { locale, progressText } from "../lib/i18n";

/**
 * Bundle progress reaches the status bar verbatim, so while it was a sentence
 * built in Rust it was English in a Russian window — the one part of a
 * download a crew watches that never spoke their language. The backend sends a
 * key and its arguments now, and the English wording it used to send alone
 * stays as the fallback.
 */
describe("a backend progress message in the interface's language", () => {
  it("fills the positional arguments into the active language's wording", () => {
    locale.set("ru");
    expect(
      get(progressText)(
        "progress.downloadedOfFiles",
        ["2", "3"],
        "Downloaded 2 of 3 files",
      ),
    ).toBe("Скачано 2 из 3 файлов");

    locale.set("en");
    expect(
      get(progressText)(
        "progress.downloadedOfFiles",
        ["2", "3"],
        "Downloaded 2 of 3 files",
      ),
    ).toBe("Downloaded 2 of 3 files");
  });

  /**
   * A bundle name is free text off a web listing, so it can itself contain
   * something shaped like a placeholder. Substituting one argument at a time
   * would then substitute into what the previous one inserted.
   */
  it("does not substitute into an argument it has already inserted", () => {
    locale.set("ru");
    expect(
      get(progressText)(
        "progress.extractingInParallel",
        ["{1}", "карты"],
        "Extracting {1} in parallel: карты",
      ),
    ).toBe("Распаковка {1} параллельно: карты");
  });

  it("leaves a placeholder alone when no argument was sent for it", () => {
    locale.set("ru");
    expect(
      get(progressText)("progress.downloadedOfFiles", ["2"], "Downloaded 2"),
    ).toBe("Скачано 2 из {1} файлов");
  });

  it("shows what the backend said when the key is unknown, never the key", () => {
    locale.set("ru");
    const text = get(progressText)(
      "progress.somethingAddedLater",
      ["7"],
      "Sharpening 7 axes",
    );
    expect(text).toBe("Sharpening 7 axes");
    expect(text).not.toContain("progress.");
  });
});
