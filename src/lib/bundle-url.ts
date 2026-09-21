/**
 * The slug in a LizaAlert catalogue link.
 *
 * A link to a search arrives over a messenger — a coordinator sends
 * `https://maps.lizaalert.ru/maps/2026-09-21_Веста/` and the crew has to get
 * from that to the bundle. Typing the name back in is error-prone: the names
 * are latin transliterations of Russian place names, and the one in the link
 * is exactly right.
 *
 * Deliberately lenient about the shape: what arrives from a messenger has been
 * through a link preview, an autolinker and a paste, so it may have lost its
 * scheme, gained a trailing slash, or come wrapped in a `?utm_...`.
 */
const CATALOGUE_HOST = "maps.lizaalert.ru";

/**
 * The slug, or `null` when the text is not a catalogue link.
 *
 * Returns `null` rather than guessing: a filter box that jumped to a project
 * because someone typed a word with a slash in it would be worse than one that
 * did nothing.
 */
export function bundleSlugFromUrl(text: string): string | null {
  const trimmed = text.trim();
  if (trimmed === "") return null;

  // A paste often loses the scheme; `URL` insists on one.
  const withScheme = /^[a-z]+:\/\//i.test(trimmed)
    ? trimmed
    : `https://${trimmed}`;

  let url: URL;
  try {
    url = new URL(withScheme);
  } catch {
    return null;
  }

  if (url.hostname.toLowerCase() !== CATALOGUE_HOST) return null;

  const segments = url.pathname
    .split("/")
    .map((s) => decodeURIComponent(s))
    .filter((s) => s !== "");
  if (segments[0] !== "maps") return null;

  const slug = segments[1];
  return slug === undefined || slug === "" ? null : slug;
}
