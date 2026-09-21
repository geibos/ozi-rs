## 1. Tests

- [x] 1.1 A round trip over a two-segment track: name, segment boundary, coordinates, timestamps
- [x] 1.2 The colour gap pinned, against a colour that is not the default
- [x] 1.3 A waypoint round trip: Cyrillic name, coordinates, symbol, and absence of one
- [x] 1.4 Surveyed the other two formats rather than assuming: PLT already round-trips name, colour and width; WPT has no importer by design and its output is pinned line by line

## 2. Gates

- [x] 2.1 `just ci` green
- [ ] 2.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
