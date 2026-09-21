## 1. Backend

- [x] 1.1 `ProgressText` with key, ordered arguments and English wording, with a test
- [x] 1.2 Every progress call site on the bundle path converted to it
- [x] 1.3 `bundle-progress` carries `message_key` and `message_args`

## 2. Frontend

- [x] 2.1 `progressText` translator with a fallback to the backend's wording
- [x] 2.2 Single-pass positional substitution, with a test for a name that looks like a placeholder
- [x] 2.3 Twelve message keys and four phase words in both dictionaries
- [x] 2.4 Both status surfaces use them
- [x] 2.5 The stand's played-out download carries keys, so it exercises the translation rather than the fallback

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver cannot initialise UI testing on this machine — see `docs/STATE.md`)
