1. **Top risks**
- `sanity_fmt` is failing and also allowlisted in `required_failures`, which weakens the quality gate.
- Pass rate looks good (12/13), but coverage signal is weak (`unit_tests.total = 1`), so regressions can slip through.
- `quick_mode=true` means this is a partial confidence run, not a full release signal.

2. **Probable root causes**
- Formatting/lint drift (formatter not run, rule mismatch, or config drift).
- Known failure is being institutionalized via `required_failures` instead of being fixed.
- Test pyramid is unbalanced: many smoke/practical checks, very little unit/sanity depth.

3. **Concrete next actions**
1. Reproduce and fix `sanity_fmt` immediately (run the exact formatter/lint command behind that check and commit the diff).
2. Remove `sanity_fmt` from `required_failures` once green; make sanity failures blocking in CI.
3. Add unit tests for the highest-risk modules touched recently (edge cases + error paths first).
4. Run a full suite with `quick_mode=false` before merge and gate merges on full pass.

Confidence: **high** on risks; **moderate-high** on root-cause inference (limited to summary data).