Clean. Here's a summary of what was fixed:

## Summary of Changes

1. **Removed dead code** (`spawn.rs`):
   - Deleted `build_fabro_argv()` — defined but never called; both callers constructed argv inline
   - Deleted `path_exists()` — trivial `Path::exists()` wrapper, never used outside the module
   - Deleted `current_exe_str()` — absorbed into the new shared helper
   - Deleted `write_temp_json()` — absorbed into the new shared helper
   - Removed unused `use std::path::Path` import
   - Removed 3 tests for the deleted functions

2. **Eliminated copy-paste** (`spawn.rs`, `sender.rs`, `panic.rs`):
   - Extracted `spawn_fabro_subcommand()` in `spawn.rs` — a single function that handles the repeated pattern of: serialize JSON → write temp file → resolve current exe → spawn detached with `FABRO_TELEMETRY=off`
   - Simplified `spawn_sender()` in `sender.rs` from 20 lines to 6
   - Simplified `spawn_panic_sender()` in `panic.rs` from 20 lines to 6

3. **Fixed dead `eprintln!`** (`spawn.rs`):
   - After `fork::close_fd()` closes stderr, the `eprintln!` for exec failure was a no-op. Changed to `let _err =` since nothing can be done after fd close.

4. **Restored missing trailing newlines** in `Cargo.toml` and `lib/crates/fabro-util/Cargo.toml`.