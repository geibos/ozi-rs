## 1. Backend

- [x] 1.1 `download_file_with_retries` with a bounded attempt count and a short backoff
- [x] 1.2 A test on a server that fails once and then serves: the file lands whole, after exactly one retry
- [x] 1.3 A test that a cancelled transfer is not retried
- [x] 1.4 The bundle download uses it
- [x] 1.5 `ProgressText::RetryingFile` so the retry reaches the status bar, in both dictionaries

## 2. Gates

- [x] 2.1 `just ci` green
- [ ] 2.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
