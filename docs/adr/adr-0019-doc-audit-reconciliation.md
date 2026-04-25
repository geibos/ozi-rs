# ADR-0019: Documentation Audit Reconciliation

- Status: accepted
- Date: 2026-04-25

## Context

A documentation audit found that several public docs described behavior that was either
missing, backend-only, or too broad for the intended MVP. The reconciliation must keep
the product documentation useful for LizaAlert search-and-rescue workflows without
claiming unsupported behavior.

The audit findings are:

1. **startup restore** — documented as complete, but missing in implementation.
2. **timestamp** — track point timestamps are documented as visible, but not rendered.
3. **color** — track color changes are documented, but no UI control is surfaced.
4. **line width** — track line-width changes are documented, but no UI control is surfaced.
5. **waypoint symbol** — symbol changes are documented as undoable, but currently bypass
   the command stack.
6. **OK-standard** — `YYYYMMDD_Callsign` validation is documented, but no warning is shown.
7. **10-Tracks** — exports are documented as defaulting to the LizaAlert `10-Tracks/`
   folder, but dialogs do not suggest it.
8. **multi-layer** — backend model supports multiple layers, while UI workflows still
   hardcode the default layer in places.
9. **windows.ts** — `src/lib/windows.ts` and the bundle-loader webview lifecycle are used
   but under-documented.
10. **feature-status** — there is no status matrix separating backend capability, UI
    exposure, documentation, and planned work.

## Decision

Use this reconciliation to distinguish three states explicitly: implemented behavior,
backend capability without full UI exposure, and Planned in this reconciliation work.

The selected decisions are:

| Finding | Decision |
|---------|----------|
| startup restore | Planned in this reconciliation: restore only the last project path and active map reference/path when they still exist. |
| timestamp | Planned in this reconciliation: render existing track point timestamp values in the point list without changing sorting semantics. |
| color | Planned in this reconciliation: expose the existing track color mutation through compact track UI controls. |
| line width | Planned in this reconciliation: expose the existing track line width mutation through compact track UI controls. |
| waypoint symbol | Planned in this reconciliation: route committed symbol changes through one undoable `ProjectCommand` per change. |
| OK-standard | Planned in this reconciliation: show a warning-only UI validation for `YYYYMMDD_Callsign`; do not block rename, save, or export. |
| 10-Tracks | Planned in this reconciliation: use active-bundle `10-Tracks/` paths as GPX/PLT export dialog defaults when available; users may still choose another path. |
| multi-layer | Planned in this reconciliation: add active track-layer and waypoint-layer selection for existing workflows; do not build full layer management. |
| windows.ts | Document `src/lib/windows.ts`, hidden bundle-loader precreation, `openBundleLoader()`, and `/?view=bundles` routing. |
| feature-status | Add `docs/feature-status.md` as the source for backend/UI/docs/status/evidence separation. |

### Session Restore Scope

Session restore is bounded to durable, file-backed state: last project path and active map
reference/path. It must tolerate missing files by starting fresh and reporting a warning or
diagnostic rather than panicking.

### Command-Stack Scope

Waypoint symbol changes are user-visible project edits, so they should use the same
delta-based undo model as other waypoint mutations. Track style changes remain immediate
non-undoable mutations unless a later ADR changes ADR-0017.

### Feature Status Policy

Public docs must not describe a feature as simply "supported" when only the backend exists
or only a future task is planned. `docs/feature-status.md` records the backend state, UI
state, documentation state, current status, and evidence for audited features.

## Non-goals

- No viewport, panel, selected-entity, bundle-loader window, or other window restore.
- No full layer manager: no create/rename/delete/reorder layer UI is part of this decision.
- OK-standard validation is warning-only and must not block workflows.
- No implementation changes are made by this ADR itself.

## Consequences

### Positive

- The audit findings are reconciled by explicit implement-vs-document decisions.
- Later tasks can update docs from "Planned in this reconciliation" to implemented only
  after their own verification passes.
- The feature-status table reduces future ambiguity between backend capability and UI
  exposure.

### Negative

- Some public docs remain temporarily conservative until later reconciliation tasks land.
- Minimal multi-layer UI may still leave advanced layer management as a post-MVP gap.
