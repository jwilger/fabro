All checks pass. 

**Summary: The code is already clean — no changes needed.**

The diff is a minimal, surgical bugfix:
1. **One-line fix** (`node_label` → `node_id`): Aligns the HashMap key in `extract_stage_durations()` with how callers look up values, fixing the bug where all durations showed `0s`.
2. **Test hardening**: Changed test data so `node_label` differs from `node_id` (e.g., `"Plan"` vs `"plan"`), ensuring the test would have caught the original bug.

No code reuse, quality, or efficiency issues were found. Tests pass, clippy is clean.