## 1. Review and approve bootstrap content

- [x] 1.1 Review `proposal.md` and `design.md` for accuracy and tone
- [x] 1.2 Review each of the 12 capability spec deltas for completeness against current shipping behavior
- [x] 1.3 Confirm capability boundaries and kebab-case slugs

## 2. Validate

- [x] 2.1 Run `openspec validate bootstrap-current-state --strict` and resolve any reported issues
- [x] 2.2 Run `openspec list` to confirm the change is registered

## 3. Archive

- [x] 3.1 Run `openspec archive bootstrap-current-state` to populate `openspec/specs/`
- [x] 3.2 Verify `openspec/specs/` contains 12 capability directories, each with a `spec.md`

## 4. Update agent docs to point at the OpenSpec workflow

- [x] 4.1 Update `AGENTS.md` / `CLAUDE.md` to direct contributors to the OpenSpec change workflow for behavioral changes
