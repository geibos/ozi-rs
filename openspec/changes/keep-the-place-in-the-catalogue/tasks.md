## 1. Frontend

- [x] 1.1 `bundleLoaderView` holds filter, only-cached, selection, cleared contents and scroll offset
- [x] 1.2 The loader initialises from it and writes every change back
- [x] 1.3 The first-run scroll reset no longer discards the restored offset

## 2. Tests

- [x] 2.1 A behavioural test mounts the real component, types a search, unmounts it as the Sheet does and mounts it again
- [x] 2.2 The same test confirms a fresh session starts empty
- [x] 2.3 `$app/navigation` and `$app/paths` stubs so components that navigate can be mounted

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver cannot initialise UI testing on this machine — see `docs/STATE.md`)
