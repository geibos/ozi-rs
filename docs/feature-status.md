# Feature Status

This table separates backend capability, UI exposure, documentation state, implementation
status, and evidence. Use it to avoid broad "supported" claims when a feature is only
partially available.

| Feature | Backend | UI | Docs | Status | Evidence |
|---------|---------|----|------|--------|----------|
| Startup restore | Planned in this reconciliation: persist last project path and active map reference. | Planned startup use only; no viewport/panel/window restore. | ADR-0019 and `docs/persistence-session.md` define scope. | Planned in this reconciliation | Task 2 will add tests/evidence. |
| Track point timestamp display | Timestamp values exist in imported/project data when available. | Planned in this reconciliation: render values in point list. | ADR-0019 records decision. | Planned in this reconciliation | Task 3 will add UI verification. |
| Track color controls | Backend mutation exists for track color. | Planned in this reconciliation: expose compact control. | ADR-0019 records decision. | Planned in this reconciliation | Task 4 will add verification. |
| Track line width controls | Backend mutation exists for line width. | Planned in this reconciliation: expose compact control. | ADR-0019 records decision. | Planned in this reconciliation | Task 4 will add verification. |
| Waypoint symbol undo | Current backend symbol mutation exists but bypasses undo. | Existing symbol picker remains; committed changes should become undoable. | ADR-0019 records command-stack decision. | Planned in this reconciliation | Task 5 will add undo/redo tests. |
| OK-standard warning | Backend remains permissive. | Planned in this reconciliation: warning-only `YYYYMMDD_Callsign` validation. | ADR-0019 records warning-only behavior. | Planned in this reconciliation | Task 6 will add UI verification. |
| 10-Tracks export default | Planned helper/default path for active bundle exports. | Planned in this reconciliation: GPX/PLT dialogs suggest `10-Tracks/`. | ADR-0019 records scope. | Planned in this reconciliation | Task 7 will add path/dialog verification. |
| Multi-layer workflows | Backend supports multiple layers. | Planned in this reconciliation: active-layer selection, not full layer manager. | ADR-0019 records non-goal for full layer management. | Planned in this reconciliation | Task 8 will add workflow verification. |
| `windows.ts` bundle-loader lifecycle | Tauri secondary window support exists. | Existing bundle-loader window uses `src/lib/windows.ts`. | Planned in this reconciliation: document `windows.ts` and `/?view=bundles`. | Planned in this reconciliation | Task 9 will add docs verification. |
| Backend capability/status split | Backend varies by feature. | UI exposure varies by feature. | This table is the status split skeleton. | Introduced | Task 10 will complete evidence links. |
