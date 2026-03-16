Clean. Here's a summary of the review and what was fixed:

## Review Summary

### Agent 1 (Code Reuse)
- No issues found. No existing rankdir/direction utilities elsewhere in the codebase. The new enum and regex are appropriately scoped to this CLI command.

### Agent 2 (Code Quality) — 1 fix applied
- **Redundant match duplicating `Display` impl**: The `replacement` match in `apply_direction` hand-wrote `"rankdir=LR"` / `"rankdir=TB"` when `GraphDirection` already has a `Display` impl producing `"LR"` / `"TB"`. Fixed by using `format!("rankdir={dir}")` instead of the duplicate match.

### Agent 3 (Efficiency) — 1 fix applied  
- **`replace_all` → `replace`**: A valid DOT graph has at most one `rankdir` directive, so `replace_all` needlessly scans the rest of the string after the first match. Changed to `replace` which stops after the first match — more efficient and clearer in intent. Return type (`Cow<'a, str>`) is unchanged.

Both fixes were applied and verified: all 106 graph-related tests pass and `cargo clippy --workspace -- -D warnings` is clean.