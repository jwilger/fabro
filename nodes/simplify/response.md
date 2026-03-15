All 41 relevant tests pass (plus 9 ignored that require Docker/Daytona). Here's a summary of what was fixed:

## Summary

Three issues were identified and fixed:

1. **Duplicated `parse_source` / `split_run_path`** — `parse_source` in `asset.rs` reimplemented the exact same colon-splitting logic as `split_run_path` in `cp.rs`. Fixed by making `split_run_path` `pub(crate)` and rewriting `parse_source` as a thin wrapper around it. This went from 7 lines of duplicated logic to a 4-line delegation.

2. **Double iteration with duplicated filename extraction in flat copy mode** — The flat copy path in `cp_command` iterated `&entries` twice: once for collision detection and once for copying, with identical filename extraction logic (`Path::new(&entry.relative_path).file_name()...to_string_lossy()...`) copy-pasted in both loops. Fixed by collecting `(filename, &entry)` pairs into a `Vec` during the collision-detection pass, then iterating that Vec for copies — extracting each filename only once.

3. **`format_size` called twice per entry in `list_command`** — Once during column-width calculation, once during display. This is trivial overhead for a CLI command operating on a small number of entries, so skipped as not worth the added complexity of pre-computing.