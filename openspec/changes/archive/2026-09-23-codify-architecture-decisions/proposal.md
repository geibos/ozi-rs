## Why

The project keeps its normative knowledge in three places that drift apart: 24 ADRs in `docs/adr/` (decisions and rationale, several stale or superseded — ADR-0011 still says "no async runtime" while the core depends on tokio), five legacy design specs and three executed plans in `docs/superpowers/` (April–May 2026, all implemented or abandoned), and the OpenSpec capability specs in `openspec/specs/` whose `## Purpose` sections are still the `TBD` stubs left by the bootstrap import. No spec cites any ADR; no ADR points to the requirement that implements it. Agents and humans therefore cannot tell which document is the source of truth, and the September 2026 review found human-facing docs contradicting code while specs were largely accurate.

This change makes OpenSpec the single normative source: every still-valid architecture decision becomes requirements with scenarios in the owning capability, the rationale moves into that capability's `## Purpose` as a decision history, superseded and stale decisions are recorded as history only, and the legacy documents become stubs or archives that point back into OpenSpec.

## What Changes

- **Decisions become requirements.** For each ADR the decision is verified against today's code and, where true and not yet specified, added as `### Requirement` blocks with checkable scenarios in the owning capability's delta spec. Where an existing requirement is imprecise relative to the decision, it is MODIFIED with the full block.
- **Rationale becomes Purpose.** Every capability's `## Purpose` replaces the `TBD` stub with a description paragraph and a `### Decision history` list (one bullet per ADR or legacy doc: decision, rationale, codified-as or superseded-by). Purpose sections of existing specs are edited directly, as the stub instructs; Purposes of capabilities created by this change are applied immediately after archive.
- **Three new capabilities** hold decisions that had no home: `architecture` (layering, `ProjectCommand` as the only mutation path, u64 newtype IDs, TrackStyle in the domain, Rust 2024 edition, tracing, the real threading/async model, the `ozf2` sibling crate), `product-scope` (MVP inclusions and explicit exclusions such as map printing and KML), `documentation` (docs claim only verified behavior, feature-status matrix rules, navigation docs synced with code, and the rule that new decisions are recorded in OpenSpec rather than as new ADR files).
- **Stale and superseded decisions are not codified.** ADR-0003 (egui, superseded by ADR-0016), ADR-0005 (snapshot undo, superseded by ADR-0017) and the parts of ADR-0011 contradicted by the tokio-based downloader appear only in decision history with the evidence.
- **ADR files become pointer stubs.** Because ~150 references to `ADR-00NN` exist across docs, `docs/qa/*`, code comments and archived changes, the identifiers stay resolvable: each `docs/adr/adr-00NN-*.md` is reduced to title, date, status and a "Codified in" pointer to the capability, requirement names and Purpose entry. Full texts remain in git history. `docs/adr/README.md` becomes the index. **BREAKING** for the documentation process: no new ADR files are created after this change; decisions go into a change's `design.md` and the capability's Purpose decision history.
- **Legacy superpowers documents are archived.** `docs/superpowers/specs/*` and `docs/superpowers/plans/*` move to `docs/archive/superpowers/` with a README mapping each file to the requirements, changes or ADR history that absorbed it. The Meetily-inspired backlog moves to `docs/backlog.md`. The four live references (`docs/agent-verification.md`, `docs/qa/triage.md`, `docs/qa/2026-04-28-audit-summary.md`, `docs/qa/2026-04-29-tooling-audit-findings.md`) are repointed.
- **QA process decisions** (ADR-0024 and the qa-debug-process spec) are folded into the active change `revive-ui-cycle` (capabilities `visual-verification` and `agent-workflow`), not duplicated here. `revive-ui-cycle` is amended so the planned "ADR-0025" is a spec requirement plus a Purpose decision-history entry instead of a new ADR file.

## Capabilities

### New Capabilities
- `architecture`: layering and dependency direction, single mutation path through `ProjectCommand`, identifier newtypes, style-in-domain, toolchain/edition, structured logging, threading and async model as implemented, OZF2 decoding via the sibling crate.
- `product-scope`: what the MVP includes (maps, tracks, waypoints, field tools, project persistence, GPX/PLT/WPT export) and what it explicitly excludes (map printing, KML, route planning and other ADR-0020 exclusions), each exclusion checkable against the command registry and UI.
- `documentation`: truthfulness rules for human-facing docs, feature-status matrix contract, navigation-doc sync, and decision recording through OpenSpec.

### Modified Capabilities
- `tile-rendering`: ADR-0010/0012/0016 tile decisions not yet specified (custom protocols, cache semantics, multi-level tiles) as verified in code.
- `project-persistence`: ADR-0004 JSON `.ozp` format facts and the July 2026 atomic-save behavior where not yet specified.
- `map-bundles`: ADR-0002 bundle/project separation invariants where not yet specified.
- `lizaalert-integration`: ADR-0008 HTTP stack constraints and download behavior facts (concurrency, `.part` files, cancel, no retries) where not yet specified.
- `undo-redo`: ADR-0017/0021 delta-based stack, depth 100, coalescing, identifier uniqueness across undo/redo.
- `track-editing`: editing invariants tied to the command stack where not yet specified.
- `track-import`: ADR-0007 archive staging boundary and encoding detection facts where not yet specified.
- `track-export`: ADR-0022 WPT export behavior and PLT field-order/cp1251 facts where not yet specified.
- `waypoints`: ADR-0018 symbol-as-optional-string invariant where not yet specified.
- `ui-shell`: only gaps from the shadcn UI-kit spec and ADR-0016 frontend decisions; no overlap with the `revive-ui-cycle` additions.
- `layers`, `track-display`: Purpose decision history only (no requirement changes expected).

The final list of delta files is whatever ends up non-empty after verification; capabilities whose deltas are empty receive Purpose edits only.

## Impact

- **Docs**: `openspec/specs/*/spec.md` Purpose sections (direct edits); `docs/adr/*.md` reduced to stubs plus `docs/adr/README.md`; `docs/superpowers/` moved to `docs/archive/superpowers/` with README; `docs/backlog.md` created; `AGENTS.md`, `CLAUDE.md`, `README.md`, `docs/project-map.md`, `docs/roadmap.md`, `docs/conventions.md`, `docs/agent-verification.md`, `docs/qa/triage.md` and the two QA audit docs repointed. No code changes. No CI changes beyond `openspec validate` continuing to pass.
- **Coordination**: `revive-ui-cycle` (active) owns `visual-verification` and `agent-workflow`; this change does not create or modify those capabilities. Both changes add `ui-shell` requirements with disjoint names. Archive order is free.
- **Risk**: low for code (none), medium for documentation drift during the transition: mitigated by keeping ADR identifiers resolvable through stubs and by a link check task before archive.

## Out of scope

- Writing new decisions or changing behavior. Anything found to be a gap between code and desired behavior goes to `docs/backlog.md` or a new change, not into this one.
- Re-authoring executed implementation plans; they are archived with a mapping only.
- The QA/verification process content, which lives in `revive-ui-cycle`.
