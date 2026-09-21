## Why

`product-scope` declares opening a bundle directly by URL, and it had never
been built. A link to a search is how one arrives: a coordinator sends
`https://maps.lizaalert.ru/maps/2026-09-21_Vesta/` over a messenger.

Typing the name back in instead is error-prone in exactly the way that costs
time in the field — the names are latin transliterations of Russian place
names, and the one in the link is the one that is exactly right.

## What Changes

- A catalogue link pasted into the loader's search box opens that search.
- No new control. The search box is where a paste naturally lands, and text
  that is not a link is still just a filter.
- The parsing is lenient about what a messenger does to a link — a lost
  scheme, a missing trailing slash, a `?utm_source`, a percent-encoded Cyrillic
  name — and strict about the host, so a lookalike domain is not a catalogue
  link.
- Text that is not a catalogue link yields nothing rather than a guess: a box
  that jumped to a project because someone typed a word with a slash in it
  would be worse than one that did nothing.
- A link to a search the catalogue has not listed yet says so, naming the slug.
  That is the likeliest case for a link sent minutes after the search was
  created, and it is not an error.

## Impact

- Affected specs: `lizaalert-integration`
- Affected code: `src/lib/bundle-url.ts` (new),
  `src/components/BundleLoader.svelte`, `src/lib/i18n.ts`
