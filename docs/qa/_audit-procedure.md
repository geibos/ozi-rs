# Audit Procedure (per feature)

Apply this procedure to every MVP-scope feature listed in ADR-0020.

1. Open or create `docs/qa/smoke-<feature>.md` from `_template.md`.
2. Identify the UI entry point: open accessibility inspector or query Appium
   for the element. Record the selector in the smoke doc.
3. `build_app` (skip if a fresh build from this session already exists).
4. `launch_app`.
5. `qa_observe` to record the baseline screenshot for this feature.
6. For each step in the smoke doc:
   - Drive the action via the appropriate Appium tool.
   - `appium_screenshot` (or `qa_observe`) immediately after.
   - Compare against the smoke doc's "expected outcome" and record actual.
7. Classify the feature as works / partial / broken / hidden / missing.
8. `stop_app`.
9. Append the row to the in-progress triage list in `docs/qa/triage.md`.

If any step fails to produce an expected outcome, do not retry the action
twice. Record the failure as Classification = broken (or partial), capture
the artefact, and move on. Diagnosis happens in the post-audit triage,
not inside the audit itself.
