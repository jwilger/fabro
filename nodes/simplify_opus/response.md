## Summary

**One bug found and fixed** across the three review agents:

- **Code Reuse**: Clean — no duplicates found. The existing `truncate_lines` in `fabro-agent` uses head+tail strategy, not tail-only.
- **Efficiency**: Clean — not a hot path, trivial allocations.
- **Code Quality — Indentation bug (fixed)**: `format!("    {}", tail_lines(...))` only indented the first line of multiline output. When truncation fires, the omission header got indented but all subsequent tail lines lost their indent, breaking the markdown code block formatting.

**Fix**: Added an `indent` parameter to `tail_lines` so it applies the indent to every line. Call sites now pass their indent directly (`"    "` for compact, `"  "` for summary_high) instead of wrapping in `format!`. This also fixes the pre-existing bug where multiline stdout/stderr was only indented on the first line.