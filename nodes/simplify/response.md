Here's a summary of what was fixed:

### Fixes applied

1. **Reuse existing `format_duration_ms`** (`runs.rs`): Replaced inline `|ms| super::progress::format_duration_short(Duration::from_millis(ms))` with `super::progress::format_duration_ms` — an existing helper in `progress.rs` that does exactly the same thing. Made `format_duration_ms` `pub(crate)` to enable this. Also removed the now-unused `use std::time::Duration` import.

2. **Merged redundant conditions** (`runs.rs`): `format_relative_time` had two separate `if` checks (`secs < 0` and `secs < 60`) both returning `"just now"`. Since `secs < 0` is a subset of `secs < 60`, removed the redundant first check.

3. **Removed needless lifetime** (`runs.rs`): `style_status<'a>` had an explicit lifetime `'a` on both parameters but returns an owned `String` — the lifetime serves no purpose. Simplified to `style_status(status: &RunStatus, styles: &Styles)`.

4. **Computed separator from header length** (`runs.rs`): Replaced hardcoded `"-".repeat(110)` with `"-".repeat(header.len())` so the separator always matches the actual column widths.

5. **Restored trailing newlines** (`main.rs`, `runs.rs`, `progress.rs`): All three files had lost their POSIX trailing newlines.

### Skipped (not worth addressing)
- Wildcard `_` in `style_status` match: Reasonable for a display function — explicitly listing all `StageStatus` variants would add maintenance burden without meaningful safety benefit.
- All efficiency findings: `fabro ps` is an interactive CLI command, not a hot path. The scan/clone patterns are appropriate for the expected cardinality.