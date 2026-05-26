## Why

User smoke of the redesigned workspace surfaced six concrete affordance failures that make the shell read as undifferentiated text: (1) the active Library tab has no visible state — Maps / Tracks / Waypoints all look identical; (2) the four mode chips (View / Draw / Edit / Measure) look exactly like the three tabs, so the top context-bar reads as seven peer text items; (3) the Inspector edge handle from the spec is missing — without a selection the rail unmounts and the user cannot pin it open; (4) row labels overflow rather than truncating with ellipsis + tooltip; (5) the Cmd-K trigger looks like a search input and invites text entry; (6) the bundle-loader Sheet entry point inside the Maps tab header overlaps semantically with the Maps tab itself. These were all promised in the redesign spec but lost during implementation; the workspace now ships less affordance than the spec contracts for, which is a regression the user can see on first launch.

## What Changes

- The shadcn `Tabs.Trigger` for the Library SHALL render a visible active-state contrast (background + foreground swap + 2px bottom indicator) so the active Library tab is unambiguous against the Zinc + Teal tokens.
- Mode chips (View / Draw / Edit / Measure) SHALL render as a visually-distinct cluster — pill background + a vertical separator from the Cmd-K trigger — and SHALL NOT look like tabs. As long as their behaviour is deferred to a later change, they SHALL render with `aria-disabled="true"`, `cursor: not-allowed`, and `opacity 0.55` to mark them inert rather than as clickable peers of the tabs.
- The Inspector rail SHALL always mount in the DOM, even when no selection is present, and SHALL render a thin vertical edge handle (8px wide, full height, hover-expand to 12px) on the right edge of the viewport. The handle SHALL host the pin toggle and act as the manual open affordance.
- `LibraryRow` name cells SHALL truncate via `text-overflow: ellipsis` + `white-space: nowrap` + `min-width: 0` and SHALL set `title={name}` for a native tooltip; the existing optional shadcn `Tooltip` on hover SHALL continue to read the full name when present.
- The Cmd-K trigger SHALL render as a button-shaped pill: no border resembling `input`, muted placeholder text, `⌘K` glyph pill on the right, no caret, and a hover background distinct from focus-ring styling so users do not type into it.
- The "Maps…" Sheet trigger SHALL be removed from the Maps-tab header. The Maps tab itself is the entry point to map switching; the bundle-loader Sheet SHALL be reachable only from the Cmd-K palette (already wired) and the cold-start `/` route. The two-affordance ambiguity SHALL be resolved by removing the redundant header button, not by relabelling it.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `ui-shell`: the affordance contracts for Library tab active state, mode-chip visual distinctness, Inspector edge handle, library-row truncation, Cmd-K trigger styling, and Maps-tab header content are tightened so the redesign matches the spec on first paint.

## Impact

- **Frontend**: token-level adjustments to `src/lib/components/ui/tabs/tabs-trigger.svelte`'s active-state classes; CSS-only changes to `src/components/WorkspaceShell.svelte` (mode chips, Cmd-K trigger, status-bar / canvas insets so the always-mounted Inspector can claim its 8px); rewiring of `src/components/InspectorRail.svelte` to be always-mounted with an edge handle; `src/components/library/LibraryRow.svelte` truncation rules; removal of the `Maps…` button JSX block from `src/components/library/MapsTab.svelte`.
- **Backend**: no change. No new IPC, no new events, no store schema change.
- **Spec evidence**: visual smoke at `/project` confirming the active Library tab is distinct from the inactive ones, mode chips are clearly inert and visually clustered, the Inspector edge handle is visible without a selection, list rows truncate gracefully, the Cmd-K trigger does not invite text entry, and the Maps-tab header is empty (no Sheet button).
- **Risk**: low. All changes are visual / structural with no domain semantics. The Inspector always-mount risks regressing `inspectorOpen` consumers — addressed in design.md.

## Out of scope

- Wiring the mode chips to actual mode state (Draw / Edit / Measure interpretation of map clicks) — defer to a dedicated change owning the mode store.
- Adding new colours, fonts, or shadow tokens to `src/lib/tokens.css` — this change consumes existing tokens.
- Changing the bundle-loader Sheet itself (its 480px width, its overlay, its dismiss contract). Only its entry point inside the Maps tab is removed.
- Inspector contents (Track / Waypoint / Map subcomponents). This change only fixes the rail chrome (always-mount, edge handle, pin position).
- Cmd-K palette behaviour. Only its trigger button styling changes.

## Dependencies

- Builds on the implemented `redesign-shell-layout`, `redesign-library-sidebar`, and `redesign-inspector-pane` work (all archived). This is a corrective change on top of them, not a redo.
- Compatible with `bundle-loader-as-overlay`: the Sheet still mounts at the workspace level via `bundleLoaderOpen`; only the Maps-tab-header button entry point is dropped. The Cmd-K palette entry remains.
