## 1. Frontend

- [x] 1.1 The Waypoints tab stamps its reload and drops an overtaken one, rows and failure alike
- [x] 1.2 The Waypoints tab reads every layer in parallel
- [x] 1.3 The Tracks tab stamps its reload the same way

## 2. Tests

- [x] 2.1 A held-open reload overtaken by a newer one leaves no trace, in both tabs (red before the change)
- [x] 2.2 A test that the Waypoints tab asks every layer at once — it deadlocks on the sequential version
- [x] 2.3 A `Tooltip.Provider` harness for the Tracks tab, as the Waypoints tab already had

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver cannot initialise UI testing on this machine — see `docs/STATE.md`)
