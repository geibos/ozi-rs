## 1. Frontend

- [x] 1.1 `bundleSlugFromUrl`: lenient about scheme, trailing slash, query and percent-encoding; strict about the host
- [x] 1.2 Null for anything that is not a catalogue link, including a lookalike host and the catalogue root
- [x] 1.3 The loader's search box acts on a pasted link once, and leaves the readable name behind
- [x] 1.4 An unlisted search says so, naming the slug

## 2. Evidence

- [x] 2.1 Six tests on the parsing
- [x] 2.2 Walked on the stand: pasting a link previews that slug and leaves "2026 09 20 Schuvalovo" in the box; pasting an unlisted one shows the message and previews nothing

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
