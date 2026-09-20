## 1. Translation (parallel, by capability ownership)

- [x] 1.1 Architecture slice: delta `specs/architecture/spec.md`, delta `specs/tile-rendering/spec.md`, Purposes for `tile-rendering` and `layers` (ADR-0001, 0006, 0009, 0010, 0011, 0012, 0013, 0014, 0015, backend parts of 0016)
- [x] 1.2 Persistence slice: deltas for `project-persistence`, `map-bundles`, `lizaalert-integration` and their Purposes (ADR-0002, 0004, 0008; gap check against the production-bugs-fix plan)
- [x] 1.3 Undo/editing slice: deltas for `undo-redo`, `track-editing` and their Purposes (ADR-0005 history only, 0017, 0021)
- [x] 1.4 Formats slice: deltas for `track-import`, `track-export`, `waypoints` and their Purposes (ADR-0007, 0018, 0022 export behavior)
- [x] 1.5 Scope/docs slice: deltas `specs/product-scope/spec.md`, `specs/documentation/spec.md`, gap-only delta for `ui-shell`; Purposes for `ui-shell`, `track-display` (ADR-0019, 0020, 0022 scope, 0023, frontend parts of 0016; superpowers mvp-scope, documentation-update, shadcn-ui-kit; meetily → backlog list)
- [x] 1.6 QA slice (writes into `revive-ui-cycle`): extend `visual-verification` and `agent-workflow` deltas with the still-valid legacy QA process; Purposes for `ci-pipeline`, `build-tooling` (ADR-0024, qa-debug-process spec, docs/qa, mvp-audit and mcp-tooling-fixes plans)

## 2. Consolidation

- [x] 2.1 Merge the six coverage reports into a table in `design.md` (source → covered / ADDED / MODIFIED / superseded, with evidence) and record every "code contradicts ADR" finding
- [x] 2.2 Grep both active changes for identical `### Requirement:` headers per capability; resolve any collision by renaming or merging
- [ ] 2.3 Spot-check every ADDED requirement against its cited evidence (file:line or test); drop or fix any that codify unverified behavior
- [x] 2.4 Amend `revive-ui-cycle`: replace "ADR-0025" with a requirement plus Purpose decision-history entry in proposal.md, design.md, tasks.md and the `visual-verification` delta; keep the ADR-0024 wording amendment as a stub update instead of a text edit
- [x] 2.5 `openspec validate codify-architecture-decisions --strict` and `openspec validate revive-ui-cycle --strict` both pass
- [ ] 2.6 Fix the documentation bugs listed in design.md "Findings" so the `documentation` scenarios pass: feature-status rows for sort/crop/split-join/ZIP, command preamble count, commands-reference completeness, `TBD` → `pending`, conventions.md concurrency, architecture.md ozf2, glossary DTO note, frontend-architecture Ctrl+Z, persistence-session.md paths, native-qa-mcp.md `qa_observe`

## 3. Owner review and archive

- [ ] 3.1 Owner reviews proposal, design coverage table, new capabilities `architecture`, `product-scope`, `documentation`, and the Purpose texts
- [x] 3.1a Owner decisions (2026-09-19), recorded in the owning Purposes: code is primary, so bundles-root persistence is implemented in code (`revive-ui-cycle` slice 0.3) and the requirement stands; the waypoint GPX export requirement stays as a target (CJ-6, phase 4); manual smoke documents are dropped in favour of outcome-asserting per-CJ smoke tests; the Tier-1-only rule stays dropped; an Esc-cancelled draw is discarded without a redo entry (requirement rewritten, implemented in slice 0.3); platforms: macOS primary now, Windows in the distribution phase
- [ ] 3.2 Archive the change with `/opsx:archive codify-architecture-decisions` (specs merge)
- [ ] 3.3 Apply the Purpose paragraphs and decision histories to the newly created `openspec/specs/architecture/spec.md`, `product-scope/spec.md`, `documentation/spec.md`

## 4. Legacy documents

- [ ] 4.1 Reduce each `docs/adr/adr-00NN-*.md` to a stub: title, date, status, "Codified in" pointer (capability, requirement names, Purpose entry) or "Superseded by"; keep filenames
- [ ] 4.2 Write `docs/adr/README.md` indexing all 24 ADRs with capability and status, and stating that new decisions are recorded in OpenSpec (change `design.md` + Purpose decision history)
- [ ] 4.3 Move `docs/superpowers/specs/*` and `docs/superpowers/plans/*` to `docs/archive/superpowers/` (use `git mv`); write `docs/archive/superpowers/README.md` mapping each file to the requirements, changes or history that absorbed it
- [ ] 4.3a Move `docs/qa/smoke-*.md`, `docs/qa/_template.md`, `docs/qa/_audit-procedure.md` and `docs/qa/triage.md` to `docs/archive/qa/` with a README stating that outcome-asserting per-CJ smoke tests replaced the manual smoke layer (owner decision 2026-09-19); keep the two audit summaries as history
- [ ] 4.4 Create `docs/backlog.md` from the meetily list, the July manual-test backlog and the "Code findings" list in design.md (LayerId `len()+1`, `qa_observe` no-op, HTTP timeouts, zip-slip on track import, `.kml` Supported without parser, unused `lucide-svelte`, cp1251 replacement chars, swallowed export errors, waypoint marker glyph, unused `get_ozi_tile`, declared-but-absent ADR-0020 items); repoint `docs/agent-verification.md`, `docs/qa/triage.md`, `docs/qa/2026-04-28-audit-summary.md`, `docs/qa/2026-04-29-tooling-audit-findings.md`
- [ ] 4.5 Update `AGENTS.md`, `CLAUDE.md`, `README.md`, `docs/project-map.md`, `docs/roadmap.md`, `docs/conventions.md`: OpenSpec is the single normative source; ADRs are stubs; no new ADR files
- [ ] 4.6 Link check: every `ADR-00NN` mentioned outside `docs/adr/` resolves to an existing stub; `grep -r "docs/superpowers/"` outside `docs/archive/` and `openspec/changes/archive/` returns nothing
- [ ] 4.7 `just ci` passes (openspec-validate job included); commit on a branch, PR, merge, push
