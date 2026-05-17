## Context

ozi-rs is a Tauri 2 desktop application for LizaAlert search-and-rescue volunteers (Rust backend + Svelte 5 + MapLibre GL 4). It has shipped MVP behavior covering raster maps (MBTiles and OZF2), LizaAlert bundle browsing/download, projects with independent track and waypoint layers, GPX/PLT import/export, track editing (move/insert/delete points, segment ops, drawing, Douglas–Peucker simplification), waypoints with symbols, delta-based undo/redo, and theming.

The codebase is documented across `docs/` (`project-map.md`, `architecture.md`, `requirements.md`, `commands-reference.md`, `conventions.md`, `glossary.md`, `feature-status.md`, `persistence-session.md`) and 24 ADRs. None of these artifacts function as a SHALL/MUST behavioral specification that a change proposal can target. Adopting OpenSpec going forward requires first capturing the current behavior as the initial canonical specs.

## Goals / Non-Goals

**Goals:**

- Capture currently shipping behavior as 12 capability specs that future change proposals MODIFY, ADD, or REMOVE against.
- Keep capabilities aligned with user-visible behavior rather than internal module structure.
- Ground each requirement in SHALL/MUST language so each scenario maps to a verifiable test or QA check.
- Reference ADRs from specs only where a requirement directly follows from one; avoid generating exhaustive ADR cross-refs.

**Non-Goals:**

- No runtime, dependency, or build configuration changes.
- No retroactive rewrite or supersession of `docs/` content; existing docs remain as supporting material.
- No specification of behavior that is not yet implemented. Planned features (`WPT` waypoint export, on-map distance/circle/projection tools, timestamp-sort, recent-projects list, full layer manager UI, PDF print) are out of scope and will be introduced via their own future change proposals.
- No new domain or naming taxonomy beyond what `AGENTS.md` / `README.md` / `docs/glossary.md` already use.

## Decisions

### Decision 1 — Carve 12 capabilities by user-visible behavior, not module structure

Capabilities are vertical slices of behavior a user, integrator, or QA scenario can rely on; they are NOT 1:1 with `src-tauri/src/` modules.

Alternatives considered:

- **Coarse (~6 capabilities)**: easier navigation, but each spec becomes a long simplex and most changes touch the entire spec — defeats the granularity benefit.
- **Fine (~18 capabilities)**: cleanest boundaries but forces proposals to list 5+ specs for one feature — adds friction without payoff.

The middle granularity (12) matches the natural product surfaces — bundles, LizaAlert remote, tile protocols, project files, layers, four track verticals (import, export, display, editing), waypoints, undo, UI shell — and produces specs of 3–8 requirements each.

### Decision 2 — Bootstrap as a single OpenSpec change, archived once

Rather than writing directly into `openspec/specs/`, the bootstrap goes through a single OpenSpec change so the contract — *every spec change is itself a change proposal* — holds from day one. After review, `openspec archive bootstrap-current-state` merges the deltas into the canonical specs.

Alternative considered: write directly into `openspec/specs/`. Rejected because it bypasses the very workflow this change is supposed to establish.

### Decision 3 — Reference ADRs from specs only where a requirement directly follows from one

Each spec includes a short `## References` section pointing to ADRs that anchor the requirement (e.g. `undo-redo` references ADR-0017 / ADR-0021; `project-persistence` references ADR-0002 / ADR-0004). Most ADRs (logging choice, HTTP stack, Rust edition, etc.) inform implementation but are not behavioral contracts and remain in `docs/adr/` only.

ADRs surfaced by this design:

- **ADR-0001** (initial architecture): layer separation underpins almost every spec.
- **ADR-0002** (bundle ≠ project): foundational for `map-bundles`, `project-persistence`, `track-export`.
- **ADR-0006 / 0010 / 0012** (OZF2, OZI tiling, SQLite tile caching): anchor `tile-rendering`.
- **ADR-0007** (ZIP as staged import boundary): anchors `track-import`.
- **ADR-0008** (reqwest + rustls): anchors `lizaalert-integration` HTTP behavior.
- **ADR-0013** (TrackStyle in domain): anchors `track-display`.
- **ADR-0016** (Tauri + MapLibre + Svelte): anchors `tile-rendering` custom protocols.
- **ADR-0017 / 0021** (delta-based undo): the entire `undo-redo` spec.
- **ADR-0018** (waypoint symbols as optional strings): anchors `waypoints`.
- **ADR-0019** (doc audit reconciliation): anchors the warning-only OK-standard validation in `track-display` and `10-Tracks/` suggestion in `track-export`.
- **ADR-0020** (MVP scope): the boundary between what this bootstrap captures versus what is left for future change proposals.
- **ADR-0022** (WPT in MVP) and **ADR-0023** (no PDF print) call out work explicitly NOT in current specs.

ADRs not load-bearing for any current behavioral spec (e.g. ADR-0005 superseded snapshot undo, ADR-0009 tracing, ADR-0011 no async, ADR-0015 Rust 2024 edition, ADR-0024 Playwright not for desktop QA) are not referenced from spec files.

### Decision 4 — Capture warning-only and bounded-restore semantics explicitly

Several capabilities have intentionally permissive semantics:

- **OK-standard validation is warning-only** — the backend stays permissive; renames/saves/exports are not blocked. The `track-display` spec encodes this as positive SHALL requirements ("SHALL display a warning" plus "SHALL NOT block").
- **Session restore is bounded** — only the last project path and active map are restored. The `project-persistence` spec includes an explicit "non-restored items" requirement to keep future authors honest.
- **Track style mutations bypass undo** — `set_track_color`, `set_track_line_width`, `toggle_track_visible` are intentionally non-undoable. The `undo-redo` spec encodes this as a positive SHALL NOT requirement so future contributors do not silently move them onto the command stack.

### Decision 5 — Keep `feature-status.md` alongside specs

OpenSpec specs describe behavior contracts. `docs/feature-status.md` is a per-feature QA evidence matrix (smoke status, evidence file paths). They serve different audiences. This bootstrap does not deprecate the matrix; it lives in parallel and is owned by the QA workflow.

## Risks / Trade-offs

- **Spec drift**: future contributors update code without proposing changes. → Mitigation: this change updates `AGENTS.md` and `CLAUDE.md` to call out OpenSpec as the default path for non-trivial behavioral edits.
- **Coarse capability boundaries**: `track-editing` covers point ops, segment ops, drawing, and simplification — that is a wide span. → Mitigation: if a future change needs sharper boundaries, it can REMOVE requirements from `track-editing` and ADD a new capability in the same proposal.
- **ADR reference rot**: ADR references may become stale if ADRs are deprecated or renumbered. → Mitigation: references stay narrow and scoped to load-bearing ADRs; no exhaustive cross-refs.
- **Captured behavior reflects current MVP stance** including warning-only validation and "10-Tracks/ suggestion not enforcement". A future product decision to make these stricter is a MODIFIED Requirements delta against these specs — that is the OpenSpec workflow operating as intended.
