Everything is clean. Here's a summary of all the changes I made:

## Changes Summary

### 1. `lib/crates/fabro-workflows/src/cli/fork.rs` (New file)
Created the fork command implementation with:
- **`ForkArgs`** struct — CLI args: `run_id`, optional `target`, `--list`, `--no-push`
- **`fork_command()`** — Entry point that discovers the repo, resolves run ID, builds timeline, handles `--list`, resolves target (defaults to latest checkpoint), calls `execute_fork`, and prints the resume command
- **`execute_fork()`** — Core logic that:
  1. Generates a new ULID for the forked run
  2. Creates a new run branch pointing at the target checkpoint's `run_commit_sha`
  3. Creates a new metadata branch with updated manifest (new `run_id`, `run_branch`, `start_time`), graph, checkpoint (read from the specific metadata commit via `read_blob_at`), and optional `sandbox.json`
  4. Optionally pushes both new branches to origin (without force-push, since they're new refs)
  5. Returns the new run ID
- **5 tests**:
  - `fork_creates_new_run_branch` — verifies new run branch points at target run commit
  - `fork_creates_new_metadata_branch` — verifies manifest has new run_id, checkpoint matches target
  - `fork_preserves_original_run` — verifies source branches unchanged after fork
  - `fork_defaults_to_latest_checkpoint` — no target = fork from last checkpoint
  - `fork_at_specific_ordinal` — `@2` forks from second checkpoint

### 2. `lib/crates/fabro-workflows/src/cli/mod.rs`
- Added `pub mod fork;`

### 3. `lib/crates/fabro-workflows/src/cli/rewind.rs`
- Changed `fn load_parallel_map` → `pub fn load_parallel_map` so fork.rs can reuse it

### 4. `lib/crates/fabro-cli/src/main.rs`
- Added `Fork(fabro_workflows::cli::fork::ForkArgs)` variant to `Command` enum
- Added `Command::Fork(_) => "fork"` to command name mapping
- Added dispatch arm calling `fabro_workflows::cli::fork::fork_command`