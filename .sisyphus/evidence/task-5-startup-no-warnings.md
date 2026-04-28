# Task 5 Release Startup Warning/Log Output

Source checkbox: `- [ ] 5. Fix confirmed release startup warning/log output`

## Result

No runtime/logging/source code change is required for Task 5.

Task 1's inventory states: "Release startup runtime output assigned to Task 5: no warning/error-like startup scanner matches captured. Startup emitted two `INFO` lines from `ozi_rs_lib::application`, which did not match the required scanner terms."

The raw startup phase confirms the same bounded output:

```text
===== startup: ./src-tauri/target/release/ozi-rs =====
2026-04-27T05:55:37.348907Z  INFO ozi_rs_lib::application: Loading project list...
2026-04-27T05:55:39.449675Z  INFO ozi_rs_lib::application: Loaded 12742 projects
===== startup termination: TERM after 15 seconds, KILL fallback after 2 seconds =====
===== startup capture complete =====
```

Because Task 1 captured no startup warning/error-like lines, adding release-specific log filtering, changing tracing setup, downgrading logs, or muting startup diagnostics would hide signal without fixing an actionable warning. Development diagnostics should remain available until a concrete startup warning/error is captured and classified.

## Verification

- Checked `.sisyphus/evidence/task-1-warning-inventory.md`: line 70 assigns no warning/error-like startup scanner matches to Task 5 and records only two `INFO` diagnostics from `ozi_rs_lib::application`.
- Checked `.sisyphus/evidence/task-1-run-release-raw.log`: startup lines 163-167 contain the startup boundary, two `INFO` lines, the termination marker, and the capture-complete marker; no warning/error-like startup row appears there.
- Reconfirmed process cleanup with `pgrep -fl '(^|/)ozi-rs$' || true`: no output, so no `ozi-rs` process remains.
- No runtime, logging, source, frontend startup, Tauri config, or plan files were modified for this no-op task.

Task 6 should still run the final bounded startup verification so the completed release-warning cleanup has fresh end-to-end evidence.
