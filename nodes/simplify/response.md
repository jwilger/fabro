Everything is clean. Here's a summary of what was fixed:

## Summary of Changes

### Issues Fixed

1. **Removed `VersionComparison` enum** — was a trivial 1:1 wrapper around `std::cmp::Ordering`. The call site now uses `target.cmp(&current)` with `Ordering::Less`/`Ordering::Equal` directly. Deleted the enum, the `compare_versions()` function, and 3 associated tests that were just testing `semver::Version::cmp`.

2. **Extracted `GITHUB_REPO` constant** — `"fabro-sh/fabro"` appeared 4 times as raw strings. Now a single `const GITHUB_REPO: &str = "fabro-sh/fabro"` used everywhere.

3. **Stored `reqwest::Client` in `Backend::Http` variant** — was constructing a new `reqwest::Client` for every HTTP call (2-3 times). Now `Backend::Http(reqwest::Client)` is built once in `select_backend()` and reused across all calls. Extracted `http_client()` helper.

4. **Converted `select_backend()` to async** — was using blocking `std::process::Command` which blocks the tokio runtime. Now uses `tokio::process::Command` consistently with the rest of the Backend methods.

5. **Parallel downloads with `tokio::try_join!`** — tarball and checksum downloads were sequential. Now run concurrently.

6. **Streaming SHA256 verification** — `verify_checksum()` was reading the entire tarball (10-30 MB) into memory. Now takes a `&Path` and uses `BufReader` + `std::io::copy` to stream through the hasher with ~8KB buffer. Tests updated to write temp files.

7. **Removed TOCTOU `.exists()` check** — `extracted_binary.exists()` before `fs::rename` was redundant; the rename itself will fail with a clear error if the source doesn't exist.

8. **Eliminated config double-load** — `check_and_print_notice()` was calling `load_cli_config(None)` again even though `main_inner()` already loaded it. Now the `upgrade_check` bool is extracted during the initial config load and passed to `spawn_upgrade_check()`.

9. **Background upgrade check** — `maybe_print_upgrade_notice()` was `.await`ed before the command, blocking startup by potentially 100ms-2s (sync shell commands + network request). Now `spawn_upgrade_check()` returns a `JoinHandle` that runs concurrently with the main command. The notice prints after the command completes.

10. **Fixed trailing newline** in `Cargo.toml`.

11. **Ran `cargo fmt`** to fix formatting.

### Not Changed (false positives / acceptable as-is)
- **Manual `Default` impl for `CliConfig`** — needed because `upgrade_check` defaults to `true` (not `false`). Already guarded by the `parse_empty_config_defaults` test that asserts `CliConfig::default() == toml::from_str("")`.