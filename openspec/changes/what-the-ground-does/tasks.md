## 1. Frontend

- [x] 1.1 `elevationProfile` and `profilePath` in `src/lib/elevation-profile.ts`, with tests written first
- [x] 1.2 The inspector card draws the profile and the range, or says the recording has no elevation
- [x] 1.3 `inspector.elevationSoon` removed from both dictionaries

## 2. Verification

- [x] 2.1 On the stand: the card renders `M0,40 L6.58,20 L8.76,0 L84.1,40 L100,20` in a 306×39 box with the range reading `30–32 м`, from the fixture's two segments
- [x] 2.2 `just ci` green
- [ ] 2.3 `just smoke` green (blocked: the Mac2 driver cannot enable automation mode — see `docs/STATE.md`)
