# Smoke: <feature-name>

Source: ADR-0020 (MVP scope), section <area>.

## Preconditions
- App built (Task 1, this plan)
- App launched via `launch_app`
- Fixtures: <list .gpx / .plt / bundle paths used>

## UI entry point
- Selector: <Appium accessibility selector or coords with screenshot>
- Notes: <how to reach this entry point from app start>

## Steps and expected outcomes

1. **Action**: <Appium call, e.g. `appium_click(selector="//button[@title='Maps']")`>
   **Expected**: <observable property — text appears, list non-empty, panel opens>
   **Artifact**: <screenshot path>

2. **Action**: ...
   **Expected**: ...
   **Artifact**: ...

## Classification
- [ ] works (all steps pass)
- [ ] partial (happy path passes; documented edge case fails — see Known failure modes)
- [ ] broken (UI entry reachable, but action does not produce expected outcome)
- [ ] hidden (no UI entry point found; backend may exist)
- [ ] missing (neither UI nor backend)

## Evidence
- Build: <evidence path>
- Launch: <evidence path>
- Action screenshots: <list>

## Known failure modes
- <pattern> → <likely cause> → <next diagnostic step>
