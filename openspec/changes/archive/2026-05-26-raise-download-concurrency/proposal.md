## Why

`DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY = 3` (`src-tauri/src/infrastructure/lizaalert.rs:93`) caps every production bundle download at three parallel HTTP fetches. The user reports bundle downloads as visibly slow. Modern TCP and the LizaAlert host comfortably handle 6–8 parallel connections per origin (browsers default to 6); 3 leaves bandwidth on the table for the common case of a bundle containing 8+ files of mixed sizes.

The existing `lizaalert-integration` spec already allows "default ≥3"; this change raises the actual default and tightens the lower bound to reflect the new value.

## What Changes

- `DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY` SHALL be raised from `3` to `6`.
- The `lizaalert-integration` spec's lower bound on the bounded worker pool default SHALL be tightened from `≥3` to `≥6` so future tweaks cannot regress below the new floor.
- No new configuration surface is introduced. The constant remains a single source of truth; per-bundle override remains the existing `BundleDownloadConfig.concurrency` field.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `lizaalert-integration`: tighten the bounded-pool default lower bound from `≥3` to `≥6`.

## Impact

- **Backend** (`src-tauri/src/infrastructure/lizaalert.rs`): one-line constant change; existing unit test `concurrency_cap_is_respected` uses its own local config (`concurrency: 3`) and is unaffected.
- **No frontend or IPC changes.**
- **Spec evidence**: bundle download wall-clock noticeably faster on multi-file bundles; existing `concurrency_cap_is_respected` test continues to pass against its own local cap.
- **Risk**: low. The LizaAlert host is the same one a browser would hit at 6 concurrent connections; this brings ozi-rs in line with browser baseline. If a future rate-limit issue surfaces, the per-bundle `concurrency` override stays available.
